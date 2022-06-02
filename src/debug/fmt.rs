use ansi_term::{Colour, Style};
use palette::{FromColor, Hsv, Pixel, Srgb};
use std::{
    fmt::{self, Display, Write},
    sync::{Arc, RwLock},
};
use tracing::{
    field::{Field, Visit},
    span::Record,
    Event, Id, Level, Subscriber,
};
use tracing_subscriber::{
    field::{self, VisitOutput},
    fmt::{
        format::{self, Writer},
        FmtContext, FormatEvent, FormatFields, FormattedFields,
    },
    prelude::__tracing_subscriber_field_RecordFields as RecordFields,
    registry::{LookupSpan, SpanRef},
};

const TARGET_COLOR: Colour = Colour::Fixed(242);
const SPAN_SEPARATOR_COLOR: Colour = Colour::Fixed(242);

const PREFIX_SINGLE_LINE: &str = "▪";
const PREFIX_START_LINE: &str = "┏";
const PREFIX_MID_LINE: &str = "┃";
const PREFIX_END_LINE: &str = "┗";

/// This is definitely a dumb idea, using a mutex in a tracing method...but screw it!!!
#[derive(Clone, Default)]
pub struct PrettierFormatter {
    last_span_id: Arc<RwLock<Option<Id>>>,
}

impl<S, N> FormatEvent<S, N> for PrettierFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Format values from the event's's metadata:
        let md = event.metadata();

        // Format all the spans in the event's span context.
        if let Some(scope) = ctx.event_scope() {
            let spans: Vec<SpanRef<S>> = scope.from_root().collect();

            let last_span = spans.last();
            let last_record_in_same_span = last_span
                .map(|s| {
                    let l = self.last_span_id.read().unwrap();
                    *l == Some(s.id())
                })
                .unwrap_or(false);

            if !last_record_in_same_span {
                writeln!(writer)?;

                let mut first = true;
                for span in spans.iter() {
                    if !first {
                        write!(writer, "{}", SPAN_SEPARATOR_COLOR.paint(" > "))?;
                    } else {
                        first = false;
                    }

                    write_span_name(&mut writer, span.name())?;

                    // `FormattedFields` is a formatted representation of the span's
                    // fields, which is stored in its extensions by the `fmt` layer's
                    // `new_span` method. The fields will have been formatted
                    // by the same field formatter that's provided to the event
                    // formatter in the `FmtContext`.
                    let ext = span.extensions();
                    let fields = &ext
                        .get::<FormattedFields<N>>()
                        .expect("will never be `None`");

                    // Skip formatting the fields if the span had no fields.
                    if !fields.is_empty() {
                        write!(writer, "{{{}}}", fields)?;
                    }
                }

                writeln!(writer)?;
            }

            *self.last_span_id.write().unwrap() = last_span.map(|s| s.id());
        } else {
            *self.last_span_id.write().unwrap() = None;
        }

        let level = md.level();
        let level_style = get_level_style(level, /* dimmed */ false);
        let prefixes = get_prefixes(level);

        let mut message_visitor = MessageValueVisitor::new();
        event.record(&mut message_visitor);

        if message_visitor.field_count == 0 {
            write!(writer, "{}", prefixes.single)?;
        } else {
            write!(writer, "{}", prefixes.start)?;
        }

        if let Some(message) = message_visitor.value {
            write!(writer, "{}", level_style.paint(message.to_string()))?;
        } else {
            write!(writer, "{}", level_style.dimmed().paint("(no message)"))?;
        }
        write_target(&mut writer, md.target())?;
        writeln!(writer)?;

        // Write fields on the event
        self.format_fields(prefixes, writer.by_ref(), event)?;
        Ok(())
    }
}

fn write_span_name(writer: &mut format::Writer, name: &str) -> Result<(), fmt::Error> {
    write!(writer, "{}", Colour::White.bold().paint(name))?;
    Ok(())
}

