use chrono::{DateTime, Datelike, Local, Month, NaiveDate, format::ParseErrorKind};
use colored::Colorize;
use comfy_table::{ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use inquire::DateSelect;
use pager::Pager;
use std::{
    fs,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, ErrorKind, Write},
    path::Path,
    process,
};

/// Checks if the file exists, if not, it makes the file.
/// If the file previously existed, returns true
/// else false.
pub fn check_file_existed(filename: &str) -> bool {
    let path: &Path = Path::new(filename);

    if !path.exists() {
        File::create_new(filename.to_string()).expect("Could not make new file.");
        false
    } else {
        true
    }
}

/// Takes in a number, generally provided from some NaiveDate(converted),
/// and returns the string. Useful to get the correct file path.
pub fn correct_month_nums(num: u32) -> String {
    match num {
        1 => "01".to_string(),
        2 => "02".to_string(),
        3 => "03".to_string(),
        4 => "04".to_string(),
        5 => "05".to_string(),
        6 => "06".to_string(),
        7 => "07".to_string(),
        8 => "08".to_string(),
        9 => "09".to_string(),
        _ => "00".to_string(),
    }
}

/// Takes a month number(generally from NaiveDate) and returns
/// the name of the month. Used for printing purposes.
pub fn month_no_to_name(month_num: u32) -> String {
    let month = Month::try_from(u8::try_from(month_num).unwrap())
        .ok()
        .unwrap_or(Month::January);
    month.name().to_string()
}

/// Returns all headings(<h1>) and their corresponding line numbers
pub fn get_headings(filename: &str) -> (Vec<String>, Vec<u32>) {
    let file: File = File::open(filename).expect("Some error");
    let reader: BufReader<File> = BufReader::new(file);
    let mut headings: Vec<String> = Vec::new();
    let mut corresponding_line_no: Vec<u32> = Vec::new();
    let mut line_number = 0;

    // Reads all lines and appends the line if it contains "#" in it.
    for line in reader.lines() {
        line_number += 1;
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
            corresponding_line_no.push(line_number);
        }
    }
    (headings, corresponding_line_no)
}

/// Checks if the file contains a given date as a h1 heading.
/// If not, it appends it to the file.
pub fn add_date_to_file(filename: &str, date: String) -> std::io::Result<()> {
    let mut file_data: File = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename)
        .unwrap();
    let (headings, _) = get_headings(filename);

    // If `headings` doesn't contain today's date, then append it to the file.
    if !headings.contains(&format!("# {}", &date)) {
        file_data
            .write(format!("\n# {}\n", &date).as_bytes())
            .expect("Write failed");
    }
    Ok(())
}

