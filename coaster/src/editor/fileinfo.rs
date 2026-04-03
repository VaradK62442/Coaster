use std::{
    fmt::{self, Display},
    path::PathBuf,
};

#[derive(Default, Debug, Clone)]
pub struct FileInfo {
    path: Option<PathBuf>,
}

impl FileInfo {
    pub fn from(filename: &str) -> Self {
        Self {
            path: Some(PathBuf::from(filename)),
        }
    }

    pub fn get_path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }
}

impl Display for FileInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[No Name]");
        write!(formatter, "{name}")
    }
}
