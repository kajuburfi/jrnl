/* TODO:
 * [X] Add relevant normal and doc comments.
 * [X] Add error handling when wrong config is provided
 * [X] Fix --open-entry when provided with old/new dates with no entries.
 * [X] Check all cases of usage of expect/unwrap.
 * [X] When using `-t tag -y`; print calendars chronoligically
 * [X] Fix searching with `food`
 * [ ] Add approximate string matching using stringmetrics
 */
// Author: Tejas Gudgunti
use crate::funcs::inquire_date;
use crate::utils::*;
use chrono::{DateTime, Datelike, Local};
use clap::Parser;
use colored::Colorize;
use std::process;
use utils::funcs::{check_file_existed, read_config};

mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
/// A simple tool to maintain a journal completely in CLI.
/// Provides features like tags, to search by tag, generating reports
/// for a given month, pre-filling some data(date, weekday, etc)
struct Cli {
    /// Provide the date as YYYY-MM-DD, to fetch the relevant entry.
    ///
    /// When provided with a date, it fetches the relevant entry, and prints it to the terminal(STDOUT).
    /// If provided with no date, just a `-e` or `--entry` tag, a date will be asked through an interactive calendar,
    /// which has the current date as the default value.
    /// If provided with an incorrect or non-existant date, the date will be asked by
    /// the same procedure. Currently does NOT support paging.
    #[arg(short, long, default_missing_value=Some("a"), num_args=0..=1, group="main")]
    entry: Option<String>,

    /// Provide the date as YYYY-MM-DD, to open the relevant entry in helix-editor.
    ///
    /// A feature to be able to edit any given entry at any time, provided it exists. If it doesn't exist,
    /// the program will exit. Opens the exact line number if the editor is Helix-editor. Editor can be
    /// configured in the `jrnl/config.toml` as `editor=<executable name>`
    #[arg(short, long, default_missing_value=Some("a"), num_args=0..=1, group="main")]
    open_entry: Option<String>,

    /// List all occurances of a tag in a given file; Defaults to current month's file.
    ///
    /// Makes a table that shows all the places where the tag has been used, and the date of its usage,
    /// so you can `--entry` or `--open-entry` for that date.
    /// If a month `--month` or `-m` has been passed along with this, the tag will be searched for that month.
    /// If a year `--year` or `-y` is passed along with this, but without a month, all files within that year
    /// will be recursively searched for the tag specified.
    /// If both year and month are provided, that exact file will be searched.
    /// Supports a pager (configurable, defaults to `less`) when the number of rows in the table exceeds 10.
    /// A special tag is `[food]`, which when passed, makes a different table. Check the README
    #[arg(short, long, group = "main")]
    tag: Option<String>,

    /// Search for a given string in a file; Defaults to current month's file.
    ///
    /// Similar to `--tag`, this searches for the provided string. It is case-insensitive,
    /// and searches for all matches not in the designated tag format(`[tag]`). Shows all results
    /// with entry date as a table format similar to `--tag`. Only exact words are searched for.
    /// Words are highlighted. Can provide `--year` and `--month` tags along with this.
    #[arg(short, long, group = "main")]
    search: Option<String>,

    /// Provide a year(YYYY) to search for a tag in, or to generate a report
    ///
    /// Used in both `--tag` and `--gen-report`. Accepts a year YYYY to pass to either `--tag` or `--gen-report`.
    /// If any error of the year, it defaults to the current year.
    // Note that we always provide Some(&str) for the default_missing_value, it automatically
    // adjusts the value
    #[arg(short, long, requires = "main", default_missing_value=Some("0"), num_args=0..=1)]
    year: Option<i32>,

    /// Provide the month(MM) to search for the tag in, or to generate a report
    ///
    /// Used in both `--tag` and `--gen-report`. Accepts a month MM to pass to either `--tag` or `--gen-report`.
    /// If any error of the month, it defaults to the current month.
    #[arg(short, long, requires = "main", default_missing_value=Some("0"), num_args=0..=1)]
    month: Option<u32>,

    /// Generate a report about a given month's file; Defaults to current month's file.
    ///
    /// Generates a report, and prints it to STDOUT.
    /// Shows the number of entries in the month, and the most used tags along with their frequency in
    /// a table format.
    /// Can provide the month or year to generate the report of, using `--year`(`-y`) or `--month`(`-m`) flags.
    #[arg(long, group = "main")]
    gen_report: bool,

    /// Opens the respective configuration file: ./jrnl/config.toml
    #[arg(long, group = "main")]
    open_config: bool,
}

fn main() {
    // First check if config is right
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
        None => "",
        // Use inquire if no input for `-o`
        Some("a") => &inquire_date().format("%Y-%m-%d").to_string(),
        Some(a) => &a,
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

    // Handle arguments - not very efficiently or idiomatically
    if args_tag != "" {
        handle_tags(
            args_tag,
            args_tag_year,
            args_tag_month,
            year_provided,
            month_provided,
            false,
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
        );
    }

    if args_entry != "" {
        let entry = get_entry(parse_entry_args(args_entry));
        println!("{}", entry);
    }

    if args_open_entry != "" {
        let entry_date_naive = parse_entry_args(args_open_entry);
        let entry_date = entry_date_naive.format("%Y-%m-%d").to_string();
        if get_entry(entry_date_naive)
            == format!(
                "{}",
                format!("Entry does not exist for {}", entry_date).red()
            )
        {
            println!(
                "{}",
                format!("Entry does not exist for {}", entry_date).red()
            );
            process::exit(1);
        }
        open_editor(entry_date);
    }

    if args.gen_report {
        if year_provided && !month_provided {
            gen_report_year(args_tag_year);
        } else {
            gen_report(args_tag_year, args_tag_month);
        }
    }

    if args.open_config {
        if !check_file_existed("jrnl/config.toml") {
            println!("Made config file: jrnl/config.toml");
        }
        process::Command::new(read_config().0.editor)
            .arg("jrnl/config.toml")
            .status()
            .expect("Failed to execute process");
    }

    if args_tag == ""
        && args_entry == ""
        && args_open_entry == ""
        && !args.gen_report
        && !args.open_config
        && args_search == ""
    {
        let today_date = today.format("%Y-%m-%d").to_string();
        open_editor(today_date);
    }
}
