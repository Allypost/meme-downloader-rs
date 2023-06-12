use crate::bot::telegram::handlers::message::MessageHandler;
use config::CONFIGURATION;
use log::{debug, error, info, trace};
use std::{process::exit, thread};
use teloxide::prelude::*;
use tokio::runtime;

mod download_helper;
mod handlers;

pub async fn run() {
    let bot_token: &str = match CONFIGURATION
        .telegram
        .as_ref()
        .map(|t| t.bot_token.as_ref())
    {
        Some(token) => {
            info!("Starting Telegram bot");
            token
        }
        None => {
            error!("No Telegram bot token provided. Please provide one.");
            exit(1);
        }
    };

    let bot = Bot::new(bot_token);

    match bot.get_me().send().await {
        Ok(user) => {
            debug!(
                "Running Telegram bot: {name} (@{handle})",
                name = user.full_name(),
                handle = user.username(),
            );
        }
        Err(e) => {
            error!("Error while getting bot info: {}", e);
            exit(1);
        }
    }

    run_listener(bot).await;

    info!("Telegram bot stopped");
}

async fn run_listener(bot: Bot) {
    let handler = Update::filter_message().endpoint(|bot: Bot, msg: Message| async move {
        trace!("Received message: {msg:?}");
        thread::spawn(move || {
            trace!("Spawned new thread for message handler");

            let msg_handler = MessageHandler::new(bot.clone(), msg.clone());

            let runtime = runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Error while creating runtime: {e:?}"));

            let runtime = match runtime {
                Ok(runtime) => runtime,
                Err(e) => {
                    error!("Error while creating runtime: {e:?}");
                    return;
                }
            };

            let resp = runtime.block_on(msg_handler.handle());

            if resp.is_ok() {
                return;
            }

            let e = resp.unwrap_err();

            error!("Error while handling message: {e:?}");

            let res = runtime.block_on(
                bot.send_message(msg.chat.id, format!("Error: {e:?}"))
                    .reply_to_message_id(msg.id)
                    .send(),
            );
            if let Err(e) = res {
                error!("Error while sending error message: {e:?}");
            }
        });

        respond(())
    });

    Dispatcher::builder(bot, handler).build().dispatch().await;
}