fn write_target(writer: &mut format::Writer, target: &str) -> Result<(), fmt::Error> {
    let target = format!("[{}]", target.to_string().replace("main::", "m::"));
    write!(writer, "  {}", TARGET_COLOR.paint(target))?;
    Ok(())
}

fn get_prefixes(level: &Level) -> RecordPrefixes {
    let level_style = get_level_style(level, /* is_dimmed */ false);
    let dim_level_style = get_level_style(level, /* is_dimmed */ true);
    let level_string = format!("{}:", &level.to_string().to_ascii_lowercase());
    let level_string = format!("{:8}", &level_string);
    RecordPrefixes {
        single: format!(
            "{}{} ",
            level_style.paint(&level_string),
            dim_level_style.paint(PREFIX_SINGLE_LINE)
        ),
        start: format!(
            "{}{} ",
            level_style.paint(&level_string),
            dim_level_style.paint(PREFIX_START_LINE)
        ),
        mid: format!(
            "{}{} ",
            dim_level_style.paint(&level_string),
            dim_level_style.paint(PREFIX_MID_LINE)
        ),
        end: format!(
            "{}{} ",
            dim_level_style.paint(&level_string),
            dim_level_style.paint(PREFIX_END_LINE)
        ),
    }
}

// TODO: Extract all of these colors to constants
fn get_level_style(level: &Level, is_dimmed: bool) -> Style {
    match *level {
        Level::TRACE => Style::new().fg(if is_dimmed {
            hsv_to_term_colour(221.0, 231, 90)
        } else {
            hsv_to_term_colour(221.0, 231, 224)
        }),
        Level::DEBUG => Style::new().fg(if is_dimmed {
            hsv_to_term_colour(192.0, 226, 90)
        } else {
            hsv_to_term_colour(192.0, 226, 227)
        }),
        Level::INFO => Style::new().fg(if is_dimmed {
            hsv_to_term_colour(134.0, 226, 90)
        } else {
            hsv_to_term_colour(134.0, 226, 227)
        }),
        Level::WARN => Style::new().fg(if is_dimmed {
            hsv_to_term_colour(60.0, 242, 90)
        } else {
            hsv_to_term_colour(60.0, 242, 229)
        }),
        Level::ERROR => Style::new().fg(if is_dimmed {
            hsv_to_term_colour(358.0, 202, 90)
        } else {
            hsv_to_term_colour(358.0, 202, 220)
        }),
    }
}

impl<'writer> PrettierFormatter {
    fn format_fields<R: RecordFields>(
        &self,
        prefixes: RecordPrefixes,
        mut writer: Writer<'writer>,
        fields: R,
    ) -> fmt::Result {
        let mut buf = String::new();
        let mut v = PrettierVisitor::new(&mut buf);
        fields.record(&mut v);

        let writer = &mut writer.by_ref();
        let buf_clone = v.buf.clone();
        let lines: Vec<(usize, &str)> = buf_clone.lines().enumerate().collect();
        let count = lines.len();
        for (i, line) in lines.iter() {
            let is_last = i + 1 == count;
            if is_last {
                write!(writer, "{}", prefixes.end)?;
            } else {
                write!(writer, "{}", prefixes.mid)?;
            }
            writeln!(writer, "  {}", line)?;
        }

        v.finish()
    }
}

impl<'writer> FormatFields<'writer> for PrettierFormatter {
    fn format_fields<R: RecordFields>(
        &self,
        writer: Writer<'writer>,
        fields: R,
    ) -> fmt::Result {
        let mut buf = String::new();
        let mut writer = writer;
        let mut v = PrettierVisitor::new(&mut buf);
        fields.record(&mut v);
        let buf = v.buf.clone();
        writer.write_str(buf.as_str())?;
        v.finish()
    }

