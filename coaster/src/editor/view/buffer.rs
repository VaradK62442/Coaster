use crate::editor::fileinfo::FileInfo;
use std::fs::{File, read_to_string};
use std::io::Error;
use std::io::Write;

use super::Location;
use super::line::Line;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
    pub file_info: FileInfo,
    pub dirty: bool,
}

impl Buffer {
    pub fn load(filename: &str) -> Result<Self, Error> {
        let contents = read_to_string(filename)?;
        let mut lines = Vec::new();
        for value in contents.lines() {
            lines.push(Line::from(value));
        }

        Ok(Self {
            lines,
            file_info: FileInfo::from(filename),
            dirty: false,
        })
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn save_as(&mut self, filename: &str) -> Result<(), Error> {
        let file_path = self.file_info.get_path();

        if file_path.is_some() {
            let mut path = file_path.unwrap().clone();
            if !filename.is_empty() {
                path.pop();
                path.push(filename);
            }

            let mut file = File::create(&path)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
            self.dirty = false;
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn insert_char(&mut self, character: char, at: Location) {
        if at.line_idx > self.height() {
            return;
        }
        if at.line_idx == self.height() {
            self.lines.push(Line::from(&character.to_string()));
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_idx) {
            line.insert_char(character, at.grapheme_idx);
            self.dirty = true;
        }
    }

    pub fn delete(&mut self, at: Location) {
        if let Some(line) = self.lines.get(at.line_idx) {
            if at.grapheme_idx >= line.grapheme_count()
                && self.height() > at.line_idx.saturating_add(1)
            {
                let next_line = self.lines.remove(at.line_idx + 1);
                self.lines[at.line_idx].append(&next_line);
                self.dirty = true;
            } else if at.grapheme_idx < line.grapheme_count() {
                self.lines[at.line_idx].delete(at.grapheme_idx);
                self.dirty = true;
            }
        }
    }

    pub fn insert_newline(&mut self, at: Location) {
        if at.line_idx == self.height() {
            self.lines.push(Line::default());
        } else if let Some(line) = self.lines.get_mut(at.line_idx) {
            let new = line.split(at.grapheme_idx);
            self.lines.insert(at.line_idx.saturating_add(1), new);
        }
        self.dirty = true;
    }

    pub fn search(&self, query: &str, from: Location) -> Option<(Location, usize)> {
        let mut first_occurrence = None;
        let mut count = 0;
        for (line_idx, line) in self.lines.iter().enumerate().skip(from.line_idx) {
            let from_grapheme_idx = if line_idx == from.line_idx {
                from.grapheme_idx
            } else {
                0
            };
            if let Some(grapheme_idx) = line.search(query, from_grapheme_idx) {
                if first_occurrence.is_none() {
                    first_occurrence = Some(Location {
                        grapheme_idx,
                        line_idx,
                    });
                }
                count += 1;
            }
        }

        for (line_idx, line) in self.lines.iter().enumerate().take(from.line_idx) {
            if let Some(grapheme_idx) = line.search(query, 0) {
                if first_occurrence.is_none() {
                    first_occurrence = Some(Location {
                        grapheme_idx,
                        line_idx,
                    });
                }
                count += 1;
            }
        }

        first_occurrence.map(|loc| (loc, count))
    }
}
