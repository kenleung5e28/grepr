use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::{error::Error, fs};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("grepr")
        .version("0.1.0")
        .author("Ken C.Y. Leung <kenleung5e28@gmail.com>")
        .about("Rust grep")
        .arg(
            Arg::with_name("pattern")
                .value_name("PATTERN")
                .help("Search pattern")
                .takes_value(true)
                .required(true)
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value("-")
                .multiple(true)
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Count occurrences")
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Case-insensitive")
        )
        .arg(
            Arg::with_name("invert_match")
                .short("v")
                .long("invert-match")
                .help("Invert match")
        )
        .arg(
            Arg::with_name("recursive")
                .short("r")
                .long("recursive")
                .help("Recursive search")
        )
        .get_matches();

    Ok(Config {
        pattern: RegexBuilder::new(matches.value_of("pattern").unwrap())
            .case_insensitive(matches.is_present("insensitive")).build()?,
        files: matches.values_of_lossy("files").unwrap()
            .into_iter()
            .flat_map(|s| fs::read_dir(s)
                .filter_map(|entry| entry.ok())
                .map(|entry| String::from(entry.path().to_string_lossy()))
            ).collect(),
        recursive: matches.is_present("recursive"),
        count: matches.is_present("count"),
        invert_match: matches.is_present("invert_match"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}