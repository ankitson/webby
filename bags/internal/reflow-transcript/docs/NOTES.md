## 2026-06-20 - Configurable AI Provider URL

Goal: let LLM mode call a user-supplied AI provider endpoint and API key instead of a hardcoded OpenAI URL.

Decisions:
- Kept the tool as a static single-page HTML app.
- Added a configurable provider URL field defaulting to OpenAI's chat completions endpoint.
- Kept the request/response contract OpenAI chat-completions compatible, which covers OpenAI-compatible hosted and local providers.

Verification:
- Parsed the inline script with Bun via `new Function(...)` without executing browser code.
