# nano-rust — CLAUDE.md

Minimal Rust coding agent. Single file: `src/main.rs`. ~150 LOC.

## Architecture

- **No async** — uses `reqwest::blocking`. Keeps the code linear and readable.
- **No abstraction layers** — one struct (`Msg`), four functions (`load_env`, `run_shell`, `call_api`, `main`).
- **OpenAI-compatible API** — works with OpenRouter, direct Anthropic via compatible proxy, or any endpoint.
- **Single tool** — `shell`: runs `sh -c <command>`, returns stdout or stderr+stdout on failure.

## Key design decisions

- `Msg` uses `Option<Value>` fields with `skip_serializing_if` so tool/assistant messages serialize correctly without extra variants.
- Tool results are pushed as individual `role: "tool"` messages with `tool_call_id` matching the request.
- `.env` is parsed manually (no `dotenv` crate) — just `split_once('=')`.
- Full conversation history is sent every request — no summarization, no truncation.

## Files

```
src/main.rs     # entire implementation
Cargo.toml      # 3 dependencies
.env            # runtime config (not committed)
.env.example    # template
```

## Environment variables

- `OPENROUTER_API_KEY` — checked first; fallback to `ANTHROPIC_API_KEY`
- `INFERENCE_BASE_URL` — API base (default: `https://openrouter.ai/api/v1`)
- `MODEL_NAME` — model string (default: `anthropic/claude-sonnet-4-6`)

## Build & run

```bash
cargo build --release
./target/release/nano-rust
```

## Extending

To add a tool: add an entry to the `tools` array in `call_api()`, then add a match arm in the tool execution loop in `main()`. That's it.
