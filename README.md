<h1 align=center><code>jrnl</code></h1>

> A simple tool to list the day's activities, completely in CLI.

## Why?

Many similar tools exist, but this was sort of a learning project(for rust), and 
none of them met all the requirements I was looking for in a `jrnl`-ing system.

## Demo

https://github.com/user-attachments/assets/ed63f1c8-8ef2-4dc6-a1a4-02f20681abbe

## Features(in brief)

- All data is stored in markdown format(so **nothing** is _encrypted_).
- You can have multiple different `jrnl_folder`s, for different purposes, and can use any one from any
  where using the `--path`(`-p`) flag.
- Tag system
  ```
  - [tag1] [tag2] Some data.
  ```
- Already filled in data(date, weekday, time, etc) in your file.
- A specific ordering system, with 1 file per month.
- Prints calendars with highlighted dates.
- Search system.
- Specific `food` tag.
- _Slightly_ configurable(I hardcoded most of the things).
- Multiple `jrnl`s are supported, with a default one. The specific one needed can be chosen using `--path`.

## Installation

Since this is just a side-project, there is not many ways to install this.

You can, of course, clone the repository and use `cargo` to install it, or use:
```sh
cargo install --git https://github.com/kajuburfi/jrnl
```

> Note: This is meant for my personal use, and so many things are hardcoded. Use at your own risk. 

## Features
- Arranges entries of each day such that a month of entries is stored in a file. 
  File structure(tree):
  ```
  Template           |   Example
                     |
  jrnl_folder        |   jrnl_folder
  '- YYYY            |   '- 2025
  |  '- YYYY_MM.md   |   |  '- 2025_01.md
  |  '- YYYY_MM.md   |   |  '- 2025_02.md
  '- YYYY            |   '- 2026
  |  '- YYYY_MM.md   |   |  '- 2026_01.md
  |  '- YYYY_MM.md   |   |  '- 2026_02.md
  ```
- Just by running `jrnl` with no flags opens the current day's entry in your text editor.
  Automatically fills in the date - weekday and time is configurable.
- General entry format:
  ```
  Template                |   Example
  ### WEEKDAY (HH:MM:SS)  |   ### FRI (13:05:28)
  # YYYY-MM-DD            |   # 2025-03-28
  - [tag] entry           |   - [milestone] [game] Played and won 200th game of chess.
  - entry                 |   - Cleaned up room.
  - [tag1] [tag2] entry   |   - [fees] Paid electricity bill.
  ```
- All entries are in standard markdown, for ease of reading.
- Entries of any date can be fetched and pretty-printed to the terminal.
- Any given entry can be opened with the text editor at that date, in case of any editing required.
- Tags are implemented; one can search for all occurances of a tag in a given month or year.
  When searched for, a table is printed, with the dates and respective entries that contain
  the given tag. The tag is highlighted. Further, a calendar of the current month(or months - if in a year)
  is printed, with the dates of the tags used being highlighted.
  ```
  ╭───────────────┬──────────────────────────────────────────╮
  │ Date of Entry ┆ Record                                   │
  ╞═══════════════╪══════════════════════════════════════════╡
  │ 2025-03-31    ┆ [tag_1] Stuff                            │
  ├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
  │ 2025-03-29    ┆ [tag_1] Some more                        │ 
  ├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
  │ 2025-03-28    ┆ [tag_1] Other things                     │
  ╰───────────────┴──────────────────────────────────────────╯
       March 2025
  Mo Tu We Th Fr Sa Su
                  1  2
   3  4  5  6  7  8  9
  10 11 12 13 14 15 16
  17 18 19 20 21 22 23
  24 25 26 27 28 29 30
  31
  ```
  (Note: Here, the colors cannot be shown, so you'll have to trust this.)
- A special tag - `food` is pre-defined. Input your daily food intake as:
  ```
  - [food] Breakfast | Lunch | Dinner | Other
  <!-- Example -->
  - [food] some breakfast item | A lunch item - course 1 A lunch item - course 2 A lunch item - course 3 | A filling dinner. A tasty dinner. | Snack - Chips Fruit - Mango
  ```
  When fetching the tag through `--tag food`, you get a nice ascii table:
  ```
  ╭───────────────┬─────────────────────┬─────────────────────────┬───────────────────┬────────────────╮
  │ Date of Entry ┆ Breakfast           ┆ Lunch                   ┆ Dinner            ┆ Other          │
  ╞═══════════════╪═════════════════════╪═════════════════════════╪═══════════════════╪════════════════╡
  │ 2025-03-27    ┆ some breakfast item ┆ A lunch item - course 1 ┆ A filling dinner. ┆ Snack - Chips  │
  │               ┆                     ┆ A lunch item - course 2 ┆ A tasty dinner.   ┆ Fruit - Mango  │
  │               ┆                     ┆ A lunch item - course 3 ┆                   ┆                │
  ╰───────────────┴─────────────────────┴─────────────────────────┴───────────────────┴────────────────╯
  ```
- If there are too many entries for a tag or for the food tag(specifically), it will automatically open
  a pager with the contents(configurable). 
- To fetch entries(or open them), a date is required. You can either pass this through the flag `--entry YYYY-MM-DD`
  (`-e YYYY-MM-DD`), or if you just pass an empty flag(`-e`), an interactive calendar will prompt for the 
  date(Using [inquire](https://github.com/mikaelmello/inquire)). The calendar will also open if there is any 
  problem with reading the date.
- When fetching tags, the default file to search for is the current month's file. However, you can specify any other 
  file using `--year YYYY`(`-y YYYY`) or `--month MM`(`-m MM`). If only provided with a year, and no month, all files
  of that year will be searched through and printed chronologically.(The pager comes in use here)
- A feature of "generating reports" is implemented. Currently, it goes through a file, and prints the number of entries
  of that month, and the most used tags. Similar to tags, you can specify the month and/or year to get a specific month's
  report.
- All tables are automatically wrapped around if its width exceeds 90% of the terminal width.
- Calendars are printed in tags and generating reports, with highlighting. 
  The calendars of each month will be printed in a grid-like form, extending rightwards depending
  on your terminal's width. 

## Configuration

Just copy-paste the [config file](./config.toml) into `~/.config/jrnl/config.toml`, and make the necessary changes.
All data about the config file is mentioned in the comments.

> Since most of the things are made specifically for my needs(hardcoded), many options are not configurable.
>
> If you want to try it out, and get stuck, feel free to open an issue, and I'll see what I can do.
>
> tldr; Not specifically meant for other people's use(might not fit your needs, and is mostly not very much configurable)

## License

This tool is licensed under the MIT license.
