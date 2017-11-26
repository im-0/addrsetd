#![forbid(unsafe_code)]
#![warn(unused_results)]

extern crate failure;
extern crate serde_json;

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

extern crate zicsv;

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(short = "P", long = "disable-pretty", help = "Disable pretty-printing")]
    disable_pretty: bool,

    #[structopt(help = "Path to input file")]
    input_path: String,
}

fn conv_into_json(options: &Options) -> Result<(), failure::Error> {
    let list = zicsv::List::load_from_file(&options.input_path)?;
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
        eprintln!("Error: {}", error);
        let backtrace = format!("{}", error.backtrace());
        if !backtrace.is_empty() {
            eprintln!("{}", backtrace);
        };
        1
    });
    std::process::exit(rc)
}
