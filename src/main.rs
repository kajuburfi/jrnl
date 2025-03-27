// Author: Tejas Gudgunti
use crate::utils::*;
use chrono::{DateTime, Datelike, Local};
use clap::Parser;
use colored::Colorize;
use std::process;

mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Cli {
    /// Provide the date as YYYY-MM-DD, to fetch the relevant entry.
    #[arg(short, long, default_missing_value=Some("a"), require_equals=true, num_args=0..=1)]
    entry: Option<String>,

    /// Provide the date as YYYY-MM-DD, to open the relevant entry in helix.
    #[arg(short, long)]
    open_entry: Option<String>,

    /// List all occurances of a tag in a given file.
    #[arg(short, long, group = "get_tags")]
    tag: Option<String>,

    /// short help
    ///
    /// Provide the year to search for the tag in. Note: Requires -tag;
    /// Defaults to current year. If year is provided, but month is not:
    /// All occurances of the tag within the year will be shown.
    #[arg(short, long, requires = "get_tags")]
    year: Option<i32>,

    /// Provide the month to search for the tag in. Note: Requires -tag;
    /// Defaults to current month.
    #[arg(short, long, requires = "get_tags")]
    month: Option<u32>,
}

fn main() {
    let today: DateTime<Local> = Local::now(); //Get `now` time

    let mut month_provided: bool = true;
    let mut year_provided: bool = true;

    let args = Cli::parse();
    // Use `.as_deref()` to convert Option<String> to Option<&str>
    let args_entry = match args.entry.as_deref() {
        None => "",
        Some("a") => &inquire_date().format("%Y-%m-%d").to_string(),
        Some(entry) => &entry,
    };
    let args_open_entry = match args.open_entry.as_deref() {
        None => "",
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

    if args_tag == "" && args_entry == "" && args_open_entry == "" {
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
}
