use chrono::{DateTime, Datelike, Local, Month, NaiveDate, format::ParseErrorKind};
use colored::Colorize;
use comfy_table::{ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use inquire::DateSelect;
use pager::Pager;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, ErrorKind, Write},
    path::Path,
    process,
};

#[derive(Deserialize)]
pub struct Config {
    add_weekday: bool,
    add_food_column: bool,
    editor: String,
    pager: String,
}

fn default_conf() -> Config {
    Config {
        add_weekday: true,
        add_food_column: true,
        editor: String::from("hx"),
        pager: String::from("less"),
    }
}

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
    // Syntax according to the docs
    let month = Month::try_from(u8::try_from(month_num).unwrap())
        .ok()
        .unwrap_or(Month::January);
    month.name().to_string()
}

/// Returns all headings(<h1>) and their corresponding line numbers
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

/// Checks if the file contains a given date as a h1 heading.
/// If not, it appends it to the file.
pub fn add_date_to_file(filename: &str, date: String) -> std::io::Result<()> {
    // Convert string date to NaiveDate to get the weekday
    let date_naive = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
    let weekday = date_naive.weekday().to_string();

    let mut file_data: File = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename)
        .unwrap();
    let (headings, _) = get_headings(filename);

    let config = read_config();
    let mut input_str = String::new();
    if config.add_weekday {
        input_str.push_str(&format!("\n### {}", &weekday));
    }
    input_str.push_str(&format!("\n# {}", &date));
    if config.add_food_column {
        input_str.push_str("\n- [food] | | | ");
    }

    // If `headings` doesn't contain today's date, then append it to the file.
    if !headings.contains(&format!("# {}", &date)) {
        file_data.write(input_str.as_bytes()).expect("Write failed");
    }
    Ok(())
}

/// Get a given date's entry
pub fn get_entry(date: NaiveDate) -> String {
    // Get the filename(pre-defined format) from the NaiveDate
    let filename: String = format!(
        "jrnl/{}/{}_{}.md",
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
            if read_config().add_weekday {
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
        entry.push_str(&format!("Entry does not exist for {}", entry_date));
    }
    entry
}

/// Returns the tags found from a file(a month)
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
pub fn search_for_tags(tag: &str, date: NaiveDate) -> (Vec<String>, Vec<String>) {
    let filename: String = format!(
        "jrnl/{}/{}_{}.md",
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

        if reached_date_yet && cur_line.starts_with("# ") {
            reached_date_yet = false;
        }
        if cur_line.starts_with("# ") {
            reached_date_yet = true;
            // Get the date of the following tags
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

/// Makes a food table to show food
pub fn make_food_table(dates_values: (Vec<String>, Vec<Vec<String>>)) -> Table {
    let (dates, values) = dates_values;
    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        // .set_width(100)
        // TODO: Fix weird arrangement when paging
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Date of Entry".green(),
            "Breakfast".green(),
            "Lunch".green(),
            "Dinner".green(),
            "Other".green(),
        ]);
    for (i, date) in dates.iter().enumerate() {
        for (j, value) in values.iter().enumerate() {
            if i == j {
                let mut temp: Vec<String> = Vec::new();
                temp.push(date.to_string());
                for item in value.iter() {
                    temp.push(item.to_string());
                }
                table.add_row(temp);
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
    Pager::with_default_pager(read_config().pager).setup();
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
    let mut tags_food: Vec<Vec<String>> = Vec::new();
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

    if args_tag == "food" {
        if tags_food.len() >= 5 {
            make_pager(&format!("{}", make_food_table((tags_date, tags_food))));
        } else {
            println!("{}", make_food_table((tags_date, tags_food)));
        }
    } else {
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
                let cmd_arg: String;
                if read_config().editor == "hx" {
                    cmd_arg = format!("{}:{}", filename, no);
                } else {
                    cmd_arg = format!("{}", filename);
                }
                process::Command::new(read_config().editor)
                    .arg(cmd_arg)
                    .status()
                    .expect("Failed to execute process");
                return;
            }
        }
    }
}

/// Generates a report for a month.
pub fn gen_report(year: i32, month: u32) {
    let filename = format!("jrnl/{}/{}_{}.md", year, year, correct_month_nums(month));
    let (_, no_of_entries) = get_headings(&filename);
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
    for (key, value) in sorted.iter().rev() {
        table.add_row(vec![key.to_owned().to_owned(), value.to_string()]);
    }
    println!("{}", table);
}

pub fn gen_report_year(year: i32) {
    // Loop over all possible files in the given year to find all tags in the year
    let dir_name = format!("jrnl/{}/", year);
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
    let mut monthly_entries: HashMap<String, u32> = HashMap::new();
    for path in paths {
        let name = path.unwrap().path().display().to_string();
        let parts: Vec<&str> = name.split(&['/', '_', '.'][..]).collect();
        let (year, month): (i32, u32) = (parts[2].parse().unwrap(), parts[3].parse().unwrap());

        // Stuff
        let filename = format!("jrnl/{}/{}_{}.md", year, year, correct_month_nums(month));
        let (_, curr_no_of_entries) = get_headings(&filename);
        total_entries += curr_no_of_entries.len();

        monthly_entries.insert(
            month_no_to_name(month),
            curr_no_of_entries.len().try_into().unwrap(),
        );

        let file_tags = get_tags_from_file(&filename);
        let mut freq_map: HashMap<String, u32> = HashMap::new();
        for item in file_tags.iter() {
            let count = freq_map.entry(item.to_string()).or_insert(0);
            *count += 1;
        }

        for (key, value) in freq_map {
            final_tags.extend(vec![(key, value)]);
        }
    }
    final_tags.sort_by_key(|a| a.1);

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Tag".green(), "Frequency".green()]);
    for (key, value) in final_tags.iter().rev() {
        table.add_row(vec![key.to_owned().to_owned(), value.to_string()]);
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

    for (k, v) in monthly_entries {
        println!("{} : {}", k, v);
    }

    println!("");
    println!("{}", "Most used tags:".yellow().underline());
    println!("{}", table);
}

pub fn read_config() -> Config {
    let contents_result = fs::read_to_string("jrnl/config.toml");
    let mut config: Config = default_conf();
    let mut set_default = false;
    let contents = match contents_result {
        Ok(data) => data,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                set_default = true;
                "".to_string()
            }
            e => panic!("An error: {}", e),
        },
    };
    if !set_default {
        config = toml::from_str(&contents).unwrap();
    }
    config
}
