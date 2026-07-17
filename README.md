# Coaster

A custom text editor build in Rust, deriving the [hecto](https://philippflenker.com/hecto/) editor. Currently supports editing, searching, and some vim commands.

### Config file syntax

Each config file variable should be stored as `key = value`.
Supported keys and their allowed values:
- `numbering` - Controls how line numbers are displayed.
  - `relative` - Line numbers are relative to the line number of the cursor, current line number is 0.
  - `absolute` (default) - Line numbers correspond to the line number of the overall file.
  - `hybrid` - Line numbers are relative to the line number of the cursor, current line number is the line number in the overall file.

## Future work

- Creating and saving new files
- Syntax highlighting via LSP
