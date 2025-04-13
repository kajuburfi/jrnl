use crate::get_default_path;
use crate::utils::funcs::*;
use chrono::{DateTime, Datelike, Local, NaiveDate, format::ParseErrorKind};
use colored::Colorize;
use comfy_table::{
    ContentArrangement, Table,
    modifiers::UTF8_ROUND_CORNERS,
    presets::{NOTHING, UTF8_FULL},
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, ErrorKind, Write},
    process,
};
use stringmetrics::levenshtein;

#[path = "funcs.rs"]
pub mod funcs;

/// Sets the Config
#[derive(Deserialize, Debug)]
pub struct Config {
    pub add_weekday: bool,
    pub add_food_column: bool,
    pub editor: String,
    pub pager: String,
    pub max_rows: u32,
    pub add_timestamp: bool,
    pub when_pager: String,
    pub default_path: String,
}

/// Returns all headings(`# <stuff>`) and their corresponding line numbers
/// as a tuple: (headings, corresponding_line_no)
pub fn get_headings(filename: &str) -> (Vec<String>, Vec<u32>) {
    let file_result: Result<File, std::io::Error> = File::open(filename);
    let file: File = match file_result {
        Ok(file) => file,
        Err(e) => match e.kind() {
            // If the file is not found, say that it doesn't exist, instead of panicking.
            ErrorKind::NotFound => {
                eprintln!("{}", format!("Entry does not exist for {}", filename).red());
                process::exit(1);
            }
            other => panic!("Error opening file: {other}"),
        },
    };
    let reader: BufReader<File> = BufReader::new(file); // Reader to read by lines

    let mut headings: Vec<String> = Vec::new();
    let mut corresponding_line_no: Vec<u32> = Vec::new();
    let mut line_number = 0; // Initial value

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
        if cur_line.starts_with("# ") {
            // All entries start as `# YYYY-MM-DD`
            headings.push(cur_line.clone());
            corresponding_line_no.push(line_number);
        }
    }
    (headings, corresponding_line_no)
}

/// Adds certain information to the file.
///
/// To be used when opening the file for a new entry.
/// Adds different stuff depending on the configuration
pub fn add_info_to_file(filename: &str, date: String) -> std::io::Result<()> {
    // Convert string date to NaiveDate to get the weekday
    let date_naive = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
    let weekday = date_naive.weekday().to_string().to_uppercase();
    let timestamp = Local::now().format("%H:%M:%S").to_string();

    let mut file_data: File = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename)
        .unwrap();
    // We don't need the line numbers
    let (headings, _) = get_headings(filename);

    let config = read_config().0;
    let mut input_str = String::new();
    if config.add_weekday {
        input_str.push_str(&format!("\n### {}", &weekday));
    }
    if config.add_timestamp && config.add_weekday {
        input_str.push_str(&format!(" ({})", &timestamp));
    } else if config.add_timestamp && !config.add_weekday {
        input_str.push_str(&format!("\n### ({})", &timestamp));
    }
    input_str.push_str(&format!("\n# {}", &date));
    if config.add_food_column {
        input_str.push_str("\n- [food] | | | ");
    }

    // If `headings` doesn't contain today's date, then append it to the file.
    if !headings.contains(&format!("# {}", &date)) {
        let write_result = file_data.write(input_str.as_bytes());
        match write_result {
            Ok(_) => (),
            Err(e) => panic!("An error: {}", e),
        }
    }
    Ok(())
}

