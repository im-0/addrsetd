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

extern crate failure;

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

#[derive(StructOpt, Debug)]
struct Options {}

fn real_main() -> Result<(), failure::Error> {
    use structopt::StructOpt;

    let _ = Options::from_args();

    Ok(())
}

fn main() {
    let rc = real_main().map(|_| 0).unwrap_or_else(|error| {
        eprintln!("Error:");
        for cause in error.causes() {
            eprintln!("    {}", cause);
            let _ = cause.backtrace().map(|backtrace| {
                let backtrace = format!("{}", backtrace);
                if !backtrace.is_empty() {
                    eprintln!("        {}\n", backtrace);
                };
            });
        }
        1
    });
    std::process::exit(rc)
}
