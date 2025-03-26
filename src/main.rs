// Author: Tejas Gudgunti
use crate::utils::*;
use chrono::{DateTime, Datelike, Local, TimeZone};
use colored::Colorize;
mod utils;

fn main() {
    let today: DateTime<Local> = Local::now(); //Get `now` time
    let other_day: DateTime<Local> = Local.with_ymd_and_hms(2025, 3, 25, 5, 5, 5).unwrap();

    let filename: String = format!(
        "jrnl/{}/{}_{}.md",
        today.year(),
        today.year(),
        today.month()
    );
    if !check_file_existed(&filename) {
        println!("{}{:?}", "> Created file: ".green().bold(), &filename);
    };
    let _ = add_date_to_file(&filename, today);
    let entry = get_entry(other_day);
    println!("{}", entry);
    let tags = get_tags_from_file(&filename);
}
