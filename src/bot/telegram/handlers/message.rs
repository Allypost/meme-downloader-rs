use std::path::{Path, PathBuf};

use crate::{
    bot::telegram::download_helper::{self, DownloadResult},
    config::CONFIGURATION,
    helpers::results::option_contains,
};
use futures;
use log::{info, trace};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use teloxide::{
    prelude::*,
    types::{
        InputFile, InputMedia, InputMediaPhoto, InputMediaVideo, MediaKind, MediaPhoto, MediaText,
        MediaVideo, MessageCommon, MessageEntityKind, MessageKind,
    },
};

pub struct MessageHandler {
    bot: Bot,
    msg: Message,
    is_owner: bool,
}

impl MessageHandler {
    pub fn new(bot: Bot, msg: Message) -> Self {
        let owner_id = CONFIGURATION
            .telegram
            .as_ref()
            .map(|x| x.owner_id)
            .unwrap_or_default();
        let is_owner = msg
            .from()
            .map_or(false, |sender| option_contains(&owner_id, &sender.id.0));

        Self { bot, msg, is_owner }
    }

    pub async fn handle(&self) -> Result<(), String> {
        let msg = &self.msg;
        trace!("Handling message: {id:?}", id = msg.id);
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
