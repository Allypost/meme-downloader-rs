use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::bot::telegram::{
    download_helper::{self, DownloadResult},
    Command,
};
use anyhow::anyhow;
use async_recursion::async_recursion;
use config::CONFIGURATION;
use futures::{self};
use helpers::{dirs::create_temp_dir, results::option_contains};
use log::{debug, error, info, trace};
use rayon::{
    prelude::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSlice,
};
use teloxide::{
    net::Download,
    prelude::*,
    types::{
        InputFile, InputMedia, InputMediaPhoto, InputMediaVideo, Me, MediaAnimation, MediaKind,
        MediaPhoto, MediaText, MediaVideo, MessageCommon, MessageEntityKind, MessageKind,
    },
    utils::command::BotCommands,
};
use tokio::fs::File;

pub struct MessageHandler<'a> {
    bot: &'a Bot,
    me: &'a Me,
    msg: &'a Message,
    is_owner: bool,
}

impl<'a> MessageHandler<'a> {
    pub fn new(bot: &'a Bot, me: &'a Me, msg: &'a Message) -> Self {
        let owner_id = CONFIGURATION
            .telegram
            .as_ref()
            .map(|x| x.owner_id)
            .unwrap_or_default();
        let is_owner = msg
            .from()
            .map_or(false, |sender| option_contains(&owner_id, &sender.id.0));

        Self {
            bot,
            me,
            msg,
            is_owner,
        }
    }

    pub async fn handle(&self) -> Result<(), String> {
        let msg = &self.msg;
        trace!("Handling message: {id:?}", id = msg.id);

        let parsed_cmd = msg.text().or_else(|| msg.caption()).map(|text| {
            let cmd: Result<Command, teloxide::utils::command::ParseError> =
                BotCommands::parse(text, self.me.username());

            cmd
        });

        match parsed_cmd {
            Some(Ok(Command::SplitScenes)) => {
                debug!("Got command: {cmd:?}", cmd = parsed_cmd);
                self.handle_split_cmd(msg).await?;
                return Ok(());
            }
            Some(Ok(Command::Help)) => {
                debug!("Got command: {cmd:?}", cmd = parsed_cmd);
                self.send_reply(Command::descriptions().to_string().as_str())
                    .await?;
                return Ok(());
            }
            None | Some(Err(_)) => {}
        }

        match &msg.kind {
            MessageKind::Common(MessageCommon { media_kind, .. }) => {
                self.handle_media_kind(media_kind).await?;
            }
            _ => {
                info!("Unknown message kind: {kind:?}", kind = msg.kind);
            }
        }

        Ok(())
    }

    pub async fn send_reply(&self, text: &str) -> Result<Message, String> {
        self.bot
            .send_message(self.msg.chat.id, text)
            .reply_to_message_id(self.msg.id)
            .await
            .map_err(|e| format!("Error while sending reply: {e:?}"))
    }

    pub async fn edit_message(&self, msg_to_edit: &Message, text: &str) -> Result<Message, String> {
        self.bot
            .edit_message_text(self.msg.chat.id, msg_to_edit.id, text)
            .await
            .map_err(|e| format!("Error while editing message: {e:?}"))
    }

    #[async_recursion]
    async fn handle_split_cmd(&self, msg: &Message) -> Result<(), String> {
        trace!("Handling split command for message {id:?}", id = msg.id);

        if let MessageKind::Common(MessageCommon {
            reply_to_message,
            media_kind,
            ..
        }) = &msg.kind
        {
            let file_id = match &media_kind {
                MediaKind::Video(MediaVideo { video, .. }) => {
                    trace!("Got video: {video:?}");
                    Some(&video.file.id)
                }

                MediaKind::Animation(MediaAnimation { animation, .. }) => {
                    trace!("Got animation: {animation:?}");
                    Some(&animation.file.id)
                }

                _ => None,
            };
            if let Some(file_id) = file_id {
                return split_msg_video(self, file_id)
                    .await
                    .map_err(|e| format!("Error while splitting video into scenes: {e:?}"));
            }

            if let Some(reply_to_message) = reply_to_message {
                return self.handle_split_cmd(reply_to_message).await;
            }
        }

        self.send_reply("Must be either a reply to a video message or be the text of a message containing video").await?;
        Ok(())
    }

