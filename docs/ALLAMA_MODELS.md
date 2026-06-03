# Allama model layout (Ollama-aligned)

Exodus uses **native Rust Allama** (`allama serve`, default port **11435**). Model discovery follows **Ollama-style Modelfile + per-model directories**, not Exodus’s separate CMake tree under `Exodus/allama/` (that tree is for bundled GGUF only).

## Ollama vs Allama (this install)

| Topic | Ollama | Allama (Rust, your build) |
|--------|--------|---------------------------|
| Models root | `~/.ollama/models` (manifests + blobs) | `~/.allama/models` |
| One logical model | Registry name + tag `name:tag` | **Subdirectory name** = API name (`gemma4-e2b`) |
| Definition file | `Modelfile` | `Modelfile` in model dir (same directives: `FROM`, `PARAMETER`, `SYSTEM`, …) |
| Required for `list` / `/api/tags` | Registry entry | **`metadata.json`** in `~/.allama/models/<name>/` |
| Weights file | Blob store | **`.gguf`** in that directory (prefer `<name>/<name>.gguf`) |
| Create custom model | `ollama create mymodel -f ./Modelfile` | `allama create mymodel -f ./Modelfile` |
| Copy model | `ollama cp src dst` | `allama cp src dst` |
| Default port | 11434 | **11435** (Exodus `ai_port`) |

## Canonical directory layout (valid model)

```
~/.allama/models/
  gemma4-e2b/
    metadata.json          # required — drives /api/tags
    gemma4-e2b.gguf        # preferred weight path
    Modelfile              # optional — parameters/template (Ollama-compatible)
```

`metadata.json` example (your `gemma4-e2b`):

```json
{
  "name": "gemma4-e2b",
  "tag": "latest",
  "size": 3462678272,
  "modified": "2026-05-09T23:48:55.483420+00:00",
  "digest": "sha256:gemma4-e2b",
  "details": {
    "architecture": "gemma4",
    "parameters": "2B",
    "context_length": 8192,
    "quantization": "Q4_K_M"
  }
}
```

**API rule:** HTTP `model` field and Exodus **Chat model** setting must use the **folder name** (`gemma4-e2b`), not HuggingFace repo names (`gemma-4-E4B-it-Q4_K_M`).

## Your three “recommended” models — verified

These appear in `GET /api/tags` and have real GGUF weights:

| API name | GGUF | Listed size | Notes |
|----------|------|-------------|--------|
| `gemma4-e2b` | `gemma4-e2b.gguf` | ~3.3 GB | Good default for chat |
| `gemma4-4b` | `gemma4-4b.gguf` | ~5.1 GB | Higher quality, slower |
| `gemma4-26b-moe` | `gemma4-26b-moe.gguf` | ~17 GB | Needs large RAM; slow load |

`gemma4-26b-moe-new` is a **second directory** with the same weights filename inside; both list separately (Ollama-style: directory name wins).

## Common misconfigurations on this machine

### 1. Folders without `metadata.json`

Many dirs under `~/.allama/models/` (e.g. `Qwen3.5-4B-MLX-4bit`, `gemma-4-4b`) are **not** registered — Allama skips them. They will **not** show in `/api/tags` or Exodus until you add `metadata.json` or run `allama create`.

### 2. Loose `.gguf` in models root

Files like `~/.allama/models/Qwen3.6-35B-A3B-Q4_K_M.gguf` are **ignored** until placed in a named subdirectory with `metadata.json`.

### 3. Zero-size / stub models in API

| Name | Issue |
|------|--------|
| `cli-smoke-model` | Test stub, no real GGUF — not for real inference |
| `llama3-copy`, `llama3-api-copy`, `llama3-new-copy` | `metadata.json` only, **no** `.gguf` |
| `qwen3.6-35b-a3b` | Custom metadata; GGUF present but **wrong filename** (`Qwen_Qwen3.6-35B-A3B-GGUF.gguf` ≠ `qwen3.6-35b-a3b.gguf`) — may fail to load |

### 4. Exodus bundle `Exodus/allama/models/gemma-4-E4B/`

Has Ollama-style `Modelfile` + `gemma-4-E4B-it-Q4_K_M.gguf` but **no** `metadata.json` until imported:

```bash
cd /path/to/Exodus/allama/models/gemma-4-E4B
allama create gemma-4-e4b -f Modelfile
```

Then set Exodus **Chat model** to `gemma-4-e4b` (the name you chose in `create`, not the GGUF filename).

### 5. Stray root `Modelfile`

`~/.allama/models/Modelfile` with only `FROM llama3.2` is **not** a model — safe to remove or move into a proper model directory.

## Exodus UI settings

| Setting | Must match |
|---------|------------|
| AI port | `11435` |
| Chat model | Exact name from `/api/tags` (e.g. `gemma4-e2b`) |
| Embedding model | Model that supports `/v1/embeddings` (if used) |

Wrong chat model examples: `exodus-default`, `llama2`, `gemma-4-E4B-it-Q4_K_M` (unless you `allama create` that name).

## Register a new model (Ollama workflow)

```bash
# 1) Write Modelfile (same as Ollama)
cat > Modelfile <<'EOF'
FROM ./my-model.Q4_K_M.gguf
PARAMETER temperature 0.7
PARAMETER num_ctx 8192
SYSTEM You are a helpful assistant.
EOF

# 2) Create (like ollama create)
allama create my-model -f Modelfile

# 3) Confirm
curl -s http://127.0.0.1:11435/api/tags | jq '.models[].name'

# 4) Test
curl -s http://127.0.0.1:11435/api/chat -d '{
  "model": "my-model",
  "messages": [{"role":"user","content":"hi"}],
  "stream": false
}'
```

## Per-model files (Ollama Modelfile + Allama extras)

After cleanup, each production model directory should contain:

| File | Purpose |
|------|---------|
| `Modelfile` | Ollama-compatible definition (`FROM`, `PARAMETER`, `TEMPLATE`, `SYSTEM`) |
| `template.jinja` | Chat template (minijinja; Gemma `<start_of_turn>` format) |
| `system.txt` | Default system prompt for Exodus |
| `parameters.json` | Sampling / context defaults |
| `<name>.gguf` | Weights (or symlink) |
| `metadata.json` | Registry entry for `/api/tags` |

Presets live in `Exodus/allama/model-presets/`. Apply cleanup + enrich:

```bash
sh scripts/cleanup-and-enrich-allama-models.sh
```

Archives removed stubs to `~/.allama/models/.archive-*` and loose GGUF to `.incoming-loose/`.

## Audit script

```bash
sh scripts/check-allama-models.sh
```

Lists valid models, broken entries, and loose GGUF files compared to Ollama/Allama rules.
