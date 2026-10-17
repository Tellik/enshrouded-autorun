use super::ui_snapshot_types::Activity;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

pub(crate) fn read_activity(root: &Path) -> Result<Vec<Activity>, String> {
    let file = File::open(shroudforge_loader_config::paths::current_log(root))
        .map_err(|error| error.to_string())?;
    let mut items = BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .enumerate()
        .filter_map(|(index, line)| {
            let close = line.find("] [")?;
            let severity = match line.as_bytes().first().copied()? {
                b'[' => line.chars().nth(1)?,
                _ => return None,
            };
            let (level, result) = match severity {
                'T' => ("trace", "TRACE"),
                'D' => ("debug", "DEBUG"),
                'I' => ("info", "INFO"),
                'W' => ("warn", "WARN"),
                'E' => ("error", "ERROR"),
                _ => return None,
            };
            let time_text = line.get(3..close)?;
            let source_end = line.get(close + 3..)?.find("] ")? + close + 3;
            let source = line.get(close + 3..source_end)?.to_owned();
            let message = line.get(source_end + 2..)?.to_owned();
            Some(Activity {
                id: format!("{}-{}-{}", index, time_text, source),
                time: time_text.to_owned(),
                source,
                action: message,
                result: result.to_owned(),
                level: level.to_owned(),
            })
        })
        .collect::<Vec<_>>();
    items.reverse();
    Ok(items)
}

// Log check state transitions once instead of repeating the same status on every refresh.
