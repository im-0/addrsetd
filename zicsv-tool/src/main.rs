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

#[macro_use]
extern crate serde_derive;
extern crate serde;

extern crate serde_json;

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

extern crate zicsv;

type Records = std::collections::LinkedList<zicsv::Record>;

#[derive(Serialize)]
pub struct List {
    /// Date of last update of this list.
    pub updated: zicsv::DateTime,
    /// List of records.
    pub records: Records,
}

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(short = "P", long = "disable-pretty", help = "Disable pretty-printing")]
    disable_pretty: bool,

    #[structopt(short = "i", long = "input", help = "Read from file instead of stdin")]
    input_path: Option<String>,
}

fn create_reader(options: &Options) -> Result<Box<zicsv::GenericReader>, failure::Error> {
    Ok(if let Some(input_path) = options.input_path.as_ref() {
        Box::new(zicsv::Reader::from_file(input_path)?)
    } else {
        Box::new(zicsv::Reader::from_reader(std::io::stdin())?)
    })
}

fn load_records(mut reader: Box<zicsv::GenericReader>) -> Result<List, failure::Error> {
    let records: Result<Records, failure::Error> = reader.records_boxed().collect();
    Ok(List {
        updated: *reader.get_timestamp(),
        records: records?,
    })
}

fn conv_into_json(options: &Options, reader: Box<zicsv::GenericReader>) -> Result<(), failure::Error> {
    let list = load_records(reader)?;

    let json_str = if options.disable_pretty {
        serde_json::to_string(&list)?
    } else {
        serde_json::to_string_pretty(&list)?
    };

    println!("{}", json_str);

    Ok(())
}

fn real_main() -> Result<(), failure::Error> {
    use structopt::StructOpt;

    let options = Options::from_args();

    let reader = create_reader(&options)?;

    conv_into_json(&options, reader)
}

fn main() {
    let rc = real_main().map(|_| 0).unwrap_or_else(|error| {
        eprintln!("Error:");
        let mut maybe_cause = Some(error.cause());
        while let Some(cause) = maybe_cause {
            eprintln!("    {}", cause);
            let _ = cause.backtrace().map(|backtrace| {
                let backtrace = format!("{}", backtrace);
                if !backtrace.is_empty() {
                    eprintln!("        {}\n", backtrace);
                };
            });

            maybe_cause = cause.cause();
        }
        1
    });
    std::process::exit(rc)
}
