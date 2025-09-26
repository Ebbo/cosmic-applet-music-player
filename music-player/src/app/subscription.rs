use super::Message;
use cosmic::iced::time;
use std::time::Duration;

pub fn subscription() -> cosmic::iced::Subscription<Message> {
    time::every(Duration::from_millis(500)).map(|_| Message::FindPlayer)
}
