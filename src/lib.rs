use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;
use std::{
    error::Error,
    fs,
};

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
                .takes_value(false)
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Case-insensitive")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("invert_match")
                .short("v")
                .long("invert-match")
                .help("Invert match")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("recursive")
                .short("r")
                .long("recursive")
                .help("Recursive search")
                .takes_value(false)
        )
        .get_matches();

    let pattern = matches.value_of("pattern").unwrap();
    Ok(Config {
        pattern: RegexBuilder::new(pattern)
            .case_insensitive(matches.is_present("insensitive"))
            .build()
            .map_err(|_| -> Box<dyn Error> {
                From::from(format!("Invalid pattern \"{}\"", pattern))
            })?,
        files: matches.values_of_lossy("files").unwrap(),
        recursive: matches.is_present("recursive"),
        count: matches.is_present("count"),
        invert_match: matches.is_present("invert_match"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("pattern \"{}\"", config.pattern);
    let entries = find_files(&config.files, config.recursive);
    for entry in entries {
        match entry {
            Err(e) => eprintln!("{}", e),
            Ok(filename) => println!("file \"{}\"", filename),
        }
    }
    Ok(())
}

fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    if recursive {
        paths.iter()
            .flat_map(|path| if path.as_str() == "-" {
                    vec![Ok("-".to_string())]
                } else {
                    WalkDir::new(path)
                        .into_iter()
                        .filter_map(|entry| entry
                            .map_err(|e| From::from(e))
                            .map(|entry| if entry.file_type().is_dir() {
                                None
                            } else {
                                Some(String::from(entry.path().to_string_lossy()))
                            })
                            .transpose()
                        )
                        .collect()
                }
            )
            .collect()
    } else {
        paths.iter()
            .map(|path| if path.as_str() != "-" && fs::metadata(path)?.is_dir() {
                Err(From::from(format!("{} is a directory", path)))
            } else {
                Ok(path.to_owned())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::find_files;
    use rand::{distributions::Alphanumeric, Rng};

    #[test]
    fn test_find_files() {
        // Verify that the function treats dash as a file
        let files = find_files(&["-".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "-");

        // Verify that the function finds a file known to exist
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res.iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(files, vec![
            "./tests/inputs/bustle.txt",
            "./tests/inputs/empty.txt",
            "./tests/inputs/fox.txt",
            "./tests/inputs/nobody.txt",
        ]);

        // Verify the function finds the file and recurses to find four files in the directory
        let res = find_files(&["./tests/cli.rs".to_string(), "./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res.iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 5);
        assert_eq!(files, vec![
            "./tests/cli.rs",
            "./tests/inputs/bustle.txt",
            "./tests/inputs/empty.txt",
            "./tests/inputs/fox.txt",
            "./tests/inputs/nobody.txt",
        ]);

        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        
        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }
}
