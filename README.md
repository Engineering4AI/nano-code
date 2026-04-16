# nano-rust

The smallest possible coding agent in Rust. ~150 LOC, single binary, zero fluff.

## How it works

```
you ──► agent loop ──► LLM API
                          │
                    tool_calls?
                          │ yes
                    run shell cmd
                          │
                    tool results ──► LLM API (repeat)
                          │ no (end_turn)
                    print response
```

### Three moving parts

**1. `load_env()`** — reads `.env` on startup, sets env vars. No crate needed.

**2. `call_api()`** — POST to any OpenAI-compatible `/chat/completions` endpoint with one tool registered: `shell`. Sends the full conversation history each turn (stateless HTTP, stateful client).

**3. `main()` agent loop** — two nested loops:
- Outer: reads your prompt, appends as `user` message, enters inner loop.
- Inner: calls API → if `finish_reason == "tool_calls"`, executes each shell command and appends `tool` result messages, then calls API again. Breaks when `finish_reason == "end_turn"` (or anything else).

### Message flow (OpenAI format)

```
user:      { role: "user",      content: "your prompt" }
assistant: { role: "assistant", tool_calls: [{id, function: {name, arguments}}] }
tool:      { role: "tool",      tool_call_id: id, content: "cmd output" }
assistant: { role: "assistant", content: "final answer" }
```

The model decides when to run shell commands and when to stop. You just provide the goal.

## Setup

```bash
cp .env.example .env
# edit .env with your key
cargo run
```

## Configuration (`.env`)

| Variable | Default | Description |
|---|---|---|
| `OPENROUTER_API_KEY` | required | API key (falls back to `ANTHROPIC_API_KEY`) |
| `INFERENCE_BASE_URL` | `https://openrouter.ai/api/v1` | Any OpenAI-compatible base URL |
| `MODEL_NAME` | `anthropic/claude-sonnet-4-6` | Model identifier |

## Build

```bash
cargo build --release
./target/release/nano-rust
```

## Dependencies

| Crate | Purpose |
|---|---|
| `reqwest` (blocking) | HTTP client |
| `serde` + `serde_json` | JSON serialization |

Nothing else.
