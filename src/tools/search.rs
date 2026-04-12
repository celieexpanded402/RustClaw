use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

const MAX_RESULTS: usize = 50;
const CONTEXT_LINES: usize = 2;

#[derive(Debug, Serialize)]
pub struct SearchMatch {
    pub file: String,
    pub line_number: usize,
    pub line: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

/// Search files in `dir` for `pattern` (substring match). Pure Rust, no external tools.
pub fn search_in_dir(dir: &str, pattern: &str) -> Result<Vec<SearchMatch>> {
    let mut matches = Vec::new();
    search_recursive(Path::new(dir), pattern, &mut matches)?;
    Ok(matches)
}

fn search_recursive(dir: &Path, pattern: &str, matches: &mut Vec<SearchMatch>) -> Result<()> {
    if matches.len() >= MAX_RESULTS {
        return Ok(());
    }

    let entries = std::fs::read_dir(dir).with_context(|| format!("Cannot read {}", dir.display()))?;

    for entry in entries.flatten() {
        if matches.len() >= MAX_RESULTS {
            break;
        }

        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip hidden, target, node_modules
        if name_str.starts_with('.') || name_str == "target" || name_str == "node_modules" {
            continue;
        }

        if path.is_dir() {
            search_recursive(&path, pattern, matches)?;
        } else if path.is_file() {
            // Skip binary files (check extension)
            if is_binary_extension(&path) {
                continue;
            }
            search_file(&path, pattern, matches);
        }
    }

    Ok(())
}

fn search_file(path: &Path, pattern: &str, matches: &mut Vec<SearchMatch>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return, // skip unreadable files
    };

    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if matches.len() >= MAX_RESULTS {
            break;
        }

        if line.contains(pattern) {
            let context_before: Vec<String> = lines
                [i.saturating_sub(CONTEXT_LINES)..i]
                .iter()
                .map(|s| s.to_string())
                .collect();

            let context_after: Vec<String> = lines
                .get(i + 1..=(i + CONTEXT_LINES).min(lines.len().saturating_sub(1)))
                .unwrap_or(&[])
                .iter()
                .map(|s| s.to_string())
                .collect();

            matches.push(SearchMatch {
                file: path.to_string_lossy().to_string(),
                line_number: i + 1,
                line: line.to_string(),
                context_before,
                context_after,
            });
        }
    }
}

fn is_binary_extension(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "svg"
            | "woff" | "woff2" | "ttf" | "eot"
            | "zip" | "tar" | "gz" | "bz2" | "xz" | "7z"
            | "exe" | "dll" | "so" | "dylib" | "o" | "a"
            | "pdf" | "doc" | "docx" | "xls" | "xlsx"
            | "mp3" | "mp4" | "avi" | "mov" | "wav"
            | "wasm" | "pyc" | "class"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn binary_extensions_detected() {
        for ext in &["png", "jpg", "zip", "exe", "pdf", "mp4", "wasm"] {
            let name = format!("file.{ext}");
            assert!(is_binary_extension(Path::new(&name)), "{ext} should be binary");
        }
    }

    #[test]
    fn text_extensions_not_binary() {
        for ext in &["rs", "txt", "js", "py", "toml", "md", "html", "css"] {
            let name = format!("file.{ext}");
            assert!(!is_binary_extension(Path::new(&name)), "{ext} should not be binary");
        }
    }

    #[test]
    fn no_extension_not_binary() {
        assert!(!is_binary_extension(Path::new("Makefile")));
    }

    #[test]
    fn case_insensitive() {
        assert!(is_binary_extension(Path::new("image.PNG")));
        assert!(is_binary_extension(Path::new("archive.ZIP")));
    }
}
