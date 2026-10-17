# Benachrichtigungen

Dieses Modul besitzt Nachrichten, lokale Ereignisse, ihren Lesestand sowie die Nachrichtenansicht und Toasts. Es liest keine Modloader-Fensterdateien ein. Der Modloader übergibt die benötigten Modangaben ausdrücklich als `ModNotification`.

| Handlung | Implementierung |
| --- | --- |
| Mitgelieferte Nachrichten lesen | `read_news.rs` |
| Ereignisse und Nachrichten zusammenstellen | `read_notifications.rs` |
| Lesestand mit Wiederholungsfristen lesen | `read_news_state.rs` |
| Nachrichten als gelesen oder ungelesen speichern | `mark_news_read.rs`, `mark_news_unread.rs` |
| Ereignis prüfen und speichern | `validate_notification_event.rs`, `write_notification_event.rs` |
| Installationen und Versionswechsel als Ereignisse festhalten | `sync_mod_events.rs`, `write_mod_event.rs` |
| Nachrichten für die gewählte Sprache aufbauen | `ui/build-news-items.ts` |
| Nachrichtenliste, einzelne Nachricht und Popup anzeigen | `ui/show-news.tsx`, `ui/show-notice-card.tsx`, `ui/show-notice-popup.tsx` |
| Ergebnisse als Toast anzeigen | `ui/show-toasts.tsx` |

`documents.rs` enthält die eigenen Dokumentverträge. Nachrichten, Ereignisse und Lesestand behalten ihre bisherigen JSON-Felder und Speicherorte. Toastzeiten bleiben bei 4,5 Sekunden für Erfolg und 8 Sekunden für Fehler. Mehrere gespeicherte Einstellungen werden weiterhin innerhalb von 900 Millisekunden zu einer Meldung zusammengefasst.

Gemeinsame Anzeigeelemente und Übersetzungen kommen aus `modules/ui-resources`. IPC-Nachrichten sendet `modules/transport/ui/send-host-command.ts`. Die App des Modloaders setzt die Ansichten zusammen und führt die Navigation aus.
