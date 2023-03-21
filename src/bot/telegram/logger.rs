use std::{env, fs, path::Path};

use log::{debug, LevelFilter};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    filter::{threshold::ThresholdFilter, Filter, Response},
};

#[derive(Debug)]
struct ItemFilter {}
impl ItemFilter {
    fn new() -> Self {
        Self {}
    }
}
impl Filter for ItemFilter {
    fn filter(&self, record: &log::Record) -> Response {
        let module = record.module_path();
        if let Some(module) = module {
            if module.starts_with("rustls::") || module == "want" {
                return Response::Reject;
            }
        }
        Response::Neutral
    }
}

pub fn init() {
    let program_name = get_program_name();
    let mut tmp_file = format!("{program_name}_telegram_bot");
    if cfg!(target_os = "windows") {
        tmp_file = format!("{tmp_file}.txt");
    }
    let mut tmp_file = env::temp_dir().join(tmp_file);
    tmp_file.shrink_to_fit();
    fs::create_dir_all(tmp_file.parent().unwrap()).unwrap();

    let stdout = ConsoleAppender::builder().target(Target::Stdout).build();
    let log = FileAppender::builder().build(&tmp_file).unwrap();

    let stdout_threshold = if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };

    let config = Config::builder();
    let config = config.appender(
        Appender::builder()
            .filter(Box::new(ThresholdFilter::new(LevelFilter::Debug)))
            .build("logfile", Box::new(log)),
    );
    let config = config.appender(
        Appender::builder()
            .filter(Box::new(ThresholdFilter::new(stdout_threshold)))
            .filter(Box::new(ItemFilter::new()))
            .build("stdout", Box::new(stdout)),
    );
    let config = config
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
