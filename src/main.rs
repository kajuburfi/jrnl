/* TODO:
*/

//! This is one of my first rust projects, and is therefore not very idiomatic.
//! The code contains lots of repetition, and other generally *bad* coding practices.
//! Continue at your own risk.
//!
//! This is made mostly for my own reference later on, when I will eventually need it.
use crate::utils::*;
use chrono::{DateTime, Datelike, Local};
use clap::Parser;
use colored::Colorize;
use funcs::{check_file_existed, inquire_date, read_config};
use parse_datetime::parse_datetime_at_date as pdad;
use shellexpand::tilde;
use std::{path::Path, process};

mod funcs;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
/// A simple tool to maintain a journal completely in CLI.
/// Provides features like tags, to search by tag, generating reports
/// for a given month, pre-filling some data(date, weekday, etc)
struct Cli {
    /// Optionally provide the date as YYYY-MM-DD or type `c` to open the calendar picker
    /// , to open the relevant entry in the configured editor.
    #[arg(default_missing_value=Some("c"), num_args=0..=1, group="main")]
    open_entry: Option<String>,

    /// Provide the date as YYYY-MM-DD, to fetch the relevant entry.
    #[arg(short, long, default_missing_value=Some("a"), num_args=0..=1, group="main")]
    entry: Option<String>,

    /// List all occurances of a tag in a given file; Defaults to current month's file.
    #[arg(short, long, groups = ["main", "yearmonth"])]
    tag: Option<String>,

    /// Search for a given string in a file; Defaults to current month's file.
    #[arg(short, long, groups = ["main", "searching", "yearmonth"])]
    search: Option<String>,

    /// Provide a path to search for the directory `jrnl`.
    #[arg(short, long, default_missing_value=Some("."), num_args=0..=1)]
    path: Option<String>,

    /// Generate a report about a given month's file; Defaults to current month's file.
    #[arg(long, groups = ["main", "yearmonth"])]
    gen_report: bool,

    /// Opens the configuration file: ~/.config/jrnl/config.toml
    #[arg(long, group = "main")]
    open_config: bool,

    /// Print the current configuration
    #[arg(long, group = "main")]
    print_config: bool,

    /// Opens a file, in `jrnl_folder`, with any name, just to add some notes.
    /// You can also use `--open e` to open `events.md` since it is a default file.
    #[arg(long, group = "main")]
    open: Option<String>,

    /// Provide a year(YYYY) to search for a tag in, or to generate a report
    #[arg(short, long, requires = "yearmonth", default_missing_value=Some("0"), num_args=0..=1)]
    year: Option<i32>,

    /// Provide the month(MM) to search for the tag in, or to generate a report
    #[arg(short, long, requires = "yearmonth", default_missing_value=Some("0"), num_args=0..=1)]
    month: Option<u32>,

    /// Search for similar words as well, along with the current word.
    #[arg(short, long, requires = "searching", default_missing_value=Some("0"), num_args=0..=1)]
    approx: Option<u32>,
}

