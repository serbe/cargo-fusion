use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::manifest_utils::{is_workspace_manifest, parse_manifest};

pub struct WorkspaceResolution {
    /// Always the workspace-root `Cargo.toml` (or the original path if no workspace was found).
    pub manifest_path: PathBuf,

    /// Directory used as the search root for source files.
    ///
    /// - Crate mode  → the crate directory (e.g. `crates/my-crate`)
    /// - Workspace mode → the workspace root directory
    pub search_root: PathBuf,

    pub is_crate_mode: bool,
}

pub fn resolve_workspace_manifest(manifest_path: &Path) -> Result<WorkspaceResolution> {
    let canonical = manifest_path
        .canonicalize()
        .with_context(|| format!("cannot resolve: {}", manifest_path.display()))?;

    let crate_dir = canonical
        .parent()
        .context("Cargo.toml has no parent directory")?
        .to_path_buf();

    // If this manifest already declares [workspace], we are at the root.
    if is_workspace_manifest(&canonical)? {
        return Ok(WorkspaceResolution {
            manifest_path: canonical,
            search_root: crate_dir,
            is_crate_mode: false,
        });
    }

    // Otherwise walk up to find the workspace root that lists this crate as a member.
    let mut current = crate_dir.parent();
    while let Some(dir) = current {
        let candidate = dir.join("Cargo.toml");
        if candidate.exists() && workspace_lists_member(&candidate, &crate_dir)? {
            eprintln!(
                "info: workspace root at {}, collecting only crate at {}",
                candidate.display(),
                crate_dir.display(),
            );
            return Ok(WorkspaceResolution {
                manifest_path: candidate,
                search_root: crate_dir,
                is_crate_mode: true,
            });
        }
        current = dir.parent();
    }

    // No workspace found — treat the manifest as standalone.
    Ok(WorkspaceResolution {
        manifest_path: canonical,
        search_root: crate_dir,
        is_crate_mode: false,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns `true` when `candidate_manifest` is a workspace root that lists
/// `crate_dir` among its members.
fn workspace_lists_member(candidate_manifest: &Path, crate_dir: &Path) -> Result<bool> {
    let manifest = parse_manifest(candidate_manifest)
        .with_context(|| format!("failed to parse {}", candidate_manifest.display()))?;

    let Some(workspace) = &manifest.workspace else {
        return Ok(false);
    };

    let workspace_dir = candidate_manifest
        .parent()
        .context("workspace Cargo.toml has no parent")?;

    let members = match &workspace.members {
        members if members.is_empty() => {
            // Virtual workspace with no explicit members covers everything.
            return Ok(true);
        }
        members => members,
    };

    for pattern in members {
        let glob_pattern = workspace_dir.join(pattern);
        if glob_matches(&glob_pattern, crate_dir, workspace_dir) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Checks whether `pattern_path` (which may contain `*`) matches `crate_dir`.
fn glob_matches(pattern_path: &Path, crate_dir: &Path, workspace_dir: &Path) -> bool {
    let Some(pattern_str) = pattern_path.to_str() else {
        return false;
    };

    if !pattern_str.contains('*') {
        // Exact path — canonicalize both sides for a reliable comparison.
        return pattern_path
            .canonicalize()
            .is_ok_and(|cp| crate_dir == cp || crate_dir.starts_with(&cp));
    }

    // Glob: compare relative paths with forward slashes so the logic is
    // platform-independent.
    let rel = match crate_dir.strip_prefix(workspace_dir) {
        Ok(r) => r.to_string_lossy().replace('\\', "/"),
        Err(_) => return false,
    };

    let ws_str = workspace_dir.to_string_lossy().replace('\\', "/");
    let pattern_rel = pattern_str
        .replace('\\', "/")
        .trim_start_matches(ws_str.as_str())
        .trim_start_matches('/')
        .to_string();

    simple_glob_matches(&pattern_rel, &rel)
}

/// Minimal single-`*` glob matching for workspace member patterns like `crates/*`.
///
/// A `*` matches any sequence of characters that does **not** contain `/`,
/// which mirrors Cargo's own behaviour for workspace member globs.
fn simple_glob_matches(pattern: &str, input: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == input,
        Some((prefix, suffix)) => {
            if !input.starts_with(prefix) {
                return false;
            }
            let rest = &input[prefix.len()..];
            // `*` must not span directory boundaries.
            let segment = match rest.split_once('/') {
                Some((seg, _)) => seg,
                None => rest,
            };
            suffix.is_empty() || segment.ends_with(suffix.trim_start_matches('/'))
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::simple_glob_matches;

    #[test]
    fn glob_exact_segment() {
        assert!(simple_glob_matches("crates/*", "crates/foo"));
    }

    #[test]
    fn glob_does_not_cross_slash() {
        assert!(!simple_glob_matches("crates/*", "crates/foo/bar"));
    }

    #[test]
    fn glob_no_wildcard() {
        assert!(simple_glob_matches("crates/foo", "crates/foo"));
        assert!(!simple_glob_matches("crates/foo", "crates/bar"));
    }

    #[test]
    fn glob_suffix() {
        assert!(simple_glob_matches("*-core", "my-core"));
        assert!(!simple_glob_matches("*-core", "my-core/sub"));
    }
}
