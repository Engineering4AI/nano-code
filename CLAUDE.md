# nano-code - CLAUDE.md

Minimal Rust coding agent. Single file: `src/main.rs`.

## Architecture

- **No async** — uses `reqwest::blocking`. Keeps the code linear and readable.
- **No abstraction layers** — one struct (`Msg`), five functions (`load_env`, `shell`, `read_file`, `write_file`, `dispatch`, `call_api`, `main`).
- **OpenAI-compatible API** — works with OpenRouter, or any `/chat/completions` endpoint.
- **Three tools** — `shell`, `read_file`, `write_file`.
- **Executor-mode system prompt** — forces the model to act (write files, run commands) rather than describe.
- **Ralph loop** — each user prompt becomes the active goal; `end_turn` is not completion unless the assistant marks it with `GOAL_COMPLETE`.

## Key design decisions

- `Msg` uses `Option<Value>` fields with `skip_serializing_if` so tool/assistant messages serialize correctly without extra variants.
- Tool results are pushed as individual `role: "tool"` messages with `tool_call_id` matching the request.
- `.env` is parsed manually (no `dotenv` crate) — just `split_once('=')`.
- Full conversation history is sent every request — no summarization, no truncation.
- `system` field sent on every API call alongside `messages`.
- `run_agent_loop()` handles the standard model/tool-call loop and returns the assistant's final text for that pass.
- `main()` wraps each user prompt in a Ralph loop: if final text does not start with `GOAL_COMPLETE`, it appends a continuation prompt for the original goal and runs again.
- There is intentionally no max Ralph iteration cap. A model that never emits `GOAL_COMPLETE` will keep running.

## Files

```
src/main.rs     # entire implementation
Cargo.toml      # 3 dependencies
.env            # runtime config (not committed)
.env.example    # template
```

## Environment variables

- `OPENROUTER_API_KEY` — required
- `INFERENCE_BASE_URL` — API base (default: `https://openrouter.ai/api/v1`)
- `MODEL_NAME` — model string (default: `anthropic/claude-sonnet-4-6`)

## Build & run

```bash
cargo build --release
./target/release/nano-code
```

## Extending

To add a tool: add an entry to the `tools` array in `call_api()`, add a match arm in `dispatch()`. That's it.
