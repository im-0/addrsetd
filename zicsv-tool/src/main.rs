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

fn main() {
    use structopt::StructOpt;

    let options = Options::from_args();

    let list = zicsv::List::load_from_file(options.input_path);
    let json_str = if options.disable_pretty {
        serde_json::to_string(&list).unwrap()
    } else {
        serde_json::to_string_pretty(&list).unwrap()
    };
    println!("{}", json_str);
}
