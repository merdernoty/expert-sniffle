mod commands;
mod handlers;
mod messages;

use anyhow::Result;
use teloxide::{dispatching::UpdateFilterExt, dptree, prelude::*};

use crate::commands::Command;
use crate::handlers::{handle_done, handle_start};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();
    log::info!("Starting bot");
    log::info!(
        "Переменная REQUIRED_CHANNELS сырая: {:?}",
        std::env::var("REQUIRED_CHANNELS")
    );
    log::info!(
        "Каналы для проверки: {:?}",
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

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
