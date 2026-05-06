use crate::args::FusionArgs;
use chrono::{DateTime, NaiveDateTime, Utc};
use ignore::DirEntry;
use std::{
    ffi::OsStr,
    fs::Metadata,
    path::{Path, PathBuf},
};

pub struct FileFilter {
    extensions: Vec<String>,
    max_size: Option<u64>,
    min_size: Option<u64>,
    min_date: Option<NaiveDateTime>,
    max_date: Option<NaiveDateTime>,
    include_lock: bool,
}

impl FileFilter {
    pub fn from_args(args: &FusionArgs) -> Self {
        Self {
            extensions: args.extension.clone(),
            max_size: args.smaller_than,
            min_size: args.larger_than,
            min_date: args.newer_than,
            max_date: args.older_than,
            include_lock: args.include_lock,
        }
    }

    pub fn apply(&self, entry: &DirEntry) -> Option<PathBuf> {
        let path = entry.path();

        // Check lock file
        if !self.include_lock && path.file_name() == Some(OsStr::new("Cargo.lock")) {
            return None;
        }

        // Check extension
        path.extension()
            .and_then(|ext| ext.to_str())
            .filter(|ext| self.extensions.contains(&ext.to_string()))?;

        // Check metadata
        let metadata = entry.metadata().ok()?;
        if !self.check_size(&metadata) || !self.check_date(&metadata) {
            return None;
        }

        Some(path.to_path_buf())
    }

    fn check_size(&self, metadata: &Metadata) -> bool {
        let size = metadata.len();

        let passes_max = self.max_size.map(|max| size <= max).unwrap_or(true);
        let passes_min = self.min_size.map(|min| size >= min).unwrap_or(true);

        passes_max && passes_min
    }

    fn check_date(&self, metadata: &Metadata) -> bool {
        let modified: DateTime<Utc> = match metadata.modified() {
            Ok(m) => m.into(),
            Err(e) => {
                eprintln!("Warning: cannot get modification time: {}", e);
                return false;
            }
        };

        let after_min = self
            .min_date
            .map(|min| modified >= min.and_utc())
            .unwrap_or(true);

        let before_max = self
            .max_date
            .map(|max| modified <= max.and_utc())
            .unwrap_or(true);

        after_min && before_max
    }
}
