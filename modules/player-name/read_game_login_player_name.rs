use std::fs;
use std::path::Path;

const LOGIN_PREFIX: &str = "[server] player '";
const LOGIN_SUFFIX: &str = "' logged in with permissions:";
const MAX_PLAYER_NAME_LENGTH: usize = 80;

pub(super) fn read_active_player_name(root: &Path) -> Option<String> {
    let bytes = fs::read(root.join("enshrouded.log")).ok()?;
    let log = String::from_utf8_lossy(&bytes);
    latest_login_name(&log)
}

fn latest_login_name(log: &str) -> Option<String> {
    for line in log.lines().rev() {
        if !line.to_ascii_lowercase().contains(LOGIN_PREFIX) {
            continue;
        }
        return parse_login_line(line);
    }
    None
}

fn parse_login_line(line: &str) -> Option<String> {
    let folded = line.to_ascii_lowercase();
    let name_start = folded.find(LOGIN_PREFIX)?.checked_add(LOGIN_PREFIX.len())?;
    let name_end = folded
        .get(name_start..)?
        .find(LOGIN_SUFFIX)?
        .checked_add(name_start)?;
    let name = line.get(name_start..name_end)?.trim();
    if name.is_empty()
        || name.chars().count() > MAX_PLAYER_NAME_LENGTH
        || name.chars().any(char::is_control)
    {
        return None;
    }
    Some(name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{latest_login_name, parse_login_line};

    #[test]
    fn reads_the_player_name_from_the_single_player_login_record() {
        let line = "[I 00:07:46,940] [server] Player 'bonsaibauer' logged in with Permissions:";
        assert_eq!(parse_login_line(line).as_deref(), Some("bonsaibauer"));
    }

    #[test]
    fn returns_the_latest_login_name_in_the_current_log() {
        let log = concat!(
            "[I 00:01:00,000] [server] Player 'OldName' logged in with Permissions:\n",
            "[I 00:07:46,940] [server] Player 'BonsaiBauer' logged in with Permissions:\n",
        );
        assert_eq!(latest_login_name(log).as_deref(), Some("BonsaiBauer"));
    }

    #[test]
    fn ignores_non_server_records_and_malformed_names() {
        assert_eq!(
            parse_login_line("[Client] Player 'BonsaiBauer' logged in with Permissions:"),
            None
        );
        assert_eq!(
            parse_login_line("[server] Player '' logged in with Permissions:"),
            None
        );
    }

    #[test]
    fn uses_the_most_recent_login_when_the_log_contains_older_player_names() {
        let log = concat!(
            "[I 00:01:00,000] [server] Player 'BonsaiBauer' logged in with Permissions:\n",
            "[I 00:02:00,000] [server] Player 'AnotherPlayer' logged in with Permissions:\n",
        );
        assert_eq!(latest_login_name(log).as_deref(), Some("AnotherPlayer"));
    }
}
