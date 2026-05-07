// src/workspace.rs
//
// Resolves the correct manifest and source scope for cargo-fusion.
//
// Two modes, detected automatically:
//
//   1. Crate mode  – the user runs from inside a workspace member.
//      The given Cargo.toml has no [workspace] section.
//      → parse via the workspace root (so inherited fields resolve),
//        but restrict the file search to the **crate directory only**.
//
//   2. Workspace mode – the user runs from the workspace root.
//      The given Cargo.toml owns a [workspace] section.
//      → parse and search the entire workspace tree (original behaviour).

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{error, info};

use crate::utils::display_path;

/// Everything `resolve_workspace_manifest` needs to tell the caller.
#[derive(Debug)]
pub struct WorkspaceResolution {
    /// The manifest to pass to `cargo_toml::Manifest::from_path`.
    /// Always the workspace-root `Cargo.toml` (or the original path if no
    /// workspace was found).
    pub manifest_path: PathBuf,

    /// The directory that should be used as the search root for source files.
    ///
    /// * Crate mode  → the crate directory (e.g. `crates/xray-daemon`)
    /// * Workspace mode → the workspace root directory
    pub search_root: PathBuf,

    /// `true` when we detected that the user invoked the tool from inside a
    /// workspace member rather than from the workspace root.
    pub is_crate_mode: bool,
}

/// Inspect the given manifest and return a [`WorkspaceResolution`] that
/// tells the caller which directory to use as the search root.
pub fn resolve_workspace_manifest(manifest_path: &Path) -> Result<WorkspaceResolution> {
    let canonical = manifest_path
        .canonicalize()
        .with_context(|| format!("can't load root workspace: {}", manifest_path.display()))?;

    info!("Canonical path: {}", display_path(&canonical));

    let crate_dir = canonical
        .parent()
        .context("Cargo.toml has no parent directory")?
        .to_path_buf();

    info!("Crate path: {}", display_path(&crate_dir));

    // Case 1: the given Cargo.toml already has a [workspace] section.
    // → workspace mode, search root = crate_dir (which IS the workspace root).
    if manifest_has_workspace_section(&canonical)? {
        return Ok(WorkspaceResolution {
            manifest_path: canonical.clone(),
            search_root: crate_dir,
            is_crate_mode: false,
        });
    }

    // Case 2: the given Cargo.toml is a workspace member.
    // Walk up to find the workspace root so cargo_toml can resolve inherited
    // fields, but keep the *search root* pointing at the crate directory.
    let mut current = crate_dir.parent();
    while let Some(dir) = current {
        let candidate = dir.join("Cargo.toml");
        if candidate.exists() && workspace_root_contains(dir, &candidate, &crate_dir)? {
            error!(
                "info: workspace root at {}, collecting only crate at {}",
                candidate.display(),
                crate_dir.display(),
            );
            return Ok(WorkspaceResolution {
                manifest_path: candidate,
                // ↓ Crucial: search only the crate subtree, not the whole workspace.
                search_root: crate_dir,
                is_crate_mode: true,
            });
        }
        current = dir.parent();
    }

    // No workspace root found – fall back: use the original manifest and directory.
    Ok(WorkspaceResolution {
        manifest_path: canonical,
        search_root: crate_dir,
        is_crate_mode: false,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn manifest_has_workspace_section(path: &Path) -> Result<bool> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    Ok(contains_workspace_table(&text))
}

fn contains_workspace_table(toml_text: &str) -> bool {
    toml_text.lines().any(|line| {
        let t = line.trim();
        t == "[workspace]" || t.starts_with("[workspace.")
    })
}

fn workspace_root_contains(
    workspace_dir: &Path,
    candidate_manifest: &Path,
    crate_dir: &Path,
) -> Result<bool> {
    let text = std::fs::read_to_string(candidate_manifest)
        .with_context(|| format!("failed to read {}", candidate_manifest.display()))?;

    if !contains_workspace_table(&text) {
        return Ok(false);
    }

    let value: toml::Value = toml::from_str(&text)
        .with_context(|| format!("failed to parse {}", candidate_manifest.display()))?;

    let members = value
        .get("workspace")
        .and_then(|ws| ws.get("members"))
        .and_then(|m| m.as_array());

    let Some(members) = members else {
        // [workspace] with no members list → virtual workspace covering everything.
        return Ok(true);
    };

    for member in members {
        let Some(pattern) = member.as_str() else {
            continue;
        };
        let glob_path = workspace_dir.join(pattern);
        if glob_matches(&glob_path, crate_dir, workspace_dir) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn glob_matches(pattern_path: &Path, crate_dir: &Path, workspace_dir: &Path) -> bool {
    let pattern_str = match pattern_path.to_str() {
        Some(s) => s,
        None => return false,
    };

    if pattern_str.contains('*') {
        simple_glob_match(pattern_str, crate_dir, workspace_dir)
    } else {
        match pattern_path.canonicalize() {
            Ok(cp) => crate_dir == cp || crate_dir.starts_with(&cp),
            Err(_) => false,
        }
    }
}

fn simple_glob_match(pattern: &str, crate_dir: &Path, workspace_dir: &Path) -> bool {
    let rel = match crate_dir.strip_prefix(workspace_dir) {
        Ok(r) => r,
        Err(_) => return false,
    };
    let rel_str = rel.to_string_lossy().replace('\\', "/");

    let ws_str = workspace_dir.to_string_lossy().replace('\\', "/");
    let pattern_rel = pattern
        .replace('\\', "/")
        .trim_start_matches(&*ws_str)
        .trim_start_matches('/')
        .to_string();

    glob_pattern_matches(&pattern_rel, &rel_str)
}

fn glob_pattern_matches(pattern: &str, input: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == input,
        Some((prefix, suffix)) => {
            if !input.starts_with(prefix) {
                return false;
            }
            let rest = &input[prefix.len()..];
            if suffix.is_empty() {
                !rest.contains('/')
            } else {
                (!rest.contains('/') && rest.ends_with(suffix))
                    || rest.find('/').map_or(false, |i| {
                        rest[..i].ends_with(suffix.trim_start_matches('/'))
                    })
            }
        }
    }
}
