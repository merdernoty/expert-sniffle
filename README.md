# Instagram to Telegram Bot

Minimal Telegram bot that funnels users from Instagram to your Telegram channels/groups. Built with Rust 2021, Tokio, and teloxide.

## Features
- `/start` sends a channel subscription prompt with an inline **Done** button.
- Callback handles the button, checks channel subscriptions, and sends a guide message.
- Texts and channels are configurable via environment variables.

## Prerequisites
- Rust toolchain (edition 2021)
- Telegram bot token

## Configuration
Set environment variables (use a `.env` file or export directly):
```
TELOXIDE_TOKEN=<your_bot_token>
GUIDE_MESSAGE="Thank you! Here is the guide..."
REQUIRED_CHANNELS="@channel1,@channel2"
```

## Run
```
cargo run
```

## Files
- `src/main.rs` — entry point, dispatcher wiring
- `src/commands.rs` — bot commands
- `src/handlers.rs` — message and callback handlers
- `src/messages.rs` — message templates and env parsing
