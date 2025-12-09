use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::Arc;

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::{info, warn};
use tracing_subscriber::field::Visit;
use tracing_subscriber::filter::{FilterExt, filter_fn};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Registry, fmt};

fn main() {
    if let Err(err) = init_logging() {
        eprintln!("Unable to initialize logging: {err}");
        std::process::exit(1);
    }

    info!("application started");
    info!(target: "access", method = "GET", path = "/health", status = 200, "http");
    warn!("something concerning happened");
}

fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    let app_layer = fmt::layer()
        .event_format(JsonFormatter::new("app.log"))
        .with_writer(AppWriter)
        .with_filter(filter_fn(|meta| meta.target() != "access"));

    let access_layer = fmt::layer()
        .event_format(JsonFormatter::new("access.log"))
        .with_writer(AccessWriter::new("access.log")?)
        .with_filter(filter_fn(|meta| meta.target() == "access"));

    Registry::default()
        .with(env_filter)
        .with(app_layer)
        .with(access_layer)
        .init();

    Ok(())
}

struct Rfc3339Timer;

impl Rfc3339Timer {
    fn now(&self) -> Result<String, time::error::Format> {
        OffsetDateTime::now_utc().format(&Rfc3339)
    }
}

struct JsonFormatter {
    file_label: &'static str,
    timer: Rfc3339Timer,
}

impl JsonFormatter {
    fn new(file_label: &'static str) -> Self {
        Self {
            file_label,
            timer: Rfc3339Timer,
        }
    }
}

impl<S, N> FormatEvent<S, N> for JsonFormatter
where
    S: tracing::Subscriber + for<'span> LookupSpan<'span>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        writer: &mut Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let mut visitor = JsonVisitor::default();
        event.record(&mut visitor);
        let mut map = visitor.finish();

        if let Some(message) = map.remove("message") {
            map.insert("msg".to_string(), message);
        }

        map.insert(
            "lvl".to_string(),
            serde_json::Value::String(event.metadata().level().to_string()),
        );
        map.insert(
            "file".to_string(),
            serde_json::Value::String(self.file_label.to_string()),
        );
        map.insert(
            "time".to_string(),
            serde_json::Value::String(self.timer.now().map_err(|_| std::fmt::Error)?),
        );

        writeln!(writer, "{}", serde_json::Value::Object(map))
    }
}

#[derive(Default)]
struct JsonVisitor {
    map: serde_json::Map<String, serde_json::Value>,
}

impl JsonVisitor {
    fn finish(self) -> serde_json::Map<String, serde_json::Value> {
        self.map
    }
}

impl<'a> Visit for JsonVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.map.insert(
            field.name().to_string(),
            serde_json::Value::String(format!("{:?}", value)),
        );
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.map.insert(
            field.name().to_string(),
            serde_json::Value::String(value.to_string()),
        );
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.map.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.map.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.map
            .insert(field.name().to_string(), serde_json::Value::Bool(value));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        if let Some(number) = serde_json::Number::from_f64(value) {
            self.map
                .insert(field.name().to_string(), serde_json::Value::Number(number));
        }
    }
}

struct AppWriter;

enum Stream {
    Stdout(io::Stdout),
    Stderr(io::Stderr),
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Stream::Stdout(stdout) => stdout.write(buf),
            Stream::Stderr(stderr) => stderr.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Stream::Stdout(stdout) => stdout.flush(),
            Stream::Stderr(stderr) => stderr.flush(),
        }
    }
}

impl<'a> tracing_subscriber::fmt::writer::MakeWriter<'a> for AppWriter {
    type Writer = Stream;

    fn make_writer(&'a self) -> Self::Writer {
        Stream::Stdout(io::stdout())
    }

    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        if *meta.level() >= tracing::Level::WARN {
            Stream::Stderr(io::stderr())
        } else {
            Stream::Stdout(io::stdout())
        }
    }
}

struct AccessWriter {
    file: Arc<std::sync::Mutex<std::fs::File>>,
}

impl AccessWriter {
    fn new(path: &str) -> io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            file: Arc::new(std::sync::Mutex::new(file)),
        })
    }
}

#[derive(Clone)]
struct FileWriter {
    file: Arc<std::sync::Mutex<std::fs::File>>,
}

impl Write for FileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = self.file.lock().expect("poisoned access log lock");
        guard.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut guard = self.file.lock().expect("poisoned access log lock");
        guard.flush()
    }
}

impl<'a> tracing_subscriber::fmt::writer::MakeWriter<'a> for AccessWriter {
    type Writer = FileWriter;

    fn make_writer(&'a self) -> Self::Writer {
        FileWriter {
            file: Arc::clone(&self.file),
        }
    }
}
