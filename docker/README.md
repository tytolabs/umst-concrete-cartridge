# UMST agent Docker image

Local stdio MCP for Cursor / SDK — **not** a hosted MCP service.

## Build

```bash
cd umst-concrete-cartridge
docker build -f docker/Dockerfile.agent -t umst-mcp:agent-layer .
```

Publish tag (manual, maintainer):

```bash
docker tag umst-mcp:agent-layer ghcr.io/tytolabs/umst-concrete-cartridge:agent-layer
docker push ghcr.io/tytolabs/umst-concrete-cartridge:agent-layer
# Pin digest in server.json after publish:
docker buildx imagetools inspect ghcr.io/tytolabs/umst-concrete-cartridge:agent-layer --format '{{json .Manifest}}'
```

## Run (stdio)

```bash
docker run -i --rm \
  -e UMST_MEMORY_DB=/data/memory.db \
  -v "$(pwd)/.umst-memory:/data" \
  umst-mcp:agent-layer
```

## Cursor `mcp.json`

```json
{
  "mcpServers": {
    "umst": {
      "command": "docker",
      "args": [
        "run", "-i", "--rm",
        "-e", "UMST_MEMORY_DB=/data/memory.db",
        "-v", "/absolute/path/.umst-memory:/data",
        "ghcr.io/tytolabs/umst-concrete-cartridge:agent-layer"
      ]
    }
  }
}
```

## Registry manifest

See [`server.json`](server.json) for a static MCP registry entry (stdio transport). No HTTP/OAuth until hosted MCP is a product goal.

## Docs

- [`docs/AGENT_MCP.md`](../docs/AGENT_MCP.md) — tools, env, runbook
- [`contributions/README.md`](../contributions/README.md) — git federated memory inbox