/// Get a given date's entry
pub fn get_entry(date: NaiveDate) -> String {
    // Get the filename(pre-defined format) from the NaiveDate
    let filename: String = format!(
        "{}/jrnl_folder/{}/{}_{}.md",
        get_default_path(),
        date.year(),
        date.year(),
        correct_month_nums(date.month())
    );
    let entry_date: String = date.format("%Y-%m-%d").to_string();
    let weekday = date.weekday().to_string();
    let mut entry: String = String::new();

    // Using file_result and file for easier error processing
    let file_result: Result<File, std::io::Error> = File::open(filename);
    let file: File = match file_result {
        Ok(file) => file,
        Err(e) => match e.kind() {
            // If the file is not found, say that it doesn't exist, instead of panicking.
            ErrorKind::NotFound => {
                entry.push_str(&format!(
                    "{}",
                    format!("Entry does not exist for {}", entry_date).red()
                ));
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
            // formatting with \n so that the entries can be printed(to STDOUT) correctly
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
            if read_config().0.add_weekday {
                entry.push_str(&format!(
                    "{} ({})\n",
                    cur_line.replace("#", "").trim().bold().yellow().underline(),
                    weekday.to_uppercase().purple()
                ));
            } else {
                entry.push_str(&format!(
                    "{}\n",
                    cur_line.replace("#", "").trim().bold().yellow().underline(),
                ));
            }
        }
        if reached_date_yet && !finished_entry && !(cur_line == "") {
            // Color the tags
            if cur_line.contains("[") {
                // Split the current line into parts, one or more of which contain a tag
                let parts: Vec<&str> = cur_line.split_inclusive(&['[', ']'][..]).collect();
                for part in parts.clone() {
                    if part.contains("]") {
                        // Every tag part contains this character
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
        entry.push_str(&format!(
            "{}",
            format!("Entry does not exist for {}", entry_date).red()
        ));
    }
    entry
}

/// Returns the tags found from a file(a month)
/// Useful in --gen-report
pub fn get_tags_from_file(filename: &str) -> Vec<String> {
    let file_result: Result<File, std::io::Error> = File::open(filename);
    let file: File = match file_result {
        Ok(file) => file,
        Err(e) => match e.kind() {
            // If the file is not found, say that it doesn't exist, instead of panicking.
            ErrorKind::NotFound => {
                eprintln!("Entry does not exist for {}", filename);
                return vec![];
            }
            other => panic!("Error opening file: {other}"),
        },
    };

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
            // Similar setup as in `get_entry()`, to collect the tags
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
/// Provides the date of the tag as well
/// Returns (date, entry)
/// By default, search for *tags*
pub fn search_for_stuff(
    word: &str,
    date: NaiveDate,
    search: bool,
    approx: bool,
) -> (Vec<String>, Vec<String>) {
    let filename: String = format!(
        "{}/jrnl_folder/{}/{}_{}.md",
        get_default_path(),
        date.year(),
        date.year(),
        correct_month_nums(date.month())
    );
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
        // If starting with a weekday, skip the line
        if cur_line.clone().starts_with("### ") {
            continue;
        }

        if reached_date_yet && cur_line.starts_with("# ") {
            reached_date_yet = false;
        }
        if cur_line.starts_with("# ") {
            reached_date_yet = true;
            // Get the date of the following tags
            entry_date_title = cur_line.clone()[1..].trim().to_string();
        }

        if cur_line.contains(&format!("[{}]", word)) && !search {
            let line_to_push = cur_line
                .replace(&format!("[{}]", word), &format!("[{}]", word.cyan()))
                .replace("- ", "")
                .trim()
                .to_string();
            tagged_entries.push(line_to_push);
            tagged_entry_dates.push(entry_date_title.clone());
        }

        if cur_line.contains(&format!("[{}]", word)) && search {
            println!("No matches for '{}' found in April, 2025", word.purple());
            println!("{}:", "Help".green().bold());
            println!(
                "There exists a {} with a similar name: {}",
                "tag".underline().red(),
                word.bright_yellow().bold()
            );
            println!("Perhaps you meant to get the tag?");
            process::exit(1);
        }

        // Searching within words or across words
        if cur_line.clone().contains(word)
            && search
            && (cur_line
                .clone()
                .chars()
                .nth(cur_line.clone().rfind(word).unwrap_or(0) - 1)
                .unwrap_or(' ')
                .is_alphanumeric()
                || cur_line
                    .clone()
                    .chars()
                    .nth(cur_line.clone().rfind(word).unwrap_or(0) + word.chars().count())
                    .unwrap_or(' ')
                    .is_alphanumeric())
        {
            let line_to_push = cur_line
                .replace(word, &format!("{}", word.purple()))
                .replacen("- ", "", 1) // Only replace first `-`
                .trim()
                .to_string();
            tagged_entries.push(line_to_push);
            tagged_entry_dates.push(entry_date_title.clone());
        }
        let words: Vec<&str> = cur_line
            .split(&[' ', '(', ')', ',', '.', ';', '-', '|', '/'][..])
            .collect();
        let mut line_over = false;
        for thing in words {
            if !approx {
                if (thing.to_lowercase() == word.to_lowercase()
                    || thing.to_lowercase() == word.to_lowercase() + "ed"
                    || thing.to_lowercase() == word.to_lowercase() + "d"
                    || thing.to_lowercase() == word.to_lowercase() + "es"
                    || thing.to_lowercase() == word.to_lowercase() + "'s"
                    || thing.to_lowercase() == word.to_lowercase() + "s")
                    && search
                    && !line_over
                {
                    line_over = true;
                    let line_to_push = cur_line
                        .replace(thing, &format!("{}", thing.purple()))
                        .replace("- ", "")
                        .trim()
                        .to_string();
                    tagged_entries.push(line_to_push);
                    tagged_entry_dates.push(entry_date_title.clone());
                }
            } else {
                if (thing.to_lowercase() == word.to_lowercase()
                    || thing.to_lowercase() == word.to_lowercase() + "ed"
                    || thing.to_lowercase() == word.to_lowercase() + "d"
                    || thing.to_lowercase() == word.to_lowercase() + "es"
                    || thing.to_lowercase() == word.to_lowercase() + "'s"
                    || thing.to_lowercase() == word.to_lowercase() + "s")
                    || (levenshtein(thing.to_lowercase().as_str(), word.to_lowercase().as_str())
                        <= 1)
                        && search
                        && !line_over
                {
                    line_over = true;
                    let line_to_push = cur_line
                        .replace(thing, &format!("{}", thing.purple()))
                        .replace("- ", "")
                        .trim()
                        .to_string();
                    tagged_entries.push(line_to_push);
                    tagged_entry_dates.push(entry_date_title.clone());
                }
            }
        }
    }
    (tagged_entry_dates, tagged_entries)
}

/// Returns NaiveDate when provided with a string
pub fn parse_entry_args(args: &str) -> NaiveDate {
    // Using Result<T> to handle errors nicely
    let entry_date_result = NaiveDate::parse_from_str(args, "%Y-%m-%d");
    let entry_date = match entry_date_result {
        Ok(entry_date) => entry_date,
        Err(e) => match e.kind() {
            // ErrorKinds from chrono
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
                // Inquires date when wrong format of date
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

/// Handles the processing of tags and search
pub fn handle_tags(
    args_tag: &str,
    args_tag_year: i32,
    args_tag_month: u32,
    year_provided: bool,
    month_provided: bool,
    search: bool,
    approx: bool,
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
    let mut tags_food: Vec<Vec<String>> = Vec::new();
    if year_provided && !month_provided {
        // Loop over all possible files in the given year to find all tags in the year
        let dir_name = format!("{}/jrnl_folder/{}/", get_default_path(), args_tag_year);
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
            let date = NaiveDate::from_ymd_opt(
                parts[parts.len() - 3].parse().unwrap(),
                parts[parts.len() - 2].parse().unwrap(),
                1,
            )
            .unwrap_or(today.date_naive());
            let tags_from_file = search_for_stuff(args_tag, date, search, approx);
            tags_date.extend(tags_from_file.0);
            tags_val.extend(tags_from_file.1);
        }
    } else {
        let tags_temp = search_for_stuff(args_tag, given_date, search, approx);
        tags_date.extend(tags_temp.0);
        if args_tag == "food" {
            for item in tags_temp.1.clone().iter() {
                let part_old = item.replace("[\u{1b}[36mfood\u{1b}[0m]", "");
                let parts: Vec<String> =
                    part_old.trim().split('|').map(|s| s.to_string()).collect();
                tags_food.push(parts);
            }
        } else {
            tags_val.extend(tags_temp.1);
        }
    }
    let tags = (tags_date.clone(), tags_val);

    let mut days_vec = Vec::new();
    let mut ydays_vec = Vec::new();
    let mut month_days: HashMap<u32, Vec<u32>> = HashMap::new();
    for date in tags_date.clone() {
        if date != "" {
            let day: u32 = date.split('-').collect::<Vec<&str>>()[2]
                .parse::<u32>()
                .unwrap_or(0);
            let month: u32 = date.split('-').collect::<Vec<&str>>()[1]
                .parse::<u32>()
                .unwrap_or(0);
            days_vec.push(day);
            if year_provided && !month_provided {
                ydays_vec.push((month, day));
            }
        }
    }

    let (_w, _h) = match term_size::dimensions() {
        Some((w, h)) => (w, h),
        None => (100, 30),
    };

    if tags.0.len() == 0 {
        // Other 3 cases included in the else clause.
        if year_provided && !month_provided {
            if search {
                println!(
                    "No matches for '{}' found in {}",
                    args_tag.purple(),
                    args_tag_year
                );
            } else {
                println!(
                    "No matches for the tag '{}' found in {}",
                    args_tag.cyan(),
                    args_tag_year
                );
            }
        } else {
            if search {
                println!(
                    "No matches for '{}' found in {}, {}",
                    args_tag.purple(),
                    month_no_to_name(args_tag_month),
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
        }
        process::exit(1);
    }

    if args_tag == "food" {
        if search {
            println!(
                "Searching for {}? That doesn't seem right... \nTry the tag instead: `-t food`",
                "food".purple()
            );
            process::exit(1);
        }
        if year_provided && !month_provided {
            println!(
                "{}",
                "This will fill up your terminal. Check month-wise instead.".red()
            );
            process::exit(1);
        }
        if read_config().0.when_pager == String::from("always") {
            make_pager(&format!("{}", make_food_table((tags_date, tags_food))));
        } else if read_config().0.when_pager == String::from("default") {
            if tags_food.len() >= 5 {
                make_pager(&format!("{}", make_food_table((tags_date, tags_food))));
            } else {
                println!("{}", make_food_table((tags_date, tags_food)));
            }
        } else {
            println!("{}", make_food_table((tags_date, tags_food)));
        }
    } else {
        if read_config().0.when_pager == String::from("always") {
            make_pager(&format!("{}", make_tags_table(tags)));
        } else if read_config().0.when_pager == String::from("default") {
            if tags.0.len() >= 5 {
                make_pager(&format!("{}", make_tags_table(tags)));
            } else {
                println!("{}", make_tags_table(tags));
            }
        } else {
            println!("{}", make_tags_table(tags));
        }
    }

    if year_provided && !month_provided {
        let mut calendar = Vec::new();
        for (month, day) in ydays_vec {
            month_days.entry(month).or_insert_with(Vec::new).push(day);
        }
        for (month, days) in month_days {
            calendar.push((month, print_calendar(args_tag_year, month, days)));
        }
        calendar.sort_by_key(|s| s.0);
        let mut cal = Table::new();
        // Width to get the number of columns to push to the table when
        // making the calendar grid
        let (w, _h) = match term_size::dimensions() {
            Some((w, h)) => (w, h),
            None => (100, 30),
        };
        let w = w / 23; // Each calendar takes 23 chars
        cal.set_content_arrangement(ContentArrangement::Dynamic)
            .load_preset(NOTHING);

        let mut cal_str: Vec<String> = Vec::new();
        for (_, item) in calendar {
            cal_str.push(item);
        }
        let chunks: Vec<&[String]> = cal_str.chunks(w).collect();
        for item in chunks {
            cal.add_row(item);
        }
        println!("{}", cal);
    } else {
        println!(
            "{}",
            print_calendar(args_tag_year, args_tag_month, days_vec)
        );
    }
}

/// Given an entry_date, if it exists, opens the editor at that position.
/// The editor is decided based upon the configuration
pub fn open_editor(entry_date: String) {
    let parts: Vec<&str> = entry_date.split('-').collect();
    let filename = format!(
        "{}/jrnl_folder/{}/{}_{}.md",
        get_default_path(),
        parts[0],
        parts[0],
        parts[1]
    );
    let made_new_file = !check_file_existed(&filename);

    if made_new_file {
        println!("Made a new file: {}", filename.underline());
    }

    let added_date_result = add_info_to_file(&filename, entry_date.clone());
    match added_date_result {
        Ok(_) => (),
        Err(e) => panic!("An error occured: {}", e),
    }
    let (headings, corr_line_no) = get_headings(&filename);

    for (i, head) in headings.iter().enumerate() {
        for (j, no) in corr_line_no.iter().enumerate() {
            if i == j && head[1..].trim() == entry_date {
                let cmd_arg: String;
                if read_config().0.editor == "hx" {
                    cmd_arg = format!("{}:{}", filename, no);
                } else {
                    cmd_arg = format!("{}", filename);
                }
                process::Command::new(read_config().0.editor)
                    .arg(cmd_arg)
                    .status()
                    .expect("Failed to execute process");
                return;
            }
        }
    }
}

/// Generates a report for a month.
///
/// ## Sample output:
/// Note that colors are present, but cannot be shown here.
///
/// ```text
/// Report for April, 2025
///
/// Number of entries this month: 12
///
///      April 2025
/// Mo Tu We Th Fr Sa Su
///     1  2  3  4  5  6
///  7  8  9 10 11 12 13
/// 14 15 16 17 18 19 20
/// 21 22 23 24 25 26 27
/// 28 29 30
///
/// Most used tags:
/// ╭───────────┬───────────╮
/// │ Tag       ┆ Frequency │
/// ╞═══════════╪═══════════╡
/// │ tag1      ┆ 12        │
/// ├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
/// │ tag2      ┆ 6         │
/// ├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
/// │ tag3      ┆ 2         │
/// ├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
/// │ tag4      ┆ 2         │
/// ├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
/// │ tag5      ┆ 2         │
/// ╰───────────┴───────────╯
/// ```
pub fn gen_report(year: i32, month: u32) {
    let filename = format!(
        "{}/jrnl_folder/{}/{}_{}.md",
        get_default_path(),
        year,
        year,
        correct_month_nums(month)
    );
    let (headings, no_of_entries) = get_headings(&filename);
    println!(
        "{}",
        format!("Report for {}, {}\n", month_no_to_name(month), year)
            .bold()
            .cyan()
            .underline()
    );

    println!(
        "{}",
        format!(
            "Number of entries this month: {}\n",
            no_of_entries.len().to_string().bold()
        )
        .yellow()
    );

    let mut dates: Vec<u32> = Vec::new();
    for heading in headings {
        let parts: Vec<&str> = heading.split('-').collect();
        dates.push(parts[2].to_string().parse::<u32>().unwrap_or(0));
    }
    println!("{}", print_calendar(year, month, dates));

    // Most used tags
    println!("{}", "Most used tags:".yellow().underline());
    let file_tags = get_tags_from_file(&filename);
    let mut freq_map: HashMap<String, u32> = HashMap::new();
    for item in file_tags.iter() {
        let count = freq_map.entry(item.to_string()).or_insert(0);
        *count += 1;
    }

    let mut sorted: Vec<_> = freq_map.iter().collect();
    sorted.sort_by_key(|a| a.1);

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Tag".green(), "Frequency".green()]);
    let mut no_of_rows = 0;
    for (key, value) in sorted.iter().rev() {
        table.add_row(vec![key.to_owned().to_owned(), value.to_string()]);
        no_of_rows += 1;
        if no_of_rows >= read_config().0.max_rows {
            break;
        }
    }
    println!("{}", table);
}

/// Generates a report for a year.
///
/// ## Sample output
/// Note that colors will be shown.
/// In the calendar, the dates when entries are present will be highlighted
///
/// ```text
/// Report for 2025
///
/// Number of entries this year: 17
/// March: 5
/// April: 12
/// May: 0
///
///       March 2025             April 2025             May 2025
///  Mo Tu We Th Fr Sa Su   Mo Tu We Th Fr Sa Su   Mo Tu We Th Fr Sa Su
///                  1  2       1  2  3  4  5  6             1  2  3  4
///   3  4  5  6  7  8  9    7  8  9 10 11 12 13    5  6  7  8  9 10 11
///  10 11 12 13 14 15 16   14 15 16 17 18 19 20   12 13 14 15 16 17 18
///  17 18 19 20 21 22 23   21 22 23 24 25 26 27   19 20 21 22 23 24 25
///  24 25 26 27 28 29 30   28 29 30               26 27 28 29 30 31
///  31
///
///
/// Most used tags:
/// ╭───────────┬───────────╮
/// │ Tag       ┆ Frequency │
/// ╞═══════════╪═══════════╡
/// │ food      ┆ 17        │
/// ├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
/// │ tag2      ┆ 10        │
/// ├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
/// │ tag3      ┆ 4         │
/// ├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
/// │ tag4      ┆ 3         │
/// ├╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
/// │ tag5      ┆ 2         │
/// ╰───────────┴───────────╯
/// ```
pub fn gen_report_year(year: i32) {
    // Loop over all possible files in the given year to find all tags in the year
    let dir_name = format!("{}/jrnl_folder/{}/", get_default_path(), year);
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

    let mut total_entries = 0;
    let mut final_tags: Vec<(String, u32)> = Vec::new();
    let mut monthly_entries: HashMap<u32, u32> = HashMap::new();
    let mut monthly_entries_vec: Vec<(u32, u32)> = Vec::new();
    let mut calendar: Vec<(u32, String)> = Vec::new();
    let mut freq_map: HashMap<String, u32> = HashMap::new();
    for path in paths {
        let name = path.unwrap().path().display().to_string();
        let parts: Vec<&str> = name.split(&['/', '_', '.'][..]).collect();
        let (year, month): (i32, u32) = (
            parts[parts.len() - 3].parse().unwrap(),
            parts[parts.len() - 2].parse().unwrap(),
        );

        // Stuff
        let filename = format!(
            "{}/jrnl_folder/{}/{}_{}.md",
            get_default_path(),
            year,
            year,
            correct_month_nums(month)
        );
        let (headings, curr_no_of_entries) = get_headings(&filename);
        total_entries += curr_no_of_entries.len();

        monthly_entries.insert(month, curr_no_of_entries.len().try_into().unwrap());
        let mut dates: Vec<u32> = Vec::new();
        for heading in headings {
            let parts: Vec<&str> = heading.split('-').collect();
            dates.push(parts[2].to_string().parse::<u32>().unwrap_or(0));
        }
        calendar.push((month, print_calendar(year, month, dates)));

        let file_tags = get_tags_from_file(&filename);
        for item in file_tags.iter() {
            let count = freq_map.entry(item.to_string()).or_insert(0);
            *count += 1;
        }

        monthly_entries_vec.extend(vec![(month, curr_no_of_entries.len().try_into().unwrap())]);
    }
    for (key, value) in freq_map {
        final_tags.extend(vec![(key, value)]);
    }
    final_tags.sort_by_key(|a| a.1);
    monthly_entries_vec.sort_by_key(|a| a.0);
    calendar.sort_by_key(|a| a.0);

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Tag".green(), "Frequency".green()]);
    let mut no_of_rows = 0;
    for (key, value) in final_tags.iter().rev() {
        table.add_row(vec![key.to_owned().to_owned(), value.to_string()]);
        no_of_rows += 1;
        if no_of_rows >= read_config().0.max_rows {
            break;
        }
    }

    let mut cal = Table::new();
    // Width to get the number of columns to push to the table when
    // making the calendar grid
    let (w, _h) = match term_size::dimensions() {
        Some((w, h)) => (w, h),
        None => (100, 30),
    };
    let w = w / 23; // Each calendar takes 23 chars
    cal.set_content_arrangement(ContentArrangement::Dynamic)
        .load_preset(NOTHING);

    let mut cal_str: Vec<String> = Vec::new();
    for (_, item) in calendar {
        cal_str.push(item);
    }
    let chunks: Vec<&[String]> = cal_str.chunks(w).collect();
    for item in chunks {
        cal.add_row(item);
    }

    println!(
        "{}",
        format!("Report for {}\n", year).bold().cyan().underline()
    );

    println!(
        "{}",
        format!(
            "Number of entries this year: {}",
            total_entries.to_string().bold()
        )
        .yellow()
        .underline()
    );

    for (k, v) in monthly_entries_vec {
        println!("{}: {}", month_no_to_name(k), v);
    }
    println!("\n{}", cal);

    println!();
    println!("{}", "Most used tags:".yellow().underline());
    println!("{}", table);
}
