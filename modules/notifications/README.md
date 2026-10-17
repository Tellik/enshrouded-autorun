# Notifications module

This module owns built-in news, local mod events, read state, the news view, and toast messages. It does not read Modloader window files. The Modloader passes the required mod details explicitly as `ModNotification`.

| Action | Implementation |
| --- | --- |
| Read built-in news | `read_news.rs` |
| Combine events and news | `read_notifications.rs` |
| Read the state and repeat intervals | `read_news_state.rs` |
| Mark news as read or unread | `mark_news_read.rs`, `mark_news_unread.rs` |
| Validate and save an event | `validate_notification_event.rs`, `write_notification_event.rs` |
| Record installations and version changes | `sync_mod_events.rs`, `write_mod_event.rs` |
| Build localized news items | `ui/build-news-items.ts` |
| Display the list, an item, and its popup | `ui/show-news.tsx`, `ui/show-notice-card.tsx`, `ui/show-notice-popup.tsx` |
| Display action results as toasts | `ui/show-toasts.tsx` |

`documents.rs` defines the module's persisted document formats. News, events, and read state keep their existing JSON fields and storage locations. Toasts remain visible for 4.5 seconds after success and 8 seconds after an error. Multiple saved-setting results are still combined into one toast within 900 milliseconds.

Shared display resources and translations come from `modules/ui-resources`. The UI sends IPC requests through `modules/transport/ui/send-host-command.ts`. The Modloader application composes the views and owns navigation.
