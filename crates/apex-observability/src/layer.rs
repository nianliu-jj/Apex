use std::fmt::Debug;
use std::path::{Path, PathBuf};

use chrono::Local;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

use crate::sink::FileSink;
use crate::task::current_task_context;

const EMPTY_CONTEXT: &str = "";

pub(crate) struct ApexFileLayer {
    component: String,
    run_id: String,
    _source_root: PathBuf,
    sink: FileSink,
}

impl ApexFileLayer {
    pub(crate) fn new(
        component: String,
        run_id: String,
        source_root: PathBuf,
        sink: FileSink,
    ) -> Self {
        Self {
            component,
            run_id,
            _source_root: source_root,
            sink,
        }
    }
}

impl<S> Layer<S> for ApexFileLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, context: Context<'_, S>) {
        let metadata = event.metadata();
        let mut visitor = PatternFieldVisitor::default();
        event.record(&mut visitor);

        let message = visitor.take_text("message").unwrap_or_default();
        let task = current_task_context();
        let trace_id = visitor
            .take_text_any(&["traceId", "trace_id"])
            .or_else(|| task.as_ref().map(|task| task.trace_id().to_owned()))
            .unwrap_or_else(|| EMPTY_CONTEXT.to_owned());
        visitor.discard_all(&["clientAddr", "client_addr"]);
        let exception = visitor.take_text_any(&["exception", "error", "error.message"]);
        let message_code = visitor.take_text("message_code");
        let thread = std::thread::current();
        let thread_name = thread
            .name()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("{:?}", thread.id()));
        let logger = if metadata.target().is_empty() {
            &self.component
        } else {
            metadata.target()
        };
        let source_file = metadata
            .file()
            .map(source_file_name)
            .unwrap_or_else(|| "unknown".to_owned());
        let source_line = metadata
            .line()
            .map_or_else(|| "?".to_owned(), |line| line.to_string());
        let spans = context
            .event_scope(event)
            .map(|scope| {
                scope
                    .from_root()
                    .map(|span| span.name())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let message = single_line(&message);
        let shows_diagnostic_context = matches!(*metadata.level(), Level::TRACE | Level::DEBUG);
        let rendered_message = if shows_diagnostic_context {
            let mut diagnostic_context = String::new();
            if let Some(code) = message_code {
                append_detail(&mut diagnostic_context, "messageCode", &code);
            }
            append_detail(&mut diagnostic_context, "runId", &self.run_id);
            if let Some(task) = task {
                append_detail(
                    &mut diagnostic_context,
                    "coroutine",
                    &format!("{}/{}", task.id(), task.name()),
                );
            }
            if !spans.is_empty() {
                append_detail(&mut diagnostic_context, "spans", &spans.join("/"));
            }
            for (name, value) in visitor.fields {
                append_detail(
                    &mut diagnostic_context,
                    &display_field_name(&name),
                    &value.as_text(),
                );
            }
            if let Some(exception) = exception {
                append_detail(&mut diagnostic_context, "exception", &exception);
            }
            match (diagnostic_context.is_empty(), message.is_empty()) {
                (true, _) => message,
                (false, true) => diagnostic_context,
                (false, false) => format!("{diagnostic_context} {message}"),
            }
        } else {
            message
        };

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let level = format!("{:>5}", metadata.level().as_str());
        let thread_column = fixed_width(&thread_name, 15, Alignment::Right);
        let logger_column = fixed_width(logger, 40, Alignment::Left);
        let line = format!(
            "{timestamp} {level} {} --[traceId: {}]-- [{thread_column}] {logger_column} {source_file}:{source_line} : {rendered_message}",
            std::process::id(),
            single_line(&trace_id),
        );

        self.sink.write_line(line);
    }
}

#[derive(Debug)]
enum FieldValue {
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    Text(String),
}

impl FieldValue {
    fn as_text(&self) -> String {
        match self {
            Self::Bool(value) => value.to_string(),
            Self::I64(value) => value.to_string(),
            Self::U64(value) => value.to_string(),
            Self::F64(value) => value.to_string(),
            Self::Text(value) => value.clone(),
        }
    }
}

#[derive(Default)]
struct PatternFieldVisitor {
    fields: Vec<(String, FieldValue)>,
}

impl PatternFieldVisitor {
    fn insert(&mut self, field: &Field, value: FieldValue) {
        let value = if is_sensitive_field(field.name()) {
            FieldValue::Text("[REDACTED]".to_owned())
        } else {
            value
        };
        self.fields.push((field.name().to_owned(), value));
    }

    fn take(&mut self, name: &str) -> Option<FieldValue> {
        let position = self.fields.iter().position(|(field, _)| field == name)?;
        Some(self.fields.remove(position).1)
    }

    fn take_text(&mut self, name: &str) -> Option<String> {
        self.take(name).map(|value| value.as_text())
    }

    fn take_text_any(&mut self, names: &[&str]) -> Option<String> {
        names.iter().find_map(|name| self.take_text(name))
    }

    fn discard_all(&mut self, names: &[&str]) {
        self.fields
            .retain(|(field, _)| !names.contains(&field.as_str()));
    }
}

impl Visit for PatternFieldVisitor {
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.insert(field, FieldValue::Bool(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.insert(field, FieldValue::I64(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.insert(field, FieldValue::U64(value));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.insert(field, FieldValue::F64(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.insert(field, FieldValue::Text(value.to_owned()));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.insert(field, FieldValue::Text(value.to_string()));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.insert(field, FieldValue::Text(format!("{value:?}")));
    }
}

#[derive(Clone, Copy)]
enum Alignment {
    Left,
    Right,
}

fn fixed_width(value: &str, width: usize, alignment: Alignment) -> String {
    let characters = value.chars().collect::<Vec<_>>();
    let rendered = if characters.len() > width {
        characters[characters.len() - width..]
            .iter()
            .collect::<String>()
    } else {
        value.to_owned()
    };
    let padding = width.saturating_sub(rendered.chars().count());
    match alignment {
        Alignment::Left => format!("{rendered}{}", " ".repeat(padding)),
        Alignment::Right => format!("{}{rendered}", " ".repeat(padding)),
    }
}

fn append_detail(message: &mut String, name: &str, value: &str) {
    if !message.is_empty() {
        message.push(' ');
    }
    message.push('[');
    message.push_str(name);
    message.push_str(": ");
    push_single_line(message, value);
    message.push(']');
}

fn display_field_name(name: &str) -> String {
    let mut output = String::with_capacity(name.len());
    let mut uppercase_next = false;
    for character in name.chars() {
        if character == '_' {
            uppercase_next = true;
        } else if uppercase_next {
            output.extend(character.to_uppercase());
            uppercase_next = false;
        } else {
            output.push(character);
        }
    }
    output
}

fn single_line(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    push_single_line(&mut output, value);
    output
}

fn push_single_line(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(output, "\\u{:04x}", character as u32);
            }
            character => output.push(character),
        }
    }
}

fn source_file_name(declared_file: &str) -> String {
    Path::new(declared_file)
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| declared_file.to_owned(), ToOwned::to_owned)
}

fn is_sensitive_field(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    [
        "password",
        "passwd",
        "secret",
        "token",
        "api_key",
        "apikey",
        "authorization",
        "cookie",
        "private_key",
        "credential",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}
