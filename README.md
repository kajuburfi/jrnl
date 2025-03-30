# `jrnl`
A simple tool to list the day's activities.

# Features
- Arranges entries of each day such that a month of entries is stored in a file. 

  File structure(tree):
  ```
  Template           |   Example
                     |
  jrnl               |   jrnl
  '- config.toml     |   '- config.toml
  '- YYYY            |   '- 2025
  |  '- YYYY_MM.md   |   |  '- 2025_01.md
  |  '- YYYY_MM.md   |   |  '- 2025_02.md
  '- YYYY            |   '- 2026
  |  '- YYYY_MM.md   |   |  '- 2026_01.md
  |  '- YYYY_MM.md   |   |  '- 2026_02.md
  ```
- Just by running `jrnl` with no flags opens the current day's entry in your text editor.
  Automatically fills in the date - weekday is configurable.
- General entry format:
  ```
  Template                |   Example
  ### WEEKDAY             |   ### FRI
  # YYYY-MM-DD            |   # 2025-03-28
  - [tag] entry           |   - [milestone] [game] Played and won 200th game of chess.
  - entry                 |   - Cleaned up room.
  - [tag1] [tag2] entry   |   - [fees] Paid electricity bill.
  ```
- All entries are in standard markdown, for ease of reading.
- Entries of any date can be fetched and pretty-printed to the terminal.
- Any given entry can be opened with the text editor at that date, in case of any editing required.
- Tags are implemented; one can search for all occurances of a tag in a given month or year.
- A special tag - `food` is pre-defined. Input your daily food intake as:
  ```
  # YYYY-MM-DD
  - [food] Breakfast | Lunch | Dinner | Other
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
  a pager with the contents. 
- To fetch entries(or open them), a date is required. You can either pass this through the flag `--entry YYYY-MM-DD`
  (`-e YYYY-MM-DD`), or if you just pass an empty flag(`-e`), an interactive calendar will prompt for the 
  date(Using [inquire](https://github.com/mikaelmello/inquire)). The calendar will open if there is any 
  problem with reading the date.
- When fetching tags, the default file to search for is the current month's file. However, you can specify any other 
  file using `--year YYYY`(`-y YYYY`) or `--month MM`(`-m MM`). If only provided with a year, and no month, all files
  of that year will be searched through and printed chronologically.(The pager comes in use here)
- A feature of "generating reports" is implemented. Currently, it goes through a file, and prints the number of entries
  of that month, and the most used tags. Similar to tags, you can specify the month and/or year to get a specific month's
  report.
