# API Server

Xberg runs as an HTTP REST API server (`xberg serve`) or as an MCP server (`xberg mcp`) for AI agent integration.

## HTTP REST API

### Start

=== "CLI"

    --8<-- "snippets/api_server/cli.md"

=== "Docker"

    --8<-- "snippets/api_server/docker.md"

=== "Python"

    --8<-- "snippets/api_server/python.md"

=== "Rust"

    --8<-- "snippets/api_server/rust.md"

=== "Go"

    --8<-- "snippets/api_server/go.md"

=== "Java"

    --8<-- "snippets/api_server/java.md"

=== "C#"

    --8<-- "snippets/api_server/csharp.md"

### Endpoints

#### POST /extract

Extract text from uploaded files via multipart form data.

| Field           | Required         | Description                                      |
| --------------- | ---------------- | ------------------------------------------------ |
| `files`         | Yes (repeatable) | Files to extract                                 |
| `config`        | No               | JSON config overrides                            |
| `output_format` | No               | `plain` (default), `markdown`, `djot`, or `html` |

```bash title="Terminal"
# Single file
curl -F "files=@document.pdf" http://localhost:8000/extract

# Multiple files
curl -F "files=@doc1.pdf" -F "files=@doc2.docx" http://localhost:8000/extract

# With config overrides
curl -F "files=@scanned.pdf" \
     -F 'config={"ocr":{"language":"eng"},"force_ocr":true}' \
     http://localhost:8000/extract
```

```json title="Response"
{
  "results": [
    {
      "content": "Extracted text...",
      "mime_type": "application/pdf",
      "metadata": { "page_count": 10, "author": "John Doe" },
      "tables": [],
      "detected_languages": ["eng"],
      "chunks": null,
      "images": null
    }
  ],
  "errors": [],
  "summary": {
    "inputs": 1,
    "results": 1,
    "errors": 0
  }
}
```

#### POST /extract-async

Queue an extraction job and return immediately. Accepts the same multipart form
data or JSON body as `/extract`. Returns `202 Accepted` with a job identifier.
Returns `429 Too Many Requests` when the concurrent job limit is reached.

```bash title="Terminal"
curl -F "files=@document.pdf" http://localhost:8000/extract-async
```

```json title="Response (202)"
{ "job_id": "550e8400-e29b-41d4-a716-446655440000" }
```

#### GET /jobs/{job_id}

Poll the status of an async job. `state` is one of `pending`, `running`,
`completed`, or `failed`. The `result` field is present only when
`state == completed`; the `error` field only when `state == failed`. Jobs
expire after 5 minutes and return `404` once evicted.

```bash title="Terminal"
curl http://localhost:8000/jobs/550e8400-e29b-41d4-a716-446655440000
```

```json title="Response"
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "state": "completed",
  "created_at": "2026-07-07T12:00:00Z",
  "updated_at": "2026-07-07T12:00:03Z",
  "result": { "results": [], "errors": [], "summary": {} }
}
```

#### Other Endpoints

| Endpoint          | Method | Description                                                               |
| ----------------- | ------ | ------------------------------------------------------------------------- |
| `/health`         | GET    | `{"status":"healthy","version":"1.0.0-rc.12"}`                            |
| `/version`        | GET    | `{"version":"1.0.0-rc.12"}`                                               |
| `/detect`         | POST   | MIME type detection (multipart)                                           |
| `/formats`        | GET    | List supported formats                                                   |
| `/cache/stats`    | GET    | Cache statistics                                                          |
| `/cache/warm`     | POST   | Pre-download models                                                       |
| `/cache/manifest` | GET    | Model manifest with checksums                                             |
| `/cache/clear`    | DELETE | Clear all cached files                                                    |
| `/info`           | GET    | `{"version":"...","rust_backend":true}`                                   |
| `/openapi.json`   | GET    | OpenAPI 3.1 schema                                                        |

### Client Examples

=== "Python"

    --8<-- "snippets/python/api/client_extract_single_file.md"

=== "TypeScript"

    --8<-- "snippets/typescript/getting-started/client_extract_single_file.md"

=== "Rust"

    --8<-- "snippets/rust/api/client_extract_single_file.md"

=== "Go"

    --8<-- "snippets/go/api/client_extract_single_file.md"

=== "Java"

    --8<-- "snippets/java/api/client_extract_single_file.md"