fn main() {
    // First check if config is right
    if !Path::new(&tilde("~/.config/jrnl/config.toml").into_owned()).exists() {
        println!(
            "{}: No configuration file found. Continuing with default config.\n{}: Make a config file at `~/.config/jrnl/config.toml`.",
            "WARNING".yellow().bold(),
            "HELP".green().bold()
        );
    }
    if read_config().1 != String::new() {
        println!(
            "{}",
            format!("Configuration Error: {}", read_config().1.bold()).red()
        );
        println!("{}", "Help: ".bold().green());
        println!("Note that all fields must be present in the toml file.");
        println!("Continuing with default configuration.");
    }

    let today: DateTime<Local> = Local::now(); //Get `now` time

    // Some variables to figure out whether both month and year were
    // provided or not; to check looping over a year
    let mut month_provided: bool = true;
    let mut year_provided: bool = true;

    let args = Cli::parse(); // Get args

    // Some match statements to figure out the Option<T>
    // Use `.as_deref()` to convert Option<String> to Option<&str>
    let args_entry = match args.entry.as_deref() {
        None => "",
        // Use inquire if no input for `-e`
        Some("a") => &inquire_date().format("%Y-%m-%d").to_string(),
        Some(entry) => &entry,
    };
    let args_open_entry = match args.open_entry.as_deref() {
        None => &today.format("%Y-%m-%d").to_string(),
        Some("c") => &inquire_date().format("%Y-%m-%d").to_string(),
        // Some cases to check to be able to use human relative time(yesterday, last week, etc)
        Some(a) => {
            let parts: Vec<&str> = a.split('-').collect();
            if a.len() == 10 && parts[0].len() == 4 && parts[1].len() == 2 && parts[2].len() == 2 {
                a
            } else {
                let pdad_result = pdad(today, a);
                match pdad_result {
                    Ok(value) => &value.naive_utc().format("%Y-%m-%d").to_string(),
                    Err(e) => {
                        println!("{}\nError: {}\n", "Couldn't understand your input".red(), e);
                        &inquire_date().format("%Y-%m-%d").to_string()
                    }
                }
            }
        }
    };
    let args_tag = match args.tag.as_deref() {
        None => "",
        Some(a) => &a,
    };
    let args_search = match args.search.as_deref() {
        None => "",
        Some(a) => &a,
    };
    let args_tag_year = match args.year {
        None => {
            year_provided = false;
            today.year()
        }
        // If we just pass `-y` with no <year> provided, we take the current year,
        // but we have provided the year, so year_provided=true.
        Some(0) => today.year(),
        Some(year) => year,
    };
    let args_tag_month = match args.month {
        None => {
            month_provided = false;
            today.month()
        }
        Some(0) => today.month(),
        Some(month) => month,
    };
    let args_approx: u32 = match args.approx {
        None => 0,
        Some(0) => read_config().0.approx_variation,
        Some(num) => num,
    };
    let args_open = match args.open.as_deref() {
        None => "",
        Some("e") => "events.md",
        Some(entry) => &entry,
    };

    // Handle arguments - not very efficiently or idiomatically
    if args_tag != "" {
        handle_tags(
            args_tag,
            args_tag_year,
            args_tag_month,
            year_provided,
            month_provided,
            false,
            args_approx,
        );
    }
    if args_search != "" {
        handle_tags(
            args_search,
            args_tag_year,
            args_tag_month,
            year_provided,
            month_provided,
            true,
            args_approx,
        );
    }

    if args_entry != "" {
        let entry = get_entry(parse_entry_args(args_entry));
        println!("{}", entry);
    }

    if args_open_entry != ""
        && args_tag == ""
        && args_search == ""
        && args_entry == ""
        && args_open == ""
        && !args.open_config
        && !args.print_config
        && !args.gen_report
    {
        let entry_date_naive = parse_entry_args(args_open_entry);
        let entry_date = entry_date_naive.format("%Y-%m-%d").to_string();
        if get_entry(entry_date_naive)
            == format!(
                "{}",
                format!("Entry does not exist for {}", entry_date).red()
            )
        {
            println!(
                "{}{}",
                "Entry info added for ".yellow(),
                entry_date.yellow().bold()
            );
        }
        open_editor(entry_date);
    }

    if args_open != "" {
        let has_file_existed = check_file_existed(&tilde(
            &format!("{}/jrnl_folder/{}", get_default_path(), args_open).to_owned(),
        ));
        if !has_file_existed {
            println!("Made new file: {}", args_open);
        }
        process::Command::new(read_config().0.editor)
            .arg(format!("{}/jrnl_folder/{}", get_default_path(), args_open))
            .status()
            .expect("Failed to execute process");
    }

    if args.gen_report {
        if year_provided && !month_provided {
            gen_report_year(args_tag_year);
        } else {
            gen_report(args_tag_year, args_tag_month);
        }
    }

    if args.open_config {
        if !check_file_existed(&tilde("~/.config/jrnl/config.toml").into_owned()) {
            println!("Made config file: ~/.config/jrnl/config.toml");
        }
        process::Command::new(read_config().0.editor)
            .arg(tilde("~/.config/jrnl/config.toml").into_owned())
            .status()
            .expect("Failed to execute process");
    }

    if args.print_config {
        println!("{}", read_config().0);
    }
}

/// Returns the path required in the current calling of the program.
/// If a `--path` flag is passed, it takes the value of that, else
/// it searches for the `default_path` in the config file.
pub fn get_default_path() -> String {
    match Cli::parse().path.as_deref() {
        None => tilde(&read_config().0.default_path).into_owned(),
        Some(a) => a.to_string(),
    }
}
