//! News, installation events and notification read state.
pub mod documents;
pub use shroudforge_loader_config::{config, paths};

mod mark_news_read;
mod mark_news_unread;
mod notification;
mod notification_paths;
mod read_news;
mod read_news_state;
mod read_notifications;
mod sync_mod_events;
mod validate_news_id;
mod validate_notification_event;
mod write_mod_event;
mod write_notification_event;

pub use mark_news_read::{mark_news_read, mark_read};
pub use mark_news_unread::{mark_news_unread, mark_unread};
pub use notification::{ModNotification, Notice, NoticeAction, NoticeLink};
pub use notification_paths::{directory, events_directory};
pub use read_news::read;
pub use read_news_state::{read_ids, read_news_state};
pub use read_notifications::read_notifications;
pub use sync_mod_events::sync_mod_events;
pub use validate_news_id::valid_news_id;
pub use validate_notification_event::validate_event;
pub use write_mod_event::write_event_notice;
pub use write_notification_event::write_event;

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests;
