## 2026-06-20

### Configurable AI Provider URL

- Replaced the OpenAI-specific API key label with a generic API key field.
- Added an AI provider URL field for LLM mode, defaulting to `https://api.openai.com/v1/chat/completions`.
- Routed LLM paragraph-break requests through the configured provider URL and validated that it is an HTTP(S) URL before calling it.
- Left the app static-only; no server or OpenAPI files were added.
