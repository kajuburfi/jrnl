// Author: Tejas Gudgunti
use chrono::{DateTime, Local};
use chrono::{Datelike, TimeZone};
use colored::Colorize;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::path::Path;

fn main() {
    let today: DateTime<Local> = Local::now(); //Get `now` time
    let other_day: DateTime<Local> = Local.with_ymd_and_hms(2025, 3, 25, 5, 5, 5).unwrap();

    let filename: String = format!(
        "jrnl/{}/{}_{}.md",
        today.year(),
        today.year(),
        today.month()
    );
    check_file_exist(&filename);
    let _ = add_date_to_file(&filename, today);
    let entry = get_entry(other_day);
    println!("{}", entry);
    let tags = get_tags_from_file(&filename);
}

/// Checks if the file exists, if not, it makes the file.
fn check_file_exist(filename: &str) {
    let path: &Path = Path::new(filename);

    if !path.exists() {
        File::create_new(filename.to_string()).expect("Could not make new file.");
        println!("{}{:?}", "> Created file: ".green().bold(), path);
    }
}

/// Returns all headings(<h1>) as a vector
fn get_headings(filename: &str) -> Vec<String> {
    let file: File = File::open(filename).expect("Some error");
    let reader: BufReader<File> = BufReader::new(file);
    let mut headings: Vec<String> = Vec::new();

    // Reads all lines and appends the line if it contains "#" in it.
    for line in reader.lines() {
        // Making another variable to manipulate it.
        let cur_line: String = match line {
            Ok(line) => line.clone(),
            Err(e) => {
                eprintln!("{}", e.to_string().red());
                "".to_string()
            }
        };
        if cur_line.starts_with("#") {
            headings.push(cur_line.clone());
        }
    }
    headings
}

/// Checks if the file contains today's date as a h1 heading.
/// If not, it appends it to the file.
fn add_date_to_file(filename: &str, date: DateTime<Local>) -> std::io::Result<()> {
    let mut file_data: File = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename)
        .unwrap();
    let today_date: String = date.format("%Y-%m-%d").to_string();
    let headings = get_headings(filename);

    // If `headings` doesn't contain today's date, then append it to the file.
    if !headings.contains(&format!("# {}", &today_date)) {
        file_data
            .write(format!("\n# {}\n", &today_date).as_bytes())
            .expect("Write failed");
    }
    Ok(())
}

/// Get a given date's entry
fn get_entry(date: DateTime<Local>) -> String {
    let filename: String = format!("jrnl/{}/{}_{}.md", date.year(), date.year(), date.month());
    let entry_date: String = date.format("%Y-%m-%d").to_string();
    let mut entry: String = String::new();

    // Using file_result and file for easier error processing
    let file_result: Result<File, std::io::Error> = File::open(filename);
    let file: File = match file_result {
        Ok(file) => file,
        Err(e) => match e.kind() {
            // If the file is not found, say that it doesn't exist, instead of panicking.
            ErrorKind::NotFound => {
                entry.push_str(&format!("Entry does not exist for {}", entry_date));
                return entry;
            }
            other => panic!("Error opening file: {other}"),
        },
    };
    let reader: BufReader<File> = BufReader::new(file);

    // Couple variables to make sure we only get values within the entry
    let mut reached_date_yet: bool = false;
    let mut finished_entry: bool = false;

    for line in reader.lines() {
        let cur_line: String = match line {
            // formatting with \n so that the entries can be printed correctly
            Ok(line) => format!("{}\n", line.clone()),
            Err(e) => {
                eprintln!("{}", e);
                "".to_string()
            }
        };

        if reached_date_yet && cur_line.starts_with("#") {
            finished_entry = true;
        }
        if cur_line.contains(&entry_date) {
            reached_date_yet = true;
            entry.push_str(&format!("{}", cur_line.bold().yellow().underline()));
        }
        if reached_date_yet && !finished_entry && !(cur_line == "") {
            // Color the tags
            if cur_line.contains("[") {
                let parts: Vec<&str> = cur_line.split_inclusive(&['[', ']'][..]).collect();
                for part in parts.clone() {
                    if part.contains("]") {
                        entry.push_str(&format!("{}]", &part[..part.len() - 1].cyan()));
                    } else {
                        entry.push_str(part);
                    }
                }
            } else if !cur_line.trim().starts_with('#') {
                entry.push_str(&cur_line);
            }
        }
    }

    if entry == "" {
        entry.push_str(&format!("Entry does not exist for {}", entry_date));
    }
    entry
}

/// Returns the tags found from a file(a month)
fn get_tags_from_file(filename: &str) -> Vec<String> {
    let file: File = File::open(filename).expect("Some error");
    let reader: BufReader<File> = BufReader::new(file);
    let mut tags: Vec<String> = Vec::new();

    for line in reader.lines() {
        // Making another variable to manipulate it.
        let cur_line: String = match line {
            Ok(line) => line.clone(),
            Err(e) => {
                eprintln!("{}", e.to_string().red());
                "".to_string()
            }
        };

        if cur_line.contains("[") {
            let parts: Vec<&str> = cur_line.split_inclusive(&['[', ']'][..]).collect();
            for part in parts.clone() {
                if part.contains("]") {
                    tags.push((&part[..part.len() - 1]).to_string());
                }
            }
        }
    }
    tags
}
