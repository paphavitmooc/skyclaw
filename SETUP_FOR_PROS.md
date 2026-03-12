# SkyClaw Setup — Quick Reference

Assumes familiarity with Rust, Docker, VPS management, and API key handling.

## Requirements

- Rust 1.82+ (or Docker)
- Chrome/Chromium (optional, for browser tool)
- Telegram bot token via [@BotFather](https://t.me/BotFather)

## Build

```bash
git clone https://github.com/nagisanzenin/skyclaw.git && cd skyclaw
cargo build --release   # ~2.5min cold, 9.6 MB binary
```

## Authentication

### Codex OAuth (ChatGPT Plus/Pro)

```bash
skyclaw auth login                    # browser flow
skyclaw auth login --headless         # headless (paste redirect URL)
skyclaw auth login --output ./o.json  # export token for containers
skyclaw auth status                   # check expiry
```

Tokens last ~10 days. Stored at `~/.skyclaw/oauth.json`. Auto-detected at startup.

### API Key

Skip auth. Start the bot, paste any supported key in Telegram. Auto-detected:

| Prefix | Provider |
|--------|----------|
| `sk-ant-` | Anthropic |
| `sk-` | OpenAI |
| `AIzaSy` | Gemini |
| `xai-` | Grok |
| `sk-or-` | OpenRouter |

Or use the OTK secure setup link (AES-256-GCM encrypted client-side).

## Run

```bash
export TELEGRAM_BOT_TOKEN="..."
skyclaw start                          # foreground
skyclaw start -d                       # daemon (logs: ~/.skyclaw/skyclaw.log)
skyclaw start -d --log /var/log/sk.log # custom log path
skyclaw stop                           # graceful shutdown
```

## Configuration

Config file: `skyclaw.toml` (project root) or `~/.skyclaw/skyclaw.toml`.

```toml
[provider]
name = "anthropic"
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-sonnet-4-6"

[agent]
max_spend_usd = 5.0   # 0.0 = unlimited (default)

[channel.telegram]
enabled = true
token = "${TELEGRAM_BOT_TOKEN}"
allowlist = []         # empty = auto-whitelist first user
file_transfer = true

[memory]
backend = "sqlite"

[security]
sandbox = "mandatory"
```

Environment variables are expanded via `${VAR}` syntax. Full schema: `crates/skyclaw-core/src/types/config.rs`.

## Docker

```bash
# Authenticate on host
skyclaw auth login --output ./oauth.json

# Or set API key as env var
echo "ANTHROPIC_API_KEY=sk-ant-..." > .env
```

```yaml
# docker-compose.yml
services:
  skyclaw:
    build: .
    environment:
      - TELEGRAM_BOT_TOKEN=${TELEGRAM_BOT_TOKEN}
    volumes:
      - ./oauth.json:/root/.skyclaw/oauth.json      # Codex OAuth
      - ./skyclaw.toml:/root/.skyclaw/skyclaw.toml   # config
      - skyclaw-data:/root/.skyclaw                   # persistent state
    restart: unless-stopped

volumes:
  skyclaw-data:
```

Telegram config auto-injects from `TELEGRAM_BOT_TOKEN` env var — no need to edit `skyclaw.toml` for the token.

## VPS Deployment (systemd)

```bash
# Build on server (or cross-compile and scp the binary)
cargo build --release
sudo cp target/release/skyclaw /usr/local/bin/

# Create systemd service
sudo tee /etc/systemd/system/skyclaw.service << 'EOF'
[Unit]
Description=SkyClaw AI Agent
After=network.target

[Service]
Type=simple
User=skyclaw
Environment=TELEGRAM_BOT_TOKEN=your-token
Environment=ANTHROPIC_API_KEY=your-key
ExecStart=/usr/local/bin/skyclaw start
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable --now skyclaw
journalctl -u skyclaw -f  # tail logs
```

Minimum VPS: 512 MB RAM, 1 vCPU. SkyClaw idles at 15 MB RSS.

## Multi-Provider Setup

Switch providers live in Telegram with `/model`:

```
/model                    # list available models
/model gpt-5.4           # switch to GPT-5.4
/model claude-sonnet-4-6 # switch to Claude
```

Or via natural language: "Switch to GPT-5.2"

Credentials stored at `~/.skyclaw/credentials.toml` — the agent reads and edits this file itself.

## MCP Servers

Add external tool servers at runtime:

```
/mcp add fetch npx -y @modelcontextprotocol/server-fetch
/mcp add github npx -y @modelcontextprotocol/server-github
/mcp                    # list connected servers
/mcp remove fetch       # disconnect
```

Or let the agent self-extend — it searches the 14-server built-in registry by capability.

Config: `~/.skyclaw/mcp.toml`

## Key Paths

| Path | Purpose |
|------|---------|
| `~/.skyclaw/` | Home directory (all persistent state) |
| `~/.skyclaw/credentials.toml` | Provider API keys (encrypted) |
| `~/.skyclaw/oauth.json` | Codex OAuth tokens |
| `~/.skyclaw/memory.db` | SQLite memory backend |
| `~/.skyclaw/allowlist.toml` | User whitelist |
| `~/.skyclaw/custom-tools/` | Agent-authored script tools |
| `~/.skyclaw/mcp.toml` | MCP server configuration |
| `~/.skyclaw/skyclaw.log` | Daemon log (with `-d`) |

## Updating

```bash
skyclaw update   # git pull + cargo build --release
# or manually:
git pull && cargo build --release
```

## Compilation Gates (for contributors)

```bash
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo test --workspace    # 1,378 tests, 0 failures
```

All four must pass before any commit to main.

## Design Documents

| Document | Topic |
|----------|-------|
| [COGNITIVE_ARCHITECTURE.md](docs/design/COGNITIVE_ARCHITECTURE.md) | The Finite Brain Model — context as working memory |
| [BLUEPRINT_SYSTEM.md](docs/design/BLUEPRINT_SYSTEM.md) | Blueprint procedural memory vision |
| [BLUEPRINT_MATCHING_V2.md](docs/design/BLUEPRINT_MATCHING_V2.md) | Zero-extra-LLM-call matching architecture |
| [BLUEPRINT_IMPLEMENTATION.md](docs/design/BLUEPRINT_IMPLEMENTATION.md) | Step-by-step implementation plan |
| [OTK_SECURE_KEY_SETUP.md](docs/OTK_SECURE_KEY_SETUP.md) | AES-256-GCM encrypted onboarding |
| [BENCHMARK_REPORT.md](docs/benchmarks/BENCHMARK_REPORT.md) | Performance benchmarks vs OpenClaw/ZeroClaw |
