use crate::filter::FileFilter;
use crate::fs_utils::read_file;
use crate::workspace::WorkspaceResolution;
use crate::{args::FusionArgs, manifest_utils::collect_all_path_dependencies};
use anyhow::{Result, bail};
use ignore::{
    WalkBuilder,
    overrides::{Override, OverrideBuilder},
};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct FileCollector;

impl FileCollector {
    pub fn collect_files(
        args: &FusionArgs,
        resolution: &WorkspaceResolution,
    ) -> Result<Vec<(PathBuf, Vec<u8>)>> {
        let search_paths = Self::build_search_paths(args, resolution)?;
        let file_paths = Self::find_files(&search_paths, args)?;
        Self::read_files(file_paths)
    }

    fn build_search_paths(
        args: &FusionArgs,
        resolution: &WorkspaceResolution,
    ) -> Result<Vec<PathBuf>> {
        let mut paths = Self::get_valid_include_paths(args);
        paths.push(resolution.search_root.clone());

        if args.dependencies {
            paths.extend(Self::collect_dependencies(&resolution.manifest_path)?);
        }

        Self::deduplicate_paths(paths)
    }

    fn get_valid_include_paths(args: &FusionArgs) -> Vec<PathBuf> {
        args.include
            .iter()
            .filter(|p| {
                if !p.exists() {
                    eprintln!("Warning: include path not found: {}", p.display());
                    false
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }

    fn find_files(paths: &[PathBuf], args: &FusionArgs) -> Result<Vec<PathBuf>> {
        let walker = Self::create_walker(paths, args);
        let filter = FileFilter::from_args(args);

        let files: Vec<PathBuf> = walker
            .build()
            .par_bridge()
            .filter_map(|entry| entry.ok().and_then(|e| filter.apply(&e)))
            .collect();

        if files.is_empty() {
            bail!("No files found to include");
        }

        Ok(Self::apply_file_limit(files, args.max_files))
    }

    fn create_walker(paths: &[PathBuf], args: &FusionArgs) -> WalkBuilder {
        let mut walker = WalkBuilder::new(&paths[0]);

        // Добавляем остальные пути
        for path in &paths[1..] {
            walker.add(path);
        }

        // Настраиваем параметры
        walker
            .max_depth(args.depth)
            .standard_filters(args.skip_gitignore)
            .follow_links(false)
            .same_file_system(true);

        // Добавляем overrides отдельно, так как они опциональны
        if let Some(overrides) = Self::build_exclude_overrides(args) {
            walker.overrides(overrides);
        }

        walker
    }

    fn build_exclude_overrides(args: &FusionArgs) -> Option<Override> {
        if args.exclude.is_empty() {
            return None;
        }

        let mut builder = OverrideBuilder::new(".");
        for pattern in &args.exclude {
            let _ = builder.add(&format!("!{pattern}"));
        }
        builder.build().ok()
    }

    fn apply_file_limit(mut files: Vec<PathBuf>, max_files: Option<usize>) -> Vec<PathBuf> {
        if let Some(max) = max_files
            && files.len() > max
        {
            eprintln!("Found {} files, truncating to {}", files.len(), max);
            files.truncate(max);
        }
        files
    }

    fn read_files(paths: Vec<PathBuf>) -> Result<Vec<(PathBuf, Vec<u8>)>> {
        let mut files: Vec<_> = paths
            .into_par_iter()
            .filter_map(|path| read_file(&path).ok().map(|content| (path, content)))
            .collect();

        if files.is_empty() {
            bail!("No files could be read successfully");
        }

        files.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        Ok(files)
    }

    fn collect_dependencies(manifest_path: &Path) -> Result<Vec<PathBuf>> {
        let mut visited = HashMap::new();
        collect_all_path_dependencies(manifest_path, &mut visited)
    }

    fn deduplicate_paths(mut paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
        paths.sort();
        paths.dedup();

        if paths.is_empty() {
            bail!("No search paths found");
        }

        Ok(paths)
    }
}