    async fn handle_media_kind(&self, kind: &MediaKind) -> Result<(), String> {
        match kind {
            MediaKind::Text(MediaText { text, entities, .. }) => {
                trace!("Got text: {text:?}");

                let urls = entities
                    .iter()
                    .filter(|e| e.kind == MessageEntityKind::Url)
                    .map(|e| {
                        let t = text.clone();

                        t[e.offset..e.offset + e.length].to_string()
                    })
                    .collect::<Vec<_>>();

                if urls.is_empty() {
                    self.send_reply("No URLs found in message").await?;
                    return Ok(());
                }

                let status_msg = self.send_reply("Received URL(s). Processing...").await?;

                let result = urls
                    .into_iter()
                    .map(|x| {
                        tokio::task::spawn_blocking(move || download_helper::download_tmp_file(&x))
                    })
                    .collect::<Vec<_>>();
                let result = futures::future::join_all(result).await;
                let download_results = result
                    .into_iter()
                    .collect::<Result<Result<Vec<_>, String>, _>>()
                    .map_err(|e| format!("Error while downloading file:\n\n{e:?}"))??;

                self.edit_message(&status_msg, "Downloaded files. Uploading here...")
                    .await?;

                let files = files_to_input_media(
                    download_results
                        .iter()
                        .flat_map(DownloadResult::files)
                        .collect::<Vec<_>>(),
                );

                self.bot
                    .send_media_group(self.msg.chat.id, files)
                    .reply_to_message_id(self.msg.id)
                    .await
                    .map_err(|e| format!("Error while sending media group: {e:?}"))?;

                if self.is_owner {
                    let new_paths = download_results
                        .par_iter()
                        .map(DownloadResult::move_files_to_memes_dir)
                        .collect::<Result<Vec<_>, _>>()?;
                    let new_paths = new_paths.into_iter().flatten().collect::<Vec<_>>();
                    info!("Downloaded files: {new_paths:?}");
                }

                download_results
                    .par_iter()
                    .map(DownloadResult::cleanup)
                    .collect::<Result<_, _>>()?;

                self.bot
                    .delete_message(self.msg.chat.id, status_msg.id)
                    .await
                    .map_err(|e| format!("Error while deleting status message: {e:?}"))?;

                Ok(())
            }

            MediaKind::Photo {
                0: MediaPhoto { photo, .. },
            } => {
                trace!("Got photo: {photo:?}");

                self.send_reply("Received photo").await?;

                Ok(())
            }

            MediaKind::Video {
                0: MediaVideo { video, .. },
            } => {
                trace!("Got video: {video:?}");

                self.send_reply("Received video").await?;

                Ok(())
            }

            _ => Err("Unknown media kind".to_string()),
        }
    }
}

async fn split_msg_video(
    handler: &MessageHandler<'_>,
    telegram_file_id: &str,
) -> anyhow::Result<()> {
    trace!("Splitting video: {id:?}", id = telegram_file_id);

    let status_msg = handler
        .send_reply("Splitting video...")
        .await
        .map_err(|e| anyhow!(e))?;

    let f = handler.bot.get_file(telegram_file_id).await?;
    trace!("Got file: {:?}", f);

    let download_dir = create_temp_dir()?;
    defer! {
        if let Err(e) = fs::remove_dir_all(&download_dir) {
            error!("Error while removing temp dir: {e:?}");
        }
    }

    let download_file_path = download_dir.join(format!("1.{}", f.meta.unique_id));
    let _downloaded_file = {
        let mut file = File::create(&download_file_path).await?;
        handler.bot.download_file(&f.path, &mut file).await?;
        trace!("Downloaded file: {:?}", file);

        file
    };
    let downloaded_file_path = download_file_path.clone();

    let scene_files = {
        let download_dir = download_dir.clone();

        let mut scene_files = tokio::task::spawn_blocking(move || {
            fixers::split_scenes::split_into_scenes(
                fixers::split_scenes::SplitVideoConfig::new(&download_dir, &download_file_path)
                    .with_file_template("0.$SCENE_NUMBER.$START_FRAME-$END_FRAME"),
            )
        })
        .await?
        .map_err(|e| anyhow!("Error while splitting video into scenes:\n\n{e:?}", e = e))?;

        scene_files.sort_unstable();

        scene_files
            .into_iter()
            .filter(|x| x.as_os_str() != downloaded_file_path.as_os_str())
            .collect::<Vec<_>>()
    };

    handler
        .edit_message(&status_msg, "Split video. Uploading here...")
        .await
        .map_err(|e| anyhow!(e))?;

    let reqs = send_files(handler, &scene_files).await?;

    trace!("Uploaded files: {reqs:?}", reqs = reqs);

    handler
        .bot
        .delete_message(handler.msg.chat.id, status_msg.id)
        .await?;

    Ok(())
}

async fn send_files<'a>(
    handler: &MessageHandler<'a>,
    files: &[PathBuf],
) -> anyhow::Result<Vec<Message>> {
    let reqs = files
        .iter()
        .as_slice()
        .par_chunks(10)
        .map(files_to_input_media)
        .map(|files| {
            handler
                .bot
                .send_media_group(handler.msg.chat.id, files)
                .reply_to_message_id(handler.msg.id)
                .send()
        })
        .collect::<Vec<_>>();
    let reqs = futures::future::join_all(reqs).await;
    let reqs = reqs
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    trace!("Uploaded files: {reqs:?}", reqs = reqs);

    Ok(reqs)
}

fn files_to_input_media<TFiles, TFile>(files: TFiles) -> Vec<InputMedia>
where
    TFiles: IntoIterator<Item = TFile>,
    TFile: AsRef<Path> + Into<PathBuf> + Clone,
{
    files
        .into_iter()
        .filter_map(|file_path| {
            let input_file = InputFile::file(file_path.clone());

            let file_type = match infer::get_from_path(file_path) {
                Ok(Some(f)) => f.mime_type().split_once('/').map(|x| x.0),
                _ => None,
            };

            let res = match file_type {
                Some("image") => InputMedia::Photo(InputMediaPhoto {
                    media: input_file,
                    caption: None,
                    caption_entities: None,
                    parse_mode: None,
                    has_spoiler: false,
                }),

                Some("video") => InputMedia::Video(InputMediaVideo {
                    media: input_file,
                    thumb: None,
                    caption: None,
                    caption_entities: None,
                    parse_mode: None,
                    has_spoiler: false,
                    width: None,
                    height: None,
                    duration: None,
                    supports_streaming: None,
                }),

                _ => return None,
            };

            Some(res)
        })
        .collect::<Vec<_>>()
}
