use super::Location;

#[derive(Default)]
pub struct SearchData {
    pub search_string: String,
    pub count: usize,
    pub current_occurrence: usize,
    pub occurrence_list: Vec<Location>,
}
