// Author: Tejas Gudgunti
use crate::utils::*;
use chrono::{DateTime, Datelike, Local};
use clap::Parser;
use colored::Colorize;
use std::process;

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
    #[arg(short, long, default_missing_value=Some("a"), num_args=0..=1)]
    entry: Option<String>,

    /// Provide the date as YYYY-MM-DD, to open the relevant entry in helix-editor.
    ///
    /// A feature to be able to edit any given entry at any time, provided it exists. If it doesn't exist,
    /// the program will exit. Opens the exact line number of the entry as well. Works only with Helix-Editor.
    #[arg(short, long, default_missing_value=Some("a"), num_args=0..=1)]
    open_entry: Option<String>,

    /// List all occurances of a tag in a given file; Defaults to current month's file.
    ///
    /// Makes a table that shows all the places where the tag has been used, and the date of its usage,
    /// so you can `--entry` or `--open-entry` for that date.
    /// If a month `--month` or `-m` has been passed along with this, the tag will be searched for that month.
    /// If a year `--year` or `-y` is passed along with this, but without a month, all files within that year
    /// will be recursively searched for the tag specified.
    /// If both year and month are provided, that exact file will be searched.
    /// Supports a pager `bat` when the number of rows in the table exceeds 10. \n
    /// A special tag is `[food]`, which when passed, makes a different table. Check the README
    #[arg(short, long, group = "get_tags")]
    tag: Option<String>,

    /// Provide a year(YYYY) to search for a tag in, or to generate a report
    ///
    /// Used in both `--tag` and `--gen-report`. Accepts a year YYYY to pass to either `--tag` or `--gen-report`.
    /// If any error of the year, it defaults to the current year.
    #[arg(short, long, requires = "get_tags")]
    year: Option<i32>,

    /// Provide the month(MM) to search for the tag in, or to generate a report
    ///
    /// Used in both `--tag` and `--gen-report`. Accepts a month MM to pass to either `--tag` or `--gen-report`.
    /// If any error of the month, it defaults to the current month.
    #[arg(short, long, requires = "get_tags")]
    month: Option<u32>,

    /// Generate a report about a given month's file; Defaults to current month's file.
    ///
    /// Generates a report, and prints it to STDOUT.
    /// Shows the number of entries in the month, and the most used tags along with their frequency in
    /// a table format.
    /// Can provide the month or year to generate the report of, using `--year`(`-y`) or `--month`(`-m`) flags.
    #[arg(long, group = "get_tags")]
    gen_report: bool,
}

fn main() {
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
    let args_tag_year = match args.year {
        None => {
            year_provided = false;
            today.year()
        }
        Some(year) => year,
    };
    let args_tag_month = match args.month {
        None => {
            month_provided = false;
            today.month()
        }
        Some(month) => month,
    };

    // Handle arguments - not very efficiently or idiomatically
    if args_tag != "" && args_entry == "" && args_open_entry == "" {
        handle_tags(
            args_tag,
            args_tag_year,
            args_tag_month,
            year_provided,
            month_provided,
        );
    }

    if args_entry != "" && args_tag == "" && args_open_entry == "" {
        let entry = get_entry(parse_entry_args(args_entry));
        println!("{}", entry);
    }

    if args_tag == "" && args_entry == "" && args_open_entry == "" && !args.gen_report {
        let today_date = today.format("%Y-%m-%d").to_string();
        open_editor(today_date);
    }

    if args_open_entry != "" && args_tag == "" && args_entry == "" {
        let entry_date_naive = parse_entry_args(args_open_entry);
        let entry_date = entry_date_naive.format("%Y-%m-%d").to_string();
        if get_entry(entry_date_naive) == format!("Entry does not exist for {}", entry_date) {
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
            // TODO: implement this
            gen_report_year(args_tag_year);
        } else {
            gen_report(args_tag_year, args_tag_month);
        }
    }
}
