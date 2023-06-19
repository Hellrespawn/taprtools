# taprtools

Uses [tapr](https://github.com/Hellrespawn/tapr)-scripts to rename audio files according to their tags.

## Requirements

- Rust (MSRV: 1.70)

## Installation

1. Ensure `cargo` and Cargo's `bin` folder are on your `PATH`.
1. Clone the repository.
1. Run `cargo install --path taprtools`.

## Usage

```sh
$ taprtools -h

taprtools 0.11.0
Tag Processing Tools. Renames audio files according to their tags.

USAGE:
    tapr [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -c, --config <CONFIG>    Sets a custom config file
    -h, --help               Print help information
    -p, --preview            Only preview current action
    -V, --version            Print version information

SUBCOMMANDS:
    clear     Clears the history
    help      Print this message or the help of the given subcommand(s)
    list      Lists all scripts
    redo      Redo {times} times
    rename    Rename files according to their tags
    undo      Undo {times} times
```
