// File to store small(?) functions used in utils
use chrono::{Month, NaiveDate};
use colored::Colorize;
use comfy_table::{ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use inquire::DateSelect;
use pager::Pager;
use std::{
    fs::{self, File},
    io::ErrorKind,
    path::Path,
};

use crate::utils::*;

/// Returns the default configuration to be used when no config file is found.
///
/// ## Example:
///
/// ```
/// assert_eq!(
///     default_conf(),
///     Config {
///         add_weekday: true,
///         add_food_column: false,
///         editor: String::from("nano"),
///         pager: String::from("less"),
///         max_rows: 5,
///         add_timestamp: false,
///         when_pager: "default".to_string(),
///         default_path: String::from("."),
///         approx_variation: 1,
///     }
/// );
/// ```
pub fn default_conf() -> Config {
    Config {
        add_weekday: true,
        add_food_column: false,
        editor: String::from("nano"),
        pager: String::from("less"),
        max_rows: 5,
        add_timestamp: false,
        when_pager: "default".to_string(),
        default_path: String::from("."),
        approx_variation: 1,
    }
}

/// Checks if the file exists, if not, it makes the file.
/// If the file previously existed, returns true
/// else false.
pub fn check_file_existed(filename: &str) -> bool {
    let path: &Path = Path::new(filename);

    if !path.exists() {
        let file_result = File::create_new(filename.to_string());
        match file_result {
            Ok(_) => false,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    eprintln!(
                        "{}",
                        format!(
                            "There doesn't seem to be a folder for {}. Please create it.",
                            filename
                        )
                        .red(),
                    );
                    process::exit(1);
                }
                err => panic!("An error: {}", err),
            },
        }
    } else {
        true
    }
}

/// Takes in a number, generally provided from some NaiveDate(converted),
/// and returns the string. Useful to get the correct file path.
///
/// ## Example:
/// ```
/// assert_eq!(correct_month_nums(3), String::from("03"));
/// assert_eq!(correct_month_nums(20), String::from("00"));
/// ```
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
///
/// ## Example:
/// ```
/// assert_eq!(month_no_to_name(2), String::from("February"));
/// assert_eq!(month_no_to_name(20), String::from("January"));
/// ```
pub fn month_no_to_name(month_num: u32) -> String {
    // Syntax according to the docs
    let month = Month::try_from(u8::try_from(month_num).unwrap())
        .ok()
        .unwrap_or(Month::January);
    month.name().to_string()
}

/// Makes a table to show the tags and related records
pub fn make_tags_table(dates_values: (Vec<String>, Vec<String>)) -> Table {
    let (dates, values) = dates_values;
    let mut table = Table::new();

    let (w, _h) = match term_size::dimensions() {
        Some((w, h)) => (w, h),
        None => (100, 30),
    };
    let w = w as f64 * (9.0 / 10.0);
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Date of Entry".green(), "Record".green()]);
    for (i, date) in dates.iter().enumerate().rev() {
        for (j, value) in values.iter().enumerate() {
            if i == j {
                table.add_row(vec![date, value]);
            }
        }
    }
    if table.width() >= Some(w.round() as u16) {
        table.set_width(w.round() as u16);
    }
    table
}

/// Makes a food table to show food
pub fn make_food_table(dates_values: (Vec<String>, Vec<Vec<String>>)) -> Table {
    let (dates, values) = dates_values;
    let mut table = Table::new();

    let (w, _h) = match term_size::dimensions() {
        Some((w, h)) => (w, h),
        None => (100, 30),
    };
    let w = w as f64 * (9.0 / 10.0);
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_width(w.round() as u16)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Date of Entry".green(),
            "Breakfast".green(),
            "Lunch".green(),
            "Dinner".green(),
            "Other".green(),
        ]);
    for (i, date) in dates.iter().enumerate().rev() {
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
        Err(e) => match e {
            inquire::InquireError::OperationCanceled => {
                println!("{}", "Cancelling...".red());
                process::exit(0);
            }
            _ => panic!("An error occured: {}", e),
        },
    };
    date
}

/// Makes a pager to pass some output
pub fn make_pager(output: &str) {
    Pager::with_default_pager(read_config().0.pager).setup();
    println!("{}", output);
}

/// Reads the config file and stores the result
pub fn read_config() -> (Config, String) {
    let contents_result =
        fs::read_to_string(shellexpand::tilde("~/.config/jrnl/config.toml").into_owned());
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
        let config_result = toml::from_str(&contents);
        config = match config_result {
            Ok(config) => config,
            Err(e) => {
                return (default_conf(), e.message().to_string());
            }
        }
    }
    (config, String::new())
}

/// Prints a calendar for the given month, and highlights
/// certain days with a green, bold modifier.
pub fn print_calendar(year: i32, month: u32, highlight_day: Vec<u32>) -> String {
    let mut output = String::new();

    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let last_day = first_day
        .with_day(1)
        .unwrap()
        .with_month(month + 1)
        .unwrap()
        .pred_opt()
        .unwrap();

    // Print the month and year
    output.push_str(&format!(
        "     {} {}\n",
        month_no_to_name(month).cyan().bold().underline(),
        year.to_string().cyan().bold().underline()
    ));
    output.push_str(&format!("{}", "Mo Tu We Th Fr Sa Su\n".bright_yellow()));

    // Print leading spaces for the first day of the month
    let first_weekday = first_day.weekday().num_days_from_monday(); // Change to start from Monday
    for _ in 0..first_weekday {
        output.push_str(&format!("   "));
    }
    // Print the days of the month
    for day in first_day.day()..=last_day.day() {
        if highlight_day.contains(&day) {
            output.push_str(&format!("{:>2} ", day.to_string().green().bold())); // Highlight the specified day 
        } else {
            output.push_str(&format!("{:>2} ", day));
        }
        if (first_weekday + day as u32) % 7 == 0 {
            output.push_str("\n");
        }
    }
    output.push_str("\n"); // New line at the end
    output
}
