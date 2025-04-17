<h1 align=center><code>jrnl</code></h1>

> A simple tool to list the day's activities, completely in CLI.

## Why?

Many similar tools exist, but this was sort of a learning project(for rust), and 
none of them met all the requirements I was looking for in a `jrnl`-ing system.

## Visit the Webpage

Visit the relevant webpage [here](https://kajuburfi.github.io/jrnl/) to view the 
- [Video Demo](https://kajuburfi.github.io/jrnl/#videoDemo) in case it doesn't work here.
- [Features](https://kajuburfi.github.io/jrnl/#features) in detail.
- [Usage](https://kajuburfi.github.io/jrnl/#usage), on how to properly use it(in detail).

## Demo
> Visit the website [here](https://kajuburfi.github.io/jrnl/#videoDemo) in case it doesn't work here.

https://github.com/user-attachments/assets/ed63f1c8-8ef2-4dc6-a1a4-02f20681abbe

## Features(in brief)
> Visit the [webpage](https://kajuburfi.github.io/jrnl/#features) to see the detailed features.

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

## Installation

Since this is just a side-project, there is not many ways to install this.

You can, of course, clone the repository and use `cargo` to install it, or use:
```sh
cargo install --git https://github.com/kajuburfi/jrnl --locked
```

> Note: This is meant for my personal use, and so many things are hardcoded. Use at your own risk. 


## Configuration

Just copy-paste the [config file](./config.toml) into `~/.config/jrnl/config.toml`, and make the necessary changes.
All data about the config file is mentioned in the comments.

> Since most of the things are made specifically for my needs(hardcoded), many options are not configurable.
>
> If you want to try it out, and get stuck, feel free to open an issue, and I'll see what I can do.
>
> tldr; Not specifically meant for other people's use(might not fit your needs, and is mostly not very much configurable)

## Dependencies

- [`chrono`](https://github.com/chronotope/chrono): For date and time purposes.
- [`clap`](https://github.com/clap-rs/clap): For CLI arguments.
- [`colored`](https://github.com/colored-rs/colored): For colored messages.
- [`inquire`](https://github.com/mikaelmello/inquire): To interactively get the date(of entry to be fetched) from the user.
- [`pager`](https://docs.rs/pager/latest/pager/): To page long outputs.
- [`comfy-table`](https://github.com/Nukesor/comfy-table): To print tables.
- [`toml`](https://docs.rs/toml/latest/toml/): To parse the configuration file.
- [`serde`](https://serde.rs/): For use in `toml` and `clap`.
- [`term_size`](https://docs.rs/term_size/latest/term_size/): To get the terminal width, to be able to wrap tables and calendars accordingly.
- [`stringmetrics`](https://docs.rs/stringmetrics/latest/stringmetrics/): For _approximate_ word searching.
- [`shellexpand`](https://docs.rs/shellexpand/latest/shellexpand/): To expand the `~`(tilde) in paths.

## License

This tool is licensed under the MIT license.