    fn add_fields(
        &self,
        current: &'writer mut FormattedFields<Self>,
        fields: &Record<'_>,
    ) -> fmt::Result {
        let writer = &mut current.as_writer();
        let mut buf = String::new();
        let mut v = PrettierVisitor::new(&mut buf);

        fields.record(&mut v);
        let buf = v.buf.clone();
        write!(writer, "{}", &buf)?;

        v.finish()
    }
}

#[derive(Debug, Default)]
struct RecordPrefixes {
    single: String,
    start: String,
    mid: String,
    end: String,
}

#[derive(Debug)]
pub struct PrettierVisitor<'a> {
    buf: &'a mut String,
    style: Style,
    result: fmt::Result,
}

impl<'a> PrettierVisitor<'a> {
    /// Returns a new default visitor that formats to the provided `writer`.
    ///
    /// # Arguments
    /// - `writer`: the writer to format to.
    /// - `is_empty`: whether or not any fields have been previously written to
    ///   that writer.
    pub fn new(buf: &'a mut String) -> Self {
        Self {
            buf,
            style: Style::default(),
            result: Ok(()),
        }
    }

    fn write_padded(&mut self, value: &impl fmt::Debug) {
        self.result = writeln!(self.buf, "{:?}", value);
    }
}

impl<'a> field::Visit for PrettierVisitor<'a> {
    fn record_str(&mut self, field: &Field, value: &str) {
        if self.result.is_ok() && field.name() != "message" {
            self.record_debug(field, &value)
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        if let Some(source) = value.source() {
            let bold = Colour::White.italic();
            self.record_debug(
                field,
                &format_args!(
                    "{}, {}{}.sources{}: {}",
                    value,
                    bold.prefix(),
                    field,
                    bold.infix(self.style),
                    ErrorSourceList(source),
                ),
            )
        } else {
            self.record_debug(field, &format_args!("{}", value))
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if self.result.is_err() {
            return;
        }

        let key_style = Colour::White.italic();
        match field.name() {
            "message" => {}
            // Skip fields that are actually log metadata that have already been handled
            #[cfg(feature = "tracing-log")]
            name if name.starts_with("log.") => self.result = Ok(()),
            name if name.starts_with("r#") => self.write_padded(&format_args!(
                "{}{}{}{}{:?}",
                key_style.prefix(),
                &name[2..],
                key_style.infix(self.style),
                SPAN_SEPARATOR_COLOR.paint("="),
                value
            )),
            name => self.write_padded(&format_args!(
                "{}{}{}{}{:?}",
                key_style.prefix(),
                name,
                key_style.infix(self.style),
                SPAN_SEPARATOR_COLOR.paint("="),
                value
            )),
        };
    }
}

impl<'a> VisitOutput<fmt::Result> for PrettierVisitor<'a> {
    fn finish(self) -> fmt::Result {
        self.result
    }
}

/// Renders an error into a list of sources, *including* the error
struct ErrorSourceList<'a>(&'a (dyn std::error::Error + 'static));

impl<'a> Display for ErrorSourceList<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        let mut curr = Some(self.0);
        while let Some(curr_err) = curr {
            list.entry(&format_args!("{}", curr_err));
            curr = curr_err.source();
        }
        list.finish()
    }
}

pub struct MessageValueVisitor {
    value: Option<String>,
    field_count: usize,
}
impl MessageValueVisitor {
    fn new() -> Self {
        Self {
            value: None,
            field_count: 0,
        }
    }
}
impl Visit for MessageValueVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if self.value.is_none() && field.name() == "message" {
            self.value = Some(format!("{:?}", value));
        } else {
            self.field_count += 1;
        }
    }
}

fn hsv_to_term_colour(hue_deg: f32, saturation: u8, value: u8) -> Colour {
    let rgb = Srgb::from_color(Hsv::new(
        hue_deg,
        saturation as f32 / 255.0,
        value as f32 / 255.0,
    ));
    let components: [u8; 3] = rgb.into_format().into_raw();
    Colour::RGB(components[0], components[1], components[2])
}