/// Get a given date's entry
pub fn get_entry(date: NaiveDate) -> String {
    let filename: String = format!(
        "jrnl/{}/{}_{}.md",
        date.year(),
        date.year(),
        correct_month_nums(date.month())
    );
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
            entry.push_str(&format!(
                "{}\n",
                cur_line.replace("#", "").trim().bold().yellow().underline()
            ));
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
pub fn get_tags_from_file(filename: &str) -> Vec<String> {
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

/// Returns all records with the tag in the given file.
/// Provides the date of the tag as well.
pub fn search_for_tags(tag: &str, date: NaiveDate) -> (Vec<String>, Vec<String>) {
    let filename: String = format!(
        "jrnl/{}/{}_{}.md",
        date.year(),
        date.year(),
        correct_month_nums(date.month())
    );
    // let file: File = File::open(filename).expect("Some error");
    let file_result: Result<File, std::io::Error> = File::open(&filename);
    let file: File = match file_result {
        Ok(file) => file,
        Err(e) => match e.kind() {
            // If the file is not found, say that it doesn't exist, instead of panicking.
            ErrorKind::NotFound => {
                eprintln!(
                    "{}",
                    format!("There are no entries in {}", &filename.underline()).red()
                );
                process::exit(1);
            }
            other => panic!("Error opening file: {other}"),
        },
    };
    let reader: BufReader<File> = BufReader::new(file);

    let mut tagged_entries = Vec::new();
    let mut tagged_entry_dates = Vec::new();

    let mut reached_date_yet: bool = false;
    let mut entry_date_title = String::new();

    for line in reader.lines() {
        // Making another variable to manipulate it.
        let cur_line: String = match line {
            Ok(line) => line.clone(),
            Err(e) => {
                eprintln!("{}", e.to_string().red());
                "".to_string()
            }
        };

        if reached_date_yet && cur_line.starts_with("#") {
            reached_date_yet = false;
        }
        if cur_line.starts_with("#") {
            reached_date_yet = true;
            entry_date_title = cur_line.clone()[1..].trim().to_string();
        }

        if cur_line.contains(&format!("[{}]", tag)) {
            let line_to_push = cur_line
                .replace(&format!("[{}]", tag), &format!("[{}]", tag.cyan()))
                .replace("-", "")
                .trim()
                .to_string();
            tagged_entries.push(line_to_push);
            tagged_entry_dates.push(entry_date_title.clone());
        }
    }
    (tagged_entry_dates, tagged_entries)
}

/// Returns NaiveDate when provided with a string
pub fn parse_entry_args(args: &str) -> NaiveDate {
    let entry_date_result = NaiveDate::parse_from_str(args, "%Y-%m-%d");
    let entry_date = match entry_date_result {
        Ok(entry_date) => entry_date,
        Err(e) => match e.kind() {
            ParseErrorKind::OutOfRange
            | ParseErrorKind::Impossible
            | ParseErrorKind::NotEnough
            | ParseErrorKind::Invalid
            | ParseErrorKind::TooShort
            | ParseErrorKind::TooLong
            | ParseErrorKind::BadFormat => {
                println!(
                    "{}",
                    "Please provide date in appropriate format: YYYY-MM-DD".red()
                );
                inquire_date()
            }
            e => {
                println!("{}", format!("An error has occured: {:?}", e).red());
                process::exit(1);
            }
        },
    };
    entry_date
}

/// Makes a table to show the tags and related records
pub fn make_tags_table(dates_values: (Vec<String>, Vec<String>)) -> Table {
    let (dates, values) = dates_values;
    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Date of Entry".green(), "Record".green()]);
    for (i, date) in dates.iter().enumerate() {
        for (j, value) in values.iter().enumerate() {
            if i == j {
                table.add_row(vec![date, value]);
            }
        }
    }
    table
}

/// Inquires the date in case not provided.
pub fn inquire_date() -> NaiveDate {
    let date_prompt = DateSelect::new("Select a date to search for its entry:").prompt();
    let date = match date_prompt {
        Ok(date) => date,
        Err(e) => panic!("An error occured: {}", e),
    };
    date
}

/// Makes a pager to pass some output
pub fn make_pager(output: &str) {
    Pager::with_default_pager("bat").setup();
    println!("{}", output);
}

/// Handles the processing of tags
pub fn handle_tags(
    args_tag: &str,
    args_tag_year: i32,
    args_tag_month: u32,
    year_provided: bool,
    month_provided: bool,
) {
    let today: DateTime<Local> = Local::now(); //Get `now` time
    let given_date_result = NaiveDate::from_ymd_opt(args_tag_year, args_tag_month, 1);
    let given_date = match given_date_result {
        Some(m) => m,
        None => {
            eprintln!(
                "{}",
                "Invalid year/month provided. Defaulting to today.".red()
            );
            today.date_naive()
        }
    };

    let mut tags_date: Vec<String> = Vec::new();
    let mut tags_val: Vec<String> = Vec::new();
    if year_provided && !month_provided {
        // Loop over all possible files in the given year to find all tags in the year
        let dir_name = format!("jrnl/{}/", args_tag_year);
        let paths_result = fs::read_dir(&dir_name);
        let paths = match paths_result {
            Ok(p) => p,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    eprintln!(
                        "{}",
                        format!("There are no entries in {}", &dir_name.underline()).red()
                    );
                    process::exit(1);
                }
                other => panic!("Error: {}", other),
            },
        };
        for path in paths {
            let name = path.unwrap().path().display().to_string();
            let parts: Vec<&str> = name.split(&['/', '_', '.'][..]).collect();
            let date =
                NaiveDate::from_ymd_opt(parts[2].parse().unwrap(), parts[3].parse().unwrap(), 1)
                    .unwrap_or(today.date_naive());
            let tags_from_file = search_for_tags(args_tag, date);
            tags_date.extend(tags_from_file.0);
            tags_val.extend(tags_from_file.1);
        }
    } else {
        let tags_temp = search_for_tags(args_tag, given_date);
        tags_date.extend(tags_temp.0);
        tags_val.extend(tags_temp.1);
    }
    let tags = (tags_date, tags_val);

    if tags.0.len() == 0 {
        // Other 3 cases included in the else clause.
        if year_provided && !month_provided {
            println!(
                "No matches for the tag '{}' found in {}",
                args_tag.cyan(),
                args_tag_year
            );
        } else {
            println!(
                "No matches for the tag '{}' found in {}, {}",
                args_tag.cyan(),
                month_no_to_name(args_tag_month),
                args_tag_year
            );
        }
    } else if tags.0.len() <= 10 {
        println!("{}", make_tags_table(tags));
    } else {
        make_pager(&format!("{}", make_tags_table(tags)));
    }
}

/// Given an entry_date, if it exists, opens the helix editor at that position
pub fn open_editor(entry_date: String) {
    let parts: Vec<&str> = entry_date.split('-').collect();
    let filename = format!("jrnl/{}/{}_{}.md", parts[0], parts[0], parts[1]);
    let made_new_file = !check_file_existed(&filename);

    if made_new_file {
        println!("Made a new file: {}", filename.underline());
    }

    let added_date_result = add_date_to_file(&filename, entry_date.clone());
    match added_date_result {
        Ok(_) => (),
        Err(e) => panic!("An error occured: {}", e),
    }
    let (headings, corr_line_no) = get_headings(&filename);

    for (i, head) in headings.iter().enumerate() {
        for (j, no) in corr_line_no.iter().enumerate() {
            if i == j && head[1..].trim() == entry_date {
                process::Command::new("hx")
                    .arg(format!("{}:{}", filename, no))
                    .status()
                    .expect("Failed to execute process");
                return;
            }
        }
    }
}
