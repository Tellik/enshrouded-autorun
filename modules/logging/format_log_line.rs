use crate::*;

pub(crate) struct LineFormat;
pub(crate) static STARTED: OnceLock<Instant> = OnceLock::new();

pub fn timestamp() -> String {
    let elapsed = STARTED.get_or_init(Instant::now).elapsed();
    let total_seconds = elapsed.as_secs();
    format!(
        "{:02}:{:02}:{:02},{:03}",
        total_seconds / 3600,
        total_seconds / 60 % 60,
        total_seconds % 60,
        elapsed.subsec_millis()
    )
}

fn single_line(value: &str) -> String {
    value.replace('\r', "\\r").replace('\n', "\\n")
}

pub fn format_line(level: char, source: &str, message: &str) -> String {
    let source = single_line(source).replace(']', "_");
    format!(
        "[{level} {}] [{source}] {}",
        timestamp(),
        single_line(message)
    )
}

#[derive(Default)]
struct Fields {
    message: String,
    mod_id: Option<String>,
    extra: Vec<String>,
}

impl Visit for Fields {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        let value = format!("{value:?}");
        match field.name() {
            "message" => self.message = value,
            "mod_id" => self.mod_id = Some(value.trim_matches('"').to_owned()),
            _ => self.extra.push(format!("{}={value}", field.name())),
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "message" => self.message = value.to_owned(),
            "mod_id" => self.mod_id = Some(value.to_owned()),
            _ => self.extra.push(format!("{}={value}", field.name())),
        }
    }
}

impl<S, N> FormatEvent<S, N> for LineFormat
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        _context: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let level = match *event.metadata().level() {
            tracing::Level::TRACE => 'T',
            tracing::Level::DEBUG => 'D',
            tracing::Level::INFO => 'I',
            tracing::Level::WARN => 'W',
            tracing::Level::ERROR => 'E',
        };
        let mut fields = Fields::default();
        event.record(&mut fields);
        let target = event.metadata().target();
        let source = fields.mod_id.as_deref().unwrap_or_else(|| match target {
            target if target.starts_with("shroudforge_api") => "api",
            target if target.starts_with("shroudforge_package") => "package",
            target if target.starts_with("shroudforge_modloader") => "runtime",
            _ => target.strip_prefix("shroudforge::").unwrap_or(target),
        });
        write!(writer, "{}", format_line(level, source, &fields.message))?;
        if !fields.extra.is_empty() {
            write!(writer, " {}", single_line(&fields.extra.join(" ")))?;
        }
        writeln!(writer)
    }
}
