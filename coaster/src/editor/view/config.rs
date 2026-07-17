use dirs::home_dir;
use std::collections::HashMap;
use std::fs::read_to_string;

#[derive(Default)]
pub struct Config {
    pub config: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        let mut config = HashMap::new();
        if let Some(home_dir) = home_dir() {
            if let Some(config_path) = home_dir.join(".coastrc").into_os_string().to_str() {
                if let Some(content) = read_to_string(config_path).ok() {
                    for line in content.lines() {
                        if let Some((key, value)) = line.split_once('=') {
                            config.insert(key.trim().to_string(), value.trim().to_string());
                        }
                    }
                }
            }
        }

        Self { config }
    }

    pub fn get_formatted_line_number(
        &self,
        row: usize,
        line_idx: usize,
        line_padding: usize,
    ) -> String {
        let line_number;
        if let Some(numbering) = self.config.get("numbering") {
            match numbering.as_str() {
                "relative" => line_number = line_idx.abs_diff(row),
                "absolute" => line_number = row.saturating_add(1),
                "hybrid" => {
                    if line_idx.abs_diff(row) == 0 {
                        line_number = row.saturating_add(1);
                    } else {
                        line_number = line_idx.abs_diff(row);
                    }
                }
                _ => panic!("Invalid config set for `numbering`: {}", numbering),
            }
        } else {
            line_number = row.saturating_add(1);
        }

        if line_idx == row {
            format!("{:<width$} ", line_number, width = line_padding)
        } else {
            format!("{:>width$} ", line_number, width = line_padding)
        }
    }
}
