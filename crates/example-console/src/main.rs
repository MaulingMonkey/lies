use std::env;
use std::process::exit;

fn main() {
    let mut args = env::args();
    let _exe = args.next();

    let command = args.next();
    let command = if let Some(c) = command { c } else {
        print_usage();
        exit(0);
    };

    match command.as_str() {
        "help" | "--help"       => { print_usage();     exit(0); },
        "version" | "--version" => { print_version();   exit(0); },
        "about"                 => { print_about();   exit(0); },
        "add"                   => { add(args);         exit(0); },
        other => {
            eprintln!("Unrecognized subcommand: {}", other);
            print_usage();
            exit(1);
        },
    }
}

fn print_version() {
    println!("example-console v{}", env!("CARGO_PKG_VERSION"));
    println!();
}

fn print_usage() {
    println!("{}", USAGE.trim());
    println!();
}

fn print_about() {
    print_version();
    println!("{}", lies::licenses_ansi!());
}

fn add(args: env::Args) {
    let mut sum = 0.0;
    for value in args {
        sum += value.parse::<f64>().expect("\"example-console add\" expected only numerical arguments");
    }
    println!("{}", sum);
}

const USAGE : &'static str = "
Usage:  example-console [subcommand]

Subcommands:
    help        Print this help/usage information
    version     Print the current version of example-console
    about       Print the license information of example-console
    add         Add a series of numbers together and print the result
";
