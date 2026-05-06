use chrono::{DateTime, NaiveDateTime, Utc};
use std::path::PathBuf;

use crate::args::FusionArgs;

/// Encapsulates all file-filtering logic in one place,
/// avoiding the long parameter list that `filter_path` previously had.
pub struct FileFilter<'a> {
    extension: &'a [String],
    smaller_than: Option<u64>,
    larger_than: Option<u64>,
    newer_than: Option<NaiveDateTime>,
    older_than: Option<NaiveDateTime>,
    include_lock: bool,
}

impl<'a> FileFilter<'a> {
    pub fn from_args(args: &'a FusionArgs) -> Self {
        Self {
            extension: &args.extension,
            smaller_than: args.smaller_than,
            larger_than: args.larger_than,
            newer_than: args.newer_than,
            older_than: args.older_than,
            include_lock: args.include_lock,
        }
    }

    /// Returns `Some(path)` if the entry passes all filters, `None` otherwise.
    pub fn apply(&self, entry: &ignore::DirEntry) -> Option<PathBuf> {
        let path = entry.path();

        if !self.include_lock && path.file_name() == Some(std::ffi::OsStr::new("Cargo.lock")) {
            return None;
        }

        let has_valid_extension = self.extension.iter().any(|ext| {
            path.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == ext)
        });

        if !has_valid_extension {
            return None;
        }

        if !self.passes_metadata_filter(entry) {
            return None;
        }

        Some(path.to_path_buf())
    }

    fn passes_metadata_filter(&self, entry: &ignore::DirEntry) -> bool {
        let needs_check = self.smaller_than.is_some()
            || self.larger_than.is_some()
            || self.newer_than.is_some()
            || self.older_than.is_some();

        if !needs_check {
            return true;
        }

        let Ok(meta) = entry.metadata() else {
            eprintln!(
                "Warning: could not read metadata for {}",
                entry.path().display()
            );
            return false;
        };

        if let Some(max_size) = self.smaller_than
            && meta.len() > max_size
        {
            return false;
        }

        if let Some(min_size) = self.larger_than
            && meta.len() < min_size
        {
            return false;
        }

        if self.newer_than.is_some() || self.older_than.is_some() {
            let modified: DateTime<Utc> = match meta.modified() {
                Ok(m) => m.into(),
                Err(_) => return false,
            };

            if let Some(newer) = self.newer_than
                && modified < newer.and_utc()
            {
                return false;
            }

            if let Some(older) = self.older_than
                && modified > older.and_utc()
            {
                return false;
            }
        }

        true
    }
}
