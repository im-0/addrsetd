#![forbid(unsafe_code)]
#![warn(unused_results)]

#![cfg_attr(feature = "cargo-clippy", warn(filter_map))]
#![cfg_attr(feature = "cargo-clippy", warn(if_not_else))]
#![cfg_attr(feature = "cargo-clippy", warn(mut_mut))]
#![cfg_attr(feature = "cargo-clippy", warn(non_ascii_literal))]
#![cfg_attr(feature = "cargo-clippy", warn(option_map_unwrap_or))]
#![cfg_attr(feature = "cargo-clippy", warn(option_map_unwrap_or_else))]
#![cfg_attr(feature = "cargo-clippy", warn(single_match_else))]
#![cfg_attr(feature = "cargo-clippy", warn(wrong_pub_self_convention))]
#![cfg_attr(feature = "cargo-clippy", warn(use_self))]
#![cfg_attr(feature = "cargo-clippy", warn(used_underscore_binding))]
#![cfg_attr(feature = "cargo-clippy", warn(print_stdout))]

extern crate failure;
extern crate isatty;

#[macro_use]
extern crate log;
extern crate log4rs;

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

#[derive(StructOpt, Debug)]
struct Options {}

fn init_basic_logger() -> Result<log4rs::Handle, failure::Error> {
    let pattern = if isatty::stderr_isatty() {
        "{h({d} [{M}] {l})} {m}{n}"
    } else {
        "{d} [{M}] {l} {m}{n}"
    };
    let encoder = Box::new(log4rs::encode::pattern::PatternEncoder::new(pattern));

    let appender = Box::new(
        log4rs::append::console::ConsoleAppender::builder()
            .encoder(encoder)
            .target(log4rs::append::console::Target::Stderr)
            .build(),
    );

    let config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build(
            "appender",
            appender,
        ))
        .build(log4rs::config::Root::builder().appender("appender").build(
            log::LogLevelFilter::Trace,
        ))?;

    log4rs::init_config(config).map_err(|error| error.into())
}

fn real_main() -> Result<(), failure::Error> {
    use structopt::StructOpt;

    let _ = init_basic_logger().expect("Unable to initialize basic logger (stderr)");

    let _ = Options::from_args();

    info!(
        "{} version {} started",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    Ok(())
}

fn main() {
    let rc = real_main().map(|_| 0).unwrap_or_else(|error| {
        let error_backtrace = format!("{}", error.backtrace());
        let mut duplicate_error_backtrace = false;
        for cause in error.causes() {
            error!("{}", cause);
            let _ = cause.backtrace().map(|backtrace| {
                let backtrace = format!("{}", backtrace);
                if !backtrace.is_empty() {
                    if backtrace == error_backtrace {
                        duplicate_error_backtrace = true;
                    };

                    error!("Cause {}", backtrace);
                };
            });
        }

        if !duplicate_error_backtrace && !error_backtrace.is_empty() {
            error!("Error {}", error_backtrace);
        };

        1
    });
    std::process::exit(rc)
}
