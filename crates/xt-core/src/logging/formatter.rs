use crate::logging::timer::ZonedTimer;
use std::fmt;
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields};
use tracing_subscriber::registry::LookupSpan;

#[derive(Clone, Copy)]
pub(super) struct LogFormat(pub(super) ZonedTimer);

impl<S, N> FormatEvent<S, N> for LogFormat
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        context: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        self.0.format_time(&mut writer)?;
        let metadata = event.metadata();
        let thread = std::thread::current();
        let fallback;
        let name = match thread.name() {
            Some(name) => name,
            None => {
                fallback = format!("{:?}", thread.id());
                fallback
                    .strip_prefix("ThreadId(")
                    .and_then(|value| value.strip_suffix(')'))
                    .unwrap_or(&fallback)
            }
        };
        write!(
            writer,
            " {:>5} {} [{}] ",
            metadata.level().as_str(),
            std::process::id(),
            name
        )?;
        if let Some(scope) = context.event_scope() {
            for span in scope.from_root() {
                write!(writer, "{}", span.name())?;
                let extensions = span.extensions();
                if let Some(fields) = extensions.get::<FormattedFields<N>>()
                    && !fields.is_empty()
                {
                    write!(writer, "{{{fields}}}")?;
                }
                write!(writer, ": ")?;
            }
        }
        write!(writer, "{}: ", metadata.target())?;
        context
            .field_format()
            .format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}
