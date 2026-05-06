use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Читает файл и возвращает его содержимое в виде байтов
///
/// # Errors
/// Возвращает ошибку если файл не может быть прочитан
pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))
}

/// Читает файл и возвращает его содержимое в виде строки
///
/// # Errors
/// Возвращает ошибку если файл не может быть прочитан или содержит невалидный UTF-8
pub fn read_file_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("Failed to read file as string: {}", path.display()))
}

/// Записывает байтовое содержимое в файл
///
/// # Errors
/// Возвращает ошибку если файл не может быть создан или записан
pub fn write_file(path: &Path, content: &[u8]) -> Result<()> {
    fs::write(path, content).with_context(|| format!("Failed to write file: {}", path.display()))
}

/// Создает файл для записи, создавая родительские директории при необходимости
///
/// # Errors
/// Возвращает ошибку если не удается создать директории или файл
pub fn create_file_with_dirs(path: &Path) -> Result<fs::File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create parent directories for: {}",
                parent.display()
            )
        })?;
    }
    fs::File::create(path).with_context(|| format!("Failed to create file: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_read_write_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let content = b"Hello, World!";
        write_file(&file_path, content).unwrap();

        let read_content = read_file(&file_path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_create_file_with_dirs() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("subdir").join("test.txt");

        let file = create_file_with_dirs(&file_path).unwrap();
        assert!(file_path.exists());
    }
}