=== "C#"

    --8<-- "snippets/csharp/client_extract_single_file.md"

=== "Ruby"

    --8<-- "snippets/ruby/api/client_extract_single_file.md"

### Error Handling

```json title="Error response"
{
  "error_type": "ValidationError",
  "message": "Invalid file format",
  "status_code": 400
}
```

| Status | Error type                 | Meaning           |
| ------ | -------------------------- | ----------------- |
| 400    | `ValidationError`          | Invalid input     |
| 422    | `ParsingError`, `OcrError` | Processing failed |
| 500    | Internal errors            | Server errors     |

=== "Python"

    --8<-- "snippets/python/utils/error_handling_extract.md"

=== "TypeScript"

    --8<-- "snippets/typescript/api/error_handling_extract.md"

=== "Rust"

    --8<-- "snippets/rust/api/error_handling_extract.md"

=== "Go"

    --8<-- "snippets/go/api/error_handling_extract.md"

=== "Java"

    --8<-- "snippets/java/api/error_handling_extract.md"

=== "C#"

    --8<-- "snippets/csharp/error_handling_extract.md"

=== "Ruby"

    --8<-- "snippets/ruby/api/error_handling_extract.md"

### Configuration

The server discovers `xberg.toml` in the current and parent directories. Pass `--config path/to/file` to use a different file.

| Variable                          | Default     | Description                        |
| --------------------------------- | ----------- | ---------------------------------- |
| `XBERG_MAX_REQUEST_BODY_BYTES`    | `104857600` | Max request body size in bytes     |
| `XBERG_MAX_MULTIPART_FIELD_BYTES` | `104857600` | Max multipart field size in bytes  |
| `XBERG_CORS_ORIGINS`              | `*`         | Comma-separated allowed origins    |

!!! Warning Default CORS allows all origins. Set `XBERG_CORS_ORIGINS` explicitly in production.

See [Configuration Guide](configuration.md) for all options.

---

## MCP Server

### Start

```bash title="Terminal"
xberg mcp
xberg mcp --config xberg.toml
```

=== "Python"

    --8<-- "snippets/python/mcp/mcp_server_start.md"

=== "TypeScript"

    --8<-- "snippets/typescript/mcp/mcp_server_start.md"

=== "Rust"

    --8<-- "snippets/rust/mcp/mcp_server_start.md"

=== "Go"

    --8<-- "snippets/go/mcp/mcp_server_start.md"

=== "Java"

    --8<-- "snippets/java/mcp/mcp_server_start.md"

=== "C#"

    --8<-- "snippets/csharp/mcp_server_start.md"

=== "Ruby"

    --8<-- "snippets/ruby/mcp/mcp_server_start.md"

### Tools

The MCP server exposes `extract`, `extract_batch`, `detect_mime_type`, `list_formats`, `get_version`, and the `cache_*` tools. See the [MCP Reference](../reference/mcp.md) for the full tool list, parameters, and schemas.

All extraction tools accept an optional `config` object. URI and byte payload details live in `ExtractInput` as `kind = "uri"` or `kind = "bytes"`.

### Batch Extraction

=== "Python"

    --8<-- "snippets/python/mcp/mcp_batch_extract.md"

### AI Agent Integration

=== "Claude Desktop"

    Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

    ```json
    {
      "mcpServers": {
        "xberg": {
          "command": "xberg",
          "args": ["mcp"]
        }
      }
    }
    ```

=== "Python"

    --8<-- "snippets/python/mcp/mcp_custom_client.md"

=== "Python (HTTP)"

    --8<-- "snippets/python/mcp/mcp_http_client.md"

=== "LangChain"

    --8<-- "snippets/python/mcp/mcp_langchain_integration.md"

=== "TypeScript"

    --8<-- "snippets/typescript/mcp/mcp_custom_client.md"

=== "Rust"

    --8<-- "snippets/rust/mcp/mcp_custom_client.md"

=== "Go"

    --8<-- "snippets/go/mcp/mcp_custom_client.md"

=== "Java"

    --8<-- "snippets/java/mcp/mcp_client.md"

=== "C#"

    --8<-- "snippets/csharp/mcp_custom_client.md"

=== "Ruby"

    --8<-- "snippets/ruby/mcp/mcp_custom_client.md"

---

For container deployment, see the [Docker Guide](docker.md).
