/*
* Command parser
*
* Supported commands:
* q            => quit
* q!           => force quit
* w            => save
* w <filename> => save as <filename>
* x            => write and quit (alias for `wq`)
* <number>     => jump to line number given
*/

pub enum Command {
    Save(String),
    JumpToLine(usize),
    Quit,
    ForceQuit,
}

fn get_next_token(tokens: Vec<&str>, i: usize) -> Option<&str> {
    if i + 1 < tokens.len() {
        Some(tokens[i + 1])
    } else {
        None
    }
}

fn get_next_subtoken(token: &str, i: usize) -> Option<String> {
    if i + 1 < token.len() {
        Some(token.chars().nth(i + 1).unwrap().to_string())
    } else {
        None
    }
}

pub fn parse_command(command: &str) -> Vec<Command> {
    let mut commands = Vec::new();
    let tokens = command.split_whitespace().collect::<Vec<&str>>();
    let num_tokens = tokens.clone().len();

    let mut i = 0;
    let mut j = 0;
    let mut token;
    let mut subtoken;

    while i < num_tokens {
        token = tokens.clone()[i];

        if token.parse::<f64>().is_ok() {
            // we are jumping to a line number, ignore all other tokens
            if let Some(line_number) = token.parse::<usize>().ok() {
                commands.push(Command::JumpToLine(line_number));
            }
            return commands;
        }

        while j < token.len() {
            subtoken = token.chars().nth(j).unwrap();
            match subtoken {
                'q' => {
                    if let Some(next_char) = get_next_subtoken(token, j) {
                        if next_char == "!" {
                            commands.push(Command::ForceQuit);
                            j += 1; // skip the next subtoken
                        } else {
                            commands.push(Command::Quit);
                        }
                    } else {
                        commands.push(Command::Quit);
                    }
                }
                'w' => {
                    // if `w` is the last character in the token
                    // assume next token is filename
                    if j + 1 == token.len()
                        && let Some(next_token) = get_next_token(tokens.clone(), i)
                    {
                        commands.push(Command::Save(next_token.to_string()));
                        i += 1; // skip the next token
                    } else {
                        commands.push(Command::Save("".to_string()));
                    }
                }
                'x' => {
                    commands.push(Command::Save("".to_string()));
                    commands.push(Command::Quit);
                }
                _ => {}
            }
            j += 1;
        }
        j = 0;
        i += 1;
    }

    commands
}
