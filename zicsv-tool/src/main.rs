#![forbid(unsafe_code)]
#![warn(unused_results)]

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

    #[structopt(help = "Path to input file")]
    input_path: String,
}

fn load_from_file<Path: AsRef<std::path::Path>>(path: Path) -> Result<List, failure::Error> {
    let mut reader = zicsv::Reader::from_file(path)?;
    let records: Result<Records, failure::Error> = reader.records().collect();
    Ok(List {
        updated: *reader.get_timestamp(),
        records: records?,
    })
}

fn conv_into_json(options: &Options) -> Result<(), failure::Error> {
    let list = load_from_file(&options.input_path)?;
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
    conv_into_json(&options)
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
