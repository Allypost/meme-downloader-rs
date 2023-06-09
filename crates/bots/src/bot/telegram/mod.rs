use crate::bot::telegram::handlers::message::MessageHandler;
use config::CONFIGURATION;
use log::{debug, error, info, trace};
use reqwest::Url;
use std::{process::exit, thread};
use teloxide::{prelude::*, types::Me, utils::command::BotCommands};
use tokio::runtime;

mod download_helper;
mod handlers;

#[derive(Debug, BotCommands)]
#[command(
    rename_rule = "camelCase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Display this help message")]
    Help,
    #[command(
        description = "Split the video into scenes (best effort). Must be a reply to a video message or text of a video message."
    )]
    SplitScenes,
}

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

    let bot_api_url = CONFIGURATION
        .telegram
        .as_ref()
        .and_then(|x| x.api_url.clone())
        .unwrap_or("https://api.telegram.org".to_string());

    let bot_api_url = Url::parse(&bot_api_url).unwrap_or_else(|e| {
        error!("Error while parsing Telegram API URL: {}", e);
        exit(1);
    });

    let bot = Bot::new(bot_token).set_api_url(bot_api_url);

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
    let handler = Update::filter_message().endpoint(|bot: Bot, msg: Message, me: Me| async move {
        trace!("Received message: {msg:?}");
        thread::spawn(move || {
            trace!("Spawned new thread for message handler");

            let msg_handler = MessageHandler::new(&bot, &me, &msg);

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
                    .allow_sending_without_reply(true)
                    .reply_to_message_id(msg.id)
                    .send(),
            );
            if let Err(e) = res {
                error!("Error while sending error message: {e:?}");
            }
        });

        respond(())
    });

    let handler_tree = dptree::entry().branch(handler);

    Dispatcher::builder(bot, handler_tree)
        .build()
        .dispatch()
        .await;
}
