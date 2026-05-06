use anyhow::Result;
use cargo_toml::Package;
use std::{
    fmt::{Display, Formatter},
    path::Path,
};

use crate::{
    fs_utils::read_file_to_string,
    manifest_utils::{get_package, parse_manifest},
};

#[derive(Debug)]
pub struct ProjectMetadata {
    name: String,
    version: String,
    description: Option<String>,
    readme: Option<String>,
    repository: Option<String>,
    authors: Vec<String>,
    license: Option<String>,
}

impl ProjectMetadata {
    pub fn from_manifest(manifest_path: &Path) -> Result<Option<Self>> {
        let manifest = parse_manifest(manifest_path)?;
        let package = get_package(&manifest)?;

        Ok(Some(Self {
            name: package.name.clone(),
            version: package.version().to_string(),
            description: package.description().map(ToString::to_string),
            readme: Self::find_readme(manifest_path, package),
            repository: package.repository().map(ToString::to_string),
            authors: package.authors().to_vec(),
            license: package.license().map(ToString::to_string),
        }))
    }

    fn find_readme(manifest_path: &Path, package: &Package) -> Option<String> {
        let parent = manifest_path.parent()?;

        // Try explicit readme path first
        if let Some(readme_path) = package.readme().as_path()
            && let Ok(content) = read_file_to_string(&parent.join(readme_path))
        {
            return Some(content);
        }

        // Fallback to common README filenames
        const README_NAMES: &[&str] = &["README.md", "README", "Readme.md", "README.txt"];
        README_NAMES
            .iter()
            .find_map(|name| read_file_to_string(&parent.join(name)).ok())
    }
}

impl Display for ProjectMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "// Project: {} (v{})", self.name, self.version)?;

        if let Some(desc) = &self.description {
            writeln!(f, "// Description: {desc}")?;
        }
        if !self.authors.is_empty() {
            writeln!(f, "// Authors: {}", self.authors.join(", "))?;
        }
        if let Some(license) = &self.license {
            writeln!(f, "// License: {license}")?;
        }
        if let Some(repo) = &self.repository {
            writeln!(f, "// Repository: {repo}")?;
        }
        if let Some(readme) = &self.readme {
            writeln!(f, "\n// README\n// ======")?;
            for line in readme.lines() {
                writeln!(f, "// {line}")?;
            }
            writeln!(f, "// ======")?;
        }

        Ok(())
    }
}
