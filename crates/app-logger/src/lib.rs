use std::{env, fs, path::PathBuf};

pub use log::{debug, error, info, trace, warn, LevelFilter};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Logger, Root},
    filter::threshold::ThresholdFilter,
};
use sanitize_filename::sanitize_with_options;

#[derive(Debug, Clone, Copy, Default)]
pub struct LoggerConfigBuilder<'a> {
    config: LoggerConfig<'a>,
}

impl<'a> LoggerConfigBuilder<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn name_suffix(mut self, name_suffix: &'a str) -> Self {
        self.config.name_suffix = Some(name_suffix);
        self
    }

    #[must_use]
    pub fn program_name(mut self, program_name: &'a str) -> Self {
        self.config.program_name = Some(program_name);
        self
    }

    #[must_use]
    pub fn file_log_level(mut self, log_level: LevelFilter) -> Self {
        self.config.file_log_level = Some(log_level);
        self
    }

    #[must_use]
    pub fn stdout_log_level(mut self, log_level: LevelFilter) -> Self {
        self.config.stdout_log_level = Some(log_level);
        self
    }

    #[must_use]
    pub fn build(self) -> LoggerConfig<'a> {
        self.config
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LoggerConfig<'a> {
    pub(crate) name_suffix: Option<&'a str>,
    pub(crate) program_name: Option<&'a str>,
    pub(crate) file_log_level: Option<LevelFilter>,
    pub(crate) stdout_log_level: Option<LevelFilter>,
}

impl<'a> From<LoggerConfigBuilder<'a>> for LoggerConfig<'a> {
    fn from(builder: LoggerConfigBuilder<'a>) -> Self {
        builder.build()
    }
}

impl LoggerConfig<'_> {
    #[must_use]
    pub fn builder() -> LoggerConfigBuilder<'static> {
        LoggerConfigBuilder::new()
    }
}

pub fn init<'a, T: Into<LoggerConfig<'a>>>(cfg: T) -> anyhow::Result<log4rs::Handle> {
    let cfg: LoggerConfig = cfg.into();

    let tmp_file = get_tmp_file(&cfg);
    fs::create_dir_all(tmp_file.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "Failed to get parent directory of log file path: {:?}",
            &tmp_file
        )
    })?)?;

    let config = Config::builder();
    let config = {
        let log = FileAppender::builder().build(&tmp_file)?;
        let log_level = cfg.file_log_level.unwrap_or(LevelFilter::Info);
        config.appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log_level)))
                .build("logfile", Box::new(log)),
        )
    };
    let config = {
        let stdout = ConsoleAppender::builder().target(Target::Stdout).build();
        let log_level = cfg.stdout_log_level.unwrap_or(LevelFilter::Trace);
        config.appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log_level)))
                .build("stdout", Box::new(stdout)),
        )
    };
    let config = config
        .logger(Logger::builder().build("mio", LevelFilter::Error))
        .logger(Logger::builder().build("async_io", LevelFilter::Error))
        .logger(Logger::builder().build("polling", LevelFilter::Error))
        .logger(Logger::builder().build("rustls", LevelFilter::Error))
        .logger(Logger::builder().build("want", LevelFilter::Error))
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stdout")
                .build(LevelFilter::Trace),
        )?;

    let handle = log4rs::init_config(config)?;

    debug!("Logging to {:?}", &tmp_file);

    Ok(handle)
}

fn get_tmp_file(config: &LoggerConfig) -> PathBuf {
    let program_name = if let Some(name) = config.program_name {
        name.to_string()
    } else {
        env!("CARGO_PKG_NAME").to_string()
    };

    let mut tmp_file = program_name;

    if let Some(suffix) = config.name_suffix {
        tmp_file = format!("{tmp_file}_{suffix}");
    }

    if cfg!(target_os = "windows") {
        tmp_file = format!("{tmp_file}.txt");
    }

    let tmp_file = sanitize_with_options(
        tmp_file,
        sanitize_filename::Options {
            truncate: true,
            replacement: "^",
            ..Default::default()
        },
    );

    env::temp_dir().join(tmp_file)
}
