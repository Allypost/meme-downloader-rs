use log::{debug, LevelFilter};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Logger, Root},
    filter::threshold::ThresholdFilter,
};
use sanitize_filename::sanitize_with_options;
use std::{env, fs, path::Path};

pub fn init<S: AsRef<str>>(download_url: S) {
    let program_name = get_program_name();
    let mut tmp_file = format!("{program_name}_{url}", url = download_url.as_ref());
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
    let tmp_file = env::temp_dir().join(tmp_file);
    fs::create_dir_all(tmp_file.parent().unwrap()).unwrap();

    let stdout = ConsoleAppender::builder().target(Target::Stdout).build();
    let log = FileAppender::builder().build(&tmp_file).unwrap();

    let config = Config::builder();
    let config = config.appender(
        Appender::builder()
            .filter(Box::new(ThresholdFilter::new(LevelFilter::Info)))
            .build("logfile", Box::new(log)),
    );
    let config = config.appender(
        Appender::builder()
            .filter(Box::new(ThresholdFilter::new(LevelFilter::Trace)))
            .build("stdout", Box::new(stdout)),
    );
    let config = config
        .logger(Logger::builder().build("mio", LevelFilter::Error))
        .logger(Logger::builder().build("async_io", LevelFilter::Error))
        .logger(Logger::builder().build("polling", LevelFilter::Error))
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stdout")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    debug!("Logging to {:?}", &tmp_file);
}

fn get_program_name() -> String {
    let args = env::args().collect::<Vec<String>>();
    let program_name = Path::new(args.first().unwrap())
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    program_name.to_string()
}
