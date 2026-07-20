# Coaster

A custom text editor build in Rust, deriving the [hecto](https://philippflenker.com/hecto/) editor. Currently supports editing, searching, and some vim commands.

### Full feature list

- Open files by running `coaster <filename>`
- Coaster welcome message shown when no file specified
- Show file name, lines, modified status and line number in status bar
- Show current mode in message bar
- Normal and Insert mode
- Edit file in insert mode
- Enter insert mode by pressing `i`
- Move to next character and enter insert mode by pressing `a`
- Jump to start of line and enter insert mode by pressing `I`
- Jump to end of line and enter insert mode by pressing `A`
- Move cursor with `h`, `j`, `k`, `l` and arrow keys
- Jump to borders of screen using Home, Pg Down, Pg Up, End
- Save and exit commands with `:` prefix
	 - Exit with `q`
  - Exit without saving with `q!`
  - Save with `w`
  - Save and quit with `x`
  - Jump to line numbers by entering the number
- Can combine multiple commands into one
- Configuration file support (see below)
- Relative, absolute, and hybrid line numbering
- Delete character with `x`
- Jump to start / end of line with `0` and `$`
- Search file contents (WIP)
- Jump to start / end / back of word with `w`, `e`, and `b`
- Insert new line below / above with `o` and `O`
- Jump to top / bottom of file with `g` and `G`

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
