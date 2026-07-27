# All AI generation goes through OpenRouter via the openrouter-rs crate

The four per-provider integrations (Anthropic SDK, raw `ureq` for OpenAI/Google/BFL) are replaced by a single OpenRouter client using the community `openrouter-rs` crate, hidden behind the existing `ImageGenerator`/`TextGenerator` traits. OpenRouter has no official Rust SDK; `openrouter-rs` was the only crate covering both chat and image generation. Provider-level diversity is replaced by config-driven Model Pools (`text_models`, `image_models`) with a random draw per generation; the old per-feature model pinnings (`replace_art`, `mixes`) are dropped in favor of the pools.

## Considered Options

- **async-openai pointed at OpenRouter** — more mature crate, but no support for OpenRouter's image generation surface; image calls would be hand-rolled anyway.
- **Raw reqwest** — no single-maintainer risk, but we would own the typed wrappers; rejected in favor of deleting code, with the traits as the swap-out seam if `openrouter-rs` stalls.
- **Keep per-provider integrations** — four API keys, four response shapes, sync `ureq` blocking tokio workers, triplicated base64/webp helpers. This was the pain being solved.

## Consequences

- One API key (`OPENROUTER_API_KEY`) replaces four; single point of failure and a billing relationship with OpenRouter.
- `openrouter-rs` is pre-1.0 with one maintainer — the trait boundary keeps swapping it out cheap.
- Failed generations fail like before; no cross-model retry was added.
