use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;
use std::{
    error::Error,
    fs::{self, File},
    io::{self, BufRead, BufReader},
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
    let entries = find_files(&config.files, config.recursive);
    let is_single_file = entries.len() == 1;
    for entry in entries {
        match entry {
            Err(e) => eprintln!("{}", e),
            Ok(filename) => match open(&filename) {
                Err(e) => eprintln!("{}: {}", filename, e),
                Ok(file) => {
                    let matches = find_lines(
                        file, 
                        &config.pattern,
                        config.invert_match,
                    );
                    if matches.is_err() {
                        eprintln!("{}: {}", filename, matches.unwrap_err());
                        continue;
                    }
                    let matches = matches.unwrap();
                    if config.count {
                        if is_single_file || filename.as_str() == "-" {
                            println!("{}", matches.len());
                        } else {
                            println!("{}: {}", filename, matches.len());
                        }
                    } else {
                        for s in matches {
                            let s = s.trim();
                            if is_single_file || filename.as_str() == "-" {
                                println!("{}", s);
                            } else {
                                println!("{}: {}", filename, s);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
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

fn find_lines<T: BufRead>(mut file: T, pattern: &Regex, invert_match: bool) -> MyResult<Vec<String>> {
    let mut lines = vec![];
    let mut buf = String::new();
    loop {
        let bytes_read = file.read_line(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
        if pattern.is_match(&buf) == !invert_match {
            lines.push(buf.clone());
        }
        buf.clear();
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::{find_files, find_lines};
    use rand::{distributions::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;

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

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";

        // The pattern _or_ should match the one line "Lorem"
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);

        // When inverted, the function should match the other two lines
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // This regex will be case-insensitive
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();
        
        // The two lines "Lorem" and "DOLOR" should match
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // When inverted, the one remaining line should match
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
}
