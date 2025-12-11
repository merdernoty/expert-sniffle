mod commands;
mod handlers;
mod messages;

use anyhow::Result;
use env_logger::Env;
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use teloxide::{dispatching::UpdateFilterExt, dptree, prelude::*};

use crate::commands::Command;
use crate::handlers::{handle_done, handle_start};

#[tokio::main]
async fn main() -> Result<()> {
    load_dotenv();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    log::info!("Starting bot");
    log::info!(
        "Raw REQUIRED_CHANNELS: {:?}, parsed: {:?}",
        std::env::var("REQUIRED_CHANNELS"),
        crate::messages::required_channels()
    );

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_callback_query().endpoint(handle_done))
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(handle_start),
        );

    let mut dispatcher = Dispatcher::builder(bot, handler)
        .default_handler(|_| async {}) // swallow unhandled updates to avoid warn spam
        .enable_ctrlc_handler()
        .build();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .unwrap_or(8000);
    log::info!("Requested health server port: {}", port);

    let bot_task = tokio::spawn(async move {
        dispatcher.dispatch().await;
        Ok::<(), anyhow::Error>(())
    });

    let server_task = tokio::spawn(run_health_server(port));

    let (bot_res, server_res) = tokio::join!(bot_task, server_task);
    bot_res??;
    server_res??;

    Ok(())
}

fn load_dotenv() {
    let path = PathBuf::from(".env");
    match dotenvy::from_path(&path) {
        Ok(_) => println!("Loaded .env from {:?}", path),
        Err(err) => {
            println!(
                "Could not load .env via dotenvy ({}). Trying manual parse.",
                err
            );
            if let Err(manual_err) = load_env_relaxed(&path) {
                println!(
                    "Manual parse of .env failed ({}). Env vars must be set by host.",
                    manual_err
                );
            }
        }
    };
}

fn load_env_relaxed(path: &PathBuf) -> Result<()> {
    let content = fs::read_to_string(path)?;
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((key, val)) = trimmed.split_once('=') else {
            println!("Skipping malformed .env line {}: {}", idx + 1, line);
            continue;
        };

        let key = key.trim();
        let value = val
            .trim()
            .trim_matches(|ch| ch == '"' || ch == '\'' || ch == '\r');
        // Convert \n sequences to newlines.
        let value_owned = value.replace("\\n", "\n");
        // rust-analyzer may flag set_var as unsafe; wrap explicitly.
        unsafe {
            std::env::set_var(key, value_owned);
        }
    }
    Ok(())
}

async fn run_health_server(port: u16) -> Result<()> {
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Request, Response, Server};

    let listener = TcpListener::bind(("0.0.0.0", port)).or_else(|err| {
        log::warn!(
            "Failed to bind port {}: {}. Trying ephemeral port.",
            port,
            err
        );
        TcpListener::bind(("0.0.0.0", 0))
    })?;

    let local_port = listener.local_addr()?.port();
    log::info!("Health server bound on port {}", local_port);

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(|_req: Request<Body>| async move {
            Ok::<_, hyper::Error>(Response::new(Body::from("ok")))
        }))
    });

    Server::from_tcp(listener)?.serve(make_svc).await?;
    Ok(())
}
