#![allow(deprecated)]
use std::{fmt, io, path::PathBuf};
use std::io::{Result, ErrorKind};
use std::{fs, path::Path};

use chrono;
use fern::{
    colors::{Color, ColoredLevelConfig},
    Output,
};
use log;

const SILENCED_CRATES: &[&str] = &[
    "",
];
const SLIGHTLY_SILENCED_CRATES: &[&str] = &[
    "",
];

const COLORS: ColoredLevelConfig = ColoredLevelConfig {
    error: Color::Red,
    warn: Color::Yellow,
    info: Color::Green,
    debug: Color::Blue,
    trace: Color::Magenta,
};

#[cfg(not(windows))]
const LINE_SEPARATOR: &str = "\n";

#[cfg(windows)]
const LINE_SEPARATOR: &str = "\r\n";

const DATE_TIME_FORMAT_STR: &str = "[%Y-%m-%d %H:%M:%S%.3f]";

#[derive(Default, Debug)]
struct Formatter {
    pub output_timestamp: bool,
    pub output_color: bool,
}

impl Formatter {
    fn get_timetsamp_fmt(&self) -> &str {
        if self.output_timestamp {
            DATE_TIME_FORMAT_STR
        } else {
            &""
        }
    }

    fn get_record_level(&self, level: log::Level) -> Box<dyn fmt::Display> {
        if self.output_color && cfg!(not(windows)) {
            Box::new(COLORS.color(level))
        } else {
            Box::new(level)
        }
    }

    pub fn output_msg(
        &self,
        out: fern::FormatCallback,
        message: &fmt::Arguments,
        record: &log::Record,
    ) {
        let message = escape_newlines(format!("{}", message));

        out.finish(format_args!(
            "{}[{}][{}] {}",
            chrono::Local::now().format(self.get_timetsamp_fmt()),
            record.target(),
            self.get_record_level(record.level()),
            message,
        ))
    }
}


fn one_level_quieter(level: log::LevelFilter) -> log::LevelFilter {
    use log::LevelFilter::*;
    match level {
        Off => Off,
        Error => Off,
        Warn => Error,
        Info => Warn,
        Debug => Info,
        Trace => Debug,
    }
}


#[cfg(not(windows))]
fn escape_newlines(text: String) -> String {
    text
}

#[cfg(windows)]
fn escape_newlines(text: String) -> String {
    text.replace("\n", LINE_SEPARATOR)
}

/// Create a new log file while backing up a previous version of it.
///
/// A new log file is created with the given file name, but if a file with that name already exists
/// it is backed up with the extension changed to `.old.log`.
pub fn rotate_log(file: &Path) -> Result<()> {
    let backup = file.with_extension("old.log");
    if let Err(error) = fs::rename(file, backup) {
        if error.kind() != io::ErrorKind::NotFound {
            log::warn!("Failed to rotate log file ({})", error);
        }
    }

    fs::File::create(file)?;
    Ok(())
}

pub fn init_logger(
    log_level: log::LevelFilter,
    log_file: Option<&PathBuf>,
    output_timestamp: bool,
) -> Result<()> {
    let mut top_dispatcher = fern::Dispatch::new().level(log_level);
    for silenced_crate in SILENCED_CRATES {
        top_dispatcher = top_dispatcher.level_for(*silenced_crate, log::LevelFilter::Warn);
    }
    for silenced_crate in SLIGHTLY_SILENCED_CRATES {
        top_dispatcher = top_dispatcher.level_for(*silenced_crate, one_level_quieter(log_level));
    }

    let stdout_formatter = Formatter {
        output_timestamp,
        output_color: true,
    };
    let stdout_dispatcher = fern::Dispatch::new()
        .format(move |out, message, record| stdout_formatter.output_msg(out, message, record))
        .chain(io::stdout());
    top_dispatcher = top_dispatcher.chain(stdout_dispatcher);

    if let Some(ref log_file) = log_file {
        rotate_log(log_file)?;
        let file_formatter = Formatter {
            output_timestamp: true,
            output_color: false,
        };
        let f = fern::log_file(log_file)?;
        let file_dispatcher = fern::Dispatch::new()
            .format(move |out, message, record| file_formatter.output_msg(out, message, record))
            .chain(Output::file(f, LINE_SEPARATOR));
        top_dispatcher = top_dispatcher.chain(file_dispatcher);
    }
    top_dispatcher.apply().map_err(|_|io::Error::new(ErrorKind::NotConnected, ""))?;
    Ok(())
}
