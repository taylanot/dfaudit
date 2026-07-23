use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

pub fn init(verbose: u8, quiet: bool) {
    let level = if quiet {
        LevelFilter::Error
    } else {
        match verbose {
            0 => LevelFilter::Warn,

            1 => LevelFilter::Info,

            _ => LevelFilter::Debug,
        }
    };

    Builder::new()
        .filter_level(level)
        .format(|buf, record| {
            writeln!(buf, "[{}] {}", record.level(), record.args())
        })
        .init();
}
