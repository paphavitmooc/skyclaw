# SkyClaw Setup Guide — For Beginners

This guide walks you through every step of setting up SkyClaw from scratch. No prior experience with Rust, servers, or AI APIs required.

## What You'll Need

- A computer (macOS, Linux, or Windows with WSL)
- An internet connection
- A Telegram account (free)
- **One** of these (pick whichever you have):
  - A ChatGPT Plus or Pro subscription ($20/month) — easiest, no API key needed
  - An API key from any AI provider (Anthropic, OpenAI, Google, etc.)

## Step 1: Install Rust

SkyClaw is written in Rust. You need the Rust compiler to build it.

Open your terminal and run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

When it asks, press `1` for the default installation. After it finishes:

```bash
source $HOME/.cargo/env
```

Verify it worked:

```bash
rustc --version
# Should print something like: rustc 1.82.0 (...)
```

> **Windows users:** Install [WSL2](https://learn.microsoft.com/en-us/windows/wsl/install) first (`wsl --install` in PowerShell), then run these commands inside WSL.

## Step 2: Install Chrome (Optional)

SkyClaw has a browser tool that can navigate websites, click buttons, take screenshots, and fill forms. It needs Chrome or Chromium installed.

- **macOS:** Chrome is probably already installed. If not, download from google.com/chrome.
- **Linux:** `sudo apt install chromium-browser` (Ubuntu/Debian) or `sudo dnf install chromium` (Fedora).
- **Skip this** if you don't need the browser tool — everything else works without it.

## Step 3: Create a Telegram Bot

This is how you'll talk to SkyClaw. It takes 60 seconds.

1. Open Telegram on your phone or desktop
2. Search for **@BotFather** and open a chat with it
3. Send `/newbot`
4. BotFather asks for a name — type anything (e.g., "My SkyClaw")
5. BotFather asks for a username — pick something ending in `bot` (e.g., `my_skyclaw_bot`)
6. BotFather gives you a **bot token** — it looks like `7123456789:AAHx...`. **Copy this token and save it somewhere safe.**

> Your bot token is a secret. Anyone with it can control your bot. Don't share it publicly.

## Step 4: Download and Build SkyClaw

```bash
git clone https://github.com/nagisanzenin/skyclaw.git
cd skyclaw
cargo build --release
```

The first build takes 2-4 minutes (it's compiling ~300 dependencies). Subsequent builds are much faster.

When it finishes, your binary is at `./target/release/skyclaw`.

## Step 5: Connect an AI Provider

You need an AI brain for SkyClaw. Choose ONE option:

### Option A: Use Your ChatGPT Account (Easiest)

If you have ChatGPT Plus ($20/month) or ChatGPT Pro:

```bash
./target/release/skyclaw auth login
```

A browser window opens. Log into your ChatGPT account. That's it.

You'll see:
```
Authenticated successfully!
Email:   you@gmail.com
Expires: 239h 59m
Model:   gpt-5.4 (default)
```

> **No browser on your server?** Use `skyclaw auth login --headless` — it prints a URL you can open on any device (phone, laptop), then you paste the redirect URL back into the terminal.

### Option B: Use an API Key

If you have an API key from Anthropic, OpenAI, Google, or another provider, you'll paste it in Telegram after starting (next step). No setup needed here.

Where to get API keys:
- **Anthropic (Claude):** https://console.anthropic.com/settings/keys — starts with `sk-ant-`
- **OpenAI (GPT):** https://platform.openai.com/api-keys — starts with `sk-`
- **Google (Gemini):** https://aistudio.google.com/apikey — starts with `AIzaSy`
- **xAI (Grok):** https://console.x.ai/ — starts with `xai-`

## Step 6: Start SkyClaw

Set your Telegram bot token and start:

```bash
export TELEGRAM_BOT_TOKEN="paste-your-bot-token-here"
./target/release/skyclaw start
```

You should see logs indicating the gateway is starting and Telegram is connecting.

## Step 7: Talk to Your Bot

1. Open Telegram
2. Find the bot you created in Step 3
3. Send any message

**If you used Option A (ChatGPT login):** SkyClaw is ready. Start chatting.

**If you're using Option B (API key):** SkyClaw will send you a secure setup link. You have two choices:
- Click the link, paste your API key in the browser form (encrypted locally before sending)
- Or just paste your raw API key directly in the chat — SkyClaw auto-detects the provider

After the key is validated, you're live.

## Step 8: Try It Out

Send these to your bot:

- `Hello!` — basic chat
- `What files are in my home directory?` — uses the shell tool
- `Remember that my favorite color is blue` — stores in memory
- `What's my favorite color?` — recalls from memory
- `/model` — see available AI models

## Running in the Background

Once everything works, run SkyClaw as a background daemon:

```bash
./target/release/skyclaw start -d
```

It logs to `~/.skyclaw/skyclaw.log`. Stop it with:

```bash
./target/release/skyclaw stop
```

## Updating SkyClaw

When new versions are released:

```bash
./target/release/skyclaw update
```

This pulls the latest code and rebuilds automatically.

## Troubleshooting

**"cargo: command not found"**
Run `source $HOME/.cargo/env` or restart your terminal.

**Build fails with "linker not found"**
Install build essentials: `sudo apt install build-essential` (Ubuntu/Debian) or `xcode-select --install` (macOS).

**Bot doesn't respond**
- Check that `TELEGRAM_BOT_TOKEN` is set: `echo $TELEGRAM_BOT_TOKEN`
- Check logs: `tail -50 /tmp/skyclaw.log` or `tail -50 ~/.skyclaw/skyclaw.log`
- Make sure you sent a message to the right bot

**"OAuth token expired"**
Re-authenticate: `./target/release/skyclaw auth login`

**API key not accepted**
- Make sure you copied the full key (no extra spaces)
- Check that your API key has credits/quota remaining
- Try pasting the raw key directly in chat instead of using the secure link

## What's Next

- Read about [what makes SkyClaw different](README.md#what-makes-skyclaw-different) — the Finite Brain Model and Blueprint procedural memory
- Set up [Discord](docs/channels/discord.md) or [Slack](docs/channels/slack.md) channels
- Explore [MCP servers](README.md#self-extending-tool-system) — let the agent install its own tools
- Deploy on a VPS for 24/7 operation — see [SETUP_FOR_PROS.md](SETUP_FOR_PROS.md) for Docker and systemd guides
