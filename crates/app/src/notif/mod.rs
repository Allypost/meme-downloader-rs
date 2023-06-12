use notify_rust::{error::Error, Notification, NotificationHandle, Timeout, Urgency};

#[derive(Debug)]
pub struct NotificationInfo {
    pub urgency: Urgency,
    pub timeout: Timeout,
    pub icon: String,
    pub title: String,
    pub message: String,
}

pub fn send_notification(info: &NotificationInfo) -> Result<NotificationHandle, Error> {
    let mut notif = Notification::new();

    if cfg!(target_os = "linux") {
        notif.urgency(info.urgency);
    }

    let notif = notif
        .appname("meme downloader")
        .timeout(info.timeout)
        .summary(&info.title)
        .body(&info.message)
        .icon(&info.icon);

    notif.show()
}
