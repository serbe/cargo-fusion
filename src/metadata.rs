use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Project metadata included at the top of the generated file.
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
    pub fn from_manifest(manifest_path: &Path) -> Result<Self> {
        let manifest =
            cargo_toml::Manifest::from_path(manifest_path).context("Failed to read Cargo.toml")?;

        let package = manifest
            .package
            .context("No [package] section found in Cargo.toml")?;

        let readme = Self::read_readme(manifest_path, &package);

        Ok(Self {
            name: package.name.clone(),
            version: package.version().to_string(),
            description: package.description().map(ToString::to_string),
            readme,
            repository: package.repository().map(ToString::to_string),
            authors: package.authors().to_vec(),
            license: package.license().map(ToString::to_string),
        })
    }

    fn read_readme(manifest_path: &Path, package: &cargo_toml::Package) -> Option<String> {
        let parent = manifest_path.parent()?;

        // Prefer the readme path declared in Cargo.toml.
        if let Some(readme_path) = package.readme().as_path() {
            let full = parent.join(readme_path);
            if let Ok(text) = fs::read_to_string(&full) {
                return Some(text);
            }
        }

        // Fall back to common README filenames.
        ["README.md", "README", "Readme.md", "README.txt"]
            .iter()
            .find_map(|name| fs::read_to_string(parent.join(name)).ok())
    }
}

impl std::fmt::Display for ProjectMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
