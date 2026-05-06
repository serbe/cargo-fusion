use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Represents project metadata that will be included at the top of the generated file
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
            .context("No package section found in Cargo.toml")?;

        let readme = if let Some(readme_path) = package.readme().as_path() {
            let full_path = manifest_path
                .parent()
                .context("Manifest path has no parent")?
                .join(readme_path);
            fs::read_to_string(&full_path).ok()
        } else {
            // Пробуем стандартные имена файлов README
            let parent = manifest_path
                .parent()
                .context("Manifest path has no parent")?;
            ["README.md", "README", "Readme.md", "README.txt"]
                .iter()
                .find_map(|name| {
                    let path = parent.join(name);
                    if path.exists() {
                        fs::read_to_string(path).ok()
                    } else {
                        None
                    }
                })
        };

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
            writeln!(f, "// ======\n")?;
        }

        Ok(())
    }
}
