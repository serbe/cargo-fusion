//! Утилиты для работы с Cargo.toml манифестами
//!
//! Предоставляет унифицированные функции для парсинга и анализа
//! Cargo.toml файлов с кэшированием результатов.

use anyhow::{Context, Result};
use cargo_toml::{Manifest, Package};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Кэш для распарсенных манифестов
static MANIFEST_CACHE: Lazy<Mutex<HashMap<PathBuf, Manifest>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Парсит Cargo.toml файл с кэшированием результатов
///
/// # Errors
/// Возвращает ошибку если файл не может быть прочитан или содержит невалидный TOML
pub fn parse_manifest(path: &Path) -> Result<Manifest> {
    // Проверяем кэш
    if let Some(cached) = MANIFEST_CACHE.lock().unwrap().get(path) {
        return Ok(cached.clone());
    }

    // Парсим манифест
    let manifest = Manifest::from_path(path)
        .with_context(|| format!("Failed to parse Cargo.toml at {}", path.display()))?;

    // Сохраняем в кэш
    MANIFEST_CACHE
        .lock()
        .unwrap()
        .insert(path.to_path_buf(), manifest.clone());

    Ok(manifest)
}

/// Получает секцию [package] из манифеста
///
/// # Errors
/// Возвращает ошибку если секция package отсутствует
pub fn get_package(manifest: &Manifest) -> Result<&Package> {
    manifest
        .package
        .as_ref()
        .with_context(|| "No [package] section found in Cargo.toml")
}

/// Проверяет является ли манифест корнем workspace (имеет секцию [workspace])
pub fn is_workspace_root(manifest: &Manifest) -> bool {
    manifest.workspace.is_some()
}

/// Проверяет является ли файл манифестом workspace
pub fn is_workspace_manifest(path: &Path) -> Result<bool> {
    let manifest = parse_manifest(path)?;
    Ok(is_workspace_root(&manifest))
}

/// Получает список членов workspace
pub fn get_workspace_members(manifest: &Manifest) -> Vec<String> {
    manifest
        .workspace
        .as_ref()
        .map(|ws| ws.members.clone())
        .unwrap_or_default()
}

/// Получает информацию о зависимостях, которые указаны с path
#[derive(Debug, Clone)]
pub struct PathDependency {
    pub name: String,
    pub path: PathBuf,
    pub optional: bool,
}

/// Собирает все зависимости, указанные через `path = "..."`
pub fn collect_path_dependencies(
    manifest: &Manifest,
    manifest_dir: &Path,
) -> Result<Vec<PathDependency>> {
    let mut deps = Vec::new();

    for (name, dep) in &manifest.dependencies {
        let detail = match dep.detail() {
            Some(detail) => detail,
            None => continue,
        };

        let Some(rel_path) = &detail.path else {
            continue;
        };

        let absolute_path = manifest_dir.join(rel_path);
        deps.push(PathDependency {
            name: name.clone(),
            path: absolute_path,
            optional: detail.optional,
        });
    }

    Ok(deps)
}

/// Рекурсивно собирает все path-зависимости
pub fn collect_all_path_dependencies(
    manifest_path: &Path,
    visited: &mut HashMap<PathBuf, bool>,
) -> Result<Vec<PathBuf>> {
    let manifest_dir = manifest_path
        .parent()
        .context("Cargo.toml has no parent directory")?;

    let manifest = parse_manifest(manifest_path)?;
    let mut result = Vec::new();

    for dep in collect_path_dependencies(&manifest, manifest_dir)? {
        // Проверяем не посещали ли уже эту зависимость
        if visited.contains_key(&dep.path) {
            continue;
        }

        visited.insert(dep.path.clone(), true);
        result.push(dep.path.clone());

        // Рекурсивно обрабатываем зависимость
        let dep_manifest = dep.path.join("Cargo.toml");
        if dep_manifest.exists() {
            let sub_deps = collect_all_path_dependencies(&dep_manifest, visited)?;
            result.extend(sub_deps);
        }
    }

    Ok(result)
}

/// Получает имя пакета из манифеста
pub fn get_package_name(manifest: &Manifest) -> Result<String> {
    let package = get_package(manifest)?;
    Ok(package.name.clone())
}

/// Получает версию пакета из манифеста
pub fn get_package_version(manifest: &Manifest) -> Result<String> {
    let package = get_package(manifest)?;
    Ok(package.version().to_string())
}

/// Получает описание пакета
pub fn get_package_description(manifest: &Manifest) -> Option<String> {
    manifest
        .package
        .as_ref()
        .and_then(|p| p.description().map(ToString::to_string))
}

/// Получает авторов пакета
pub fn get_package_authors(manifest: &Manifest) -> Vec<String> {
    manifest
        .package
        .as_ref()
        .map(|p| p.authors().to_vec())
        .unwrap_or_default()
}

/// Получает лицензию пакета
pub fn get_package_license(manifest: &Manifest) -> Option<String> {
    manifest
        .package
        .as_ref()
        .and_then(|p| p.license().map(ToString::to_string))
}

/// Получает репозиторий пакета
pub fn get_package_repository(manifest: &Manifest) -> Option<String> {
    manifest
        .package
        .as_ref()
        .and_then(|p| p.repository().map(ToString::to_string))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_manifest(dir: &Path, content: &str) -> PathBuf {
        let manifest_path = dir.join("Cargo.toml");
        let mut file = fs::File::create(&manifest_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        manifest_path
    }

    #[test]
    fn test_parse_manifest() {
        let dir = tempdir().unwrap();
        let manifest_path = create_test_manifest(
            dir.path(),
            r#"
            [package]
            name = "test-package"
            version = "0.1.0"
        "#,
        );

        let manifest = parse_manifest(&manifest_path).unwrap();
        let package = get_package(&manifest).unwrap();
        assert_eq!(package.name, "test-package");
    }

    #[test]
    fn test_is_workspace_root() {
        let dir = tempdir().unwrap();
        let manifest_path = create_test_manifest(
            dir.path(),
            r#"
            [workspace]
            members = ["crates/*"]
        "#,
        );

        let manifest = parse_manifest(&manifest_path).unwrap();
        assert!(is_workspace_root(&manifest));
    }

    #[test]
    fn test_collect_path_dependencies() {
        let dir = tempdir().unwrap();
        let deps_dir = dir.path().join("deps");
        fs::create_dir(&deps_dir).unwrap();

        let manifest_path = create_test_manifest(
            dir.path(),
            &format!(
                r#"
            [package]
            name = "test"
            version = "0.1.0"

            [dependencies]
            my-dep = {{ path = "{}" }}
        "#,
                deps_dir.display()
            ),
        );

        let manifest = parse_manifest(&manifest_path).unwrap();
        let deps = collect_path_dependencies(&manifest, dir.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "my-dep");
        assert_eq!(deps[0].path, deps_dir);
    }

    #[test]
    fn test_cache_works() {
        let dir = tempdir().unwrap();
        let manifest_path = create_test_manifest(
            dir.path(),
            r#"
            [package]
            name = "cached-test"
            version = "0.1.0"
        "#,
        );

        // Первый парсинг
        let manifest1 = parse_manifest(&manifest_path).unwrap();
        let name1 = get_package_name(&manifest1).unwrap();

        // Второй парсинг (должен взять из кэша)
        let manifest2 = parse_manifest(&manifest_path).unwrap();
        let name2 = get_package_name(&manifest2).unwrap();

        assert_eq!(name1, name2);
    }
}
