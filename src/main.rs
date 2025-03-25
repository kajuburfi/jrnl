// Author: Tejas Gudgunti
use chrono::Datelike;
use std::fs::File;
use std::path::Path;

fn main() {
    let today = chrono::Local::now(); //Get `now` time
    let today_date = today.format("%Y-%m-%d");
    println!("{}", today_date);

    let filename = format!(
        "jrnl/{}/{}_{}.md",
        today.year(),
        today.year(),
        today.month()
    );
    check_file_exist(filename);
}

/// Checks if the file exists, if not, it makes the file.
fn check_file_exist(filename: String) {
    let path = Path::new(&filename);
    println!("{:#?}; {}", path, path.exists());

    if !path.exists() {
        let data_file = File::create_new(filename).expect("Could not make new file.");
        println!("Made file {:?}", data_file);
    }
}
