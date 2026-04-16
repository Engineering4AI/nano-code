use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, Write};
use std::process::Command;

const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1";
const DEFAULT_MODEL: &str = "anthropic/claude-sonnet-4-6";
const MAX_TOKENS: u32 = 8192;

#[derive(Serialize, Deserialize, Clone)]
struct Msg {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

fn load_env() {
    if let Ok(s) = std::fs::read_to_string(".env") {
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            if let Some((k, v)) = line.split_once('=') {
                std::env::set_var(k.trim(), v.trim().trim_matches('"'));
            }
        }
    }
}

fn run_shell(cmd: &str) -> String {
    match Command::new("sh").arg("-c").arg(cmd).output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout);
            let err = String::from_utf8_lossy(&o.stderr);
            if o.status.success() {
                if out.trim().is_empty() { "(no output)".into() } else { out.trim().to_string() }
            } else {
                format!("ERROR: {}{}", err.trim(), out.trim())
            }
        }
        Err(e) => format!("EXEC_ERROR: {e}"),
    }
}

fn call_api(client: &Client, key: &str, base_url: &str, model: &str, messages: &[Msg]) -> Value {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    client
        .post(&url)
        .header("Authorization", format!("Bearer {key}"))
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": model,
            "max_tokens": MAX_TOKENS,
            "tools": [{
                "type": "function",
                "function": {
                    "name": "shell",
                    "description": "Run a shell command and return stdout/stderr.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "command": {"type": "string"}
                        },
                        "required": ["command"]
                    }
                }
            }],
            "messages": messages
        }))
        .send()
        .expect("request failed")
        .json::<Value>()
        .expect("json parse failed")
}

fn main() {
    load_env();

    let key = std::env::var("OPENROUTER_API_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .expect("set OPENROUTER_API_KEY or ANTHROPIC_API_KEY");
    let base_url = std::env::var("INFERENCE_BASE_URL")
        .unwrap_or_else(|_| DEFAULT_BASE_URL.into());
    let model = std::env::var("MODEL_NAME")
        .unwrap_or_else(|_| DEFAULT_MODEL.into());

    let client = Client::new();
    let mut messages: Vec<Msg> = Vec::new();

    println!("nano-rust | {model} | empty line to quit\n");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_string();
        if input.is_empty() { break; }

        messages.push(Msg {
            role: "user".into(),
            content: Some(json!(input)),
            tool_calls: None,
            tool_call_id: None,
        });

        loop {
            let resp = call_api(&client, &key, &base_url, &model, &messages);
            let choice = &resp["choices"][0];
            let msg = &choice["message"];
            let finish = choice["finish_reason"].as_str().unwrap_or("");

            // Print text content
            if let Some(text) = msg["content"].as_str() {
                if !text.is_empty() { println!("\n{text}\n"); }
            }

            // Push assistant message (preserve tool_calls if present)
            messages.push(Msg {
                role: "assistant".into(),
                content: msg["content"].as_str().map(|s| json!(s)),
                tool_calls: msg.get("tool_calls").cloned(),
                tool_call_id: None,
            });

            if finish != "tool_calls" { break; }

            // Execute each tool call, push tool result messages
            let tool_calls = msg["tool_calls"].as_array().cloned().unwrap_or_default();
            for tc in &tool_calls {
                let id = tc["id"].as_str().unwrap_or("").to_string();
                let args: Value = serde_json::from_str(
                    tc["function"]["arguments"].as_str().unwrap_or("{}")
                ).unwrap_or(json!({}));
                let cmd = args["command"].as_str().unwrap_or("");
                println!("  $ {cmd}");
                let out = run_shell(cmd);
                println!("  {out}\n");
                messages.push(Msg {
                    role: "tool".into(),
                    content: Some(json!(out)),
                    tool_calls: None,
                    tool_call_id: Some(id),
                });
            }
        }
    }
}
