use std::path::Path;

use anyhow::{Context, Result};

const MAX_READ_BYTES: u64 = 100 * 1024;
const MAX_READ_LINES: usize = 500;
const MAX_TREE_DEPTH: u8 = 3;

/// Read a file. If > 100KB, return only the first 500 lines.
pub fn read_file(path: &str) -> Result<String> {
    let p = Path::new(path);
    let meta = std::fs::metadata(p).with_context(|| format!("Cannot stat {path}"))?;

    if meta.len() > MAX_READ_BYTES {
        let content = std::fs::read_to_string(p).with_context(|| format!("Cannot read {path}"))?;
        let truncated: String = content
            .lines()
            .take(MAX_READ_LINES)
            .collect::<Vec<_>>()
            .join("\n");
        Ok(format!(
            "{truncated}\n\n... (file truncated at {MAX_READ_LINES} lines, total {} bytes)",
            meta.len()
        ))
    } else {
        std::fs::read_to_string(p).with_context(|| format!("Cannot read {path}"))
    }
}

/// Write content to a file, creating parent directories as needed.
pub fn write_file(path: &str, content: &str) -> Result<()> {
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Cannot create dirs for {path}"))?;
    }
    std::fs::write(p, content).with_context(|| format!("Cannot write {path}"))
}

/// List directory in tree format, up to `depth` levels (max 3).
pub fn list_dir(path: &str, depth: u8) -> Result<String> {
    let depth = depth.min(MAX_TREE_DEPTH);
    let mut out = String::new();
    list_dir_recursive(Path::new(path), "", depth, &mut out)?;
    Ok(out)
}

fn list_dir_recursive(dir: &Path, prefix: &str, depth: u8, out: &mut String) -> Result<()> {
    if depth == 0 {
        return Ok(());
    }

    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("Cannot read dir {}", dir.display()))?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let count = entries.len();
    for (i, entry) in entries.into_iter().enumerate() {
        let is_last = i == count - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip hidden dirs and common noise
        if name_str.starts_with('.') || name_str == "target" || name_str == "node_modules" {
            continue;
        }

        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let suffix = if is_dir { "/" } else { "" };
        out.push_str(&format!("{prefix}{connector}{name_str}{suffix}\n"));

        if is_dir {
            let next_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
            list_dir_recursive(&entry.path(), &next_prefix, depth - 1, out)?;
        }
    }

    Ok(())
}

/// Replace the first occurrence of `old` with `new` in a file.
pub fn patch_file(path: &str, old: &str, new: &str) -> Result<()> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("Cannot read {path}"))?;

    if !content.contains(old) {
        anyhow::bail!("patch_file: pattern not found in {path}");
    }

    let patched = content.replacen(old, new, 1);
    std::fs::write(path, patched).with_context(|| format!("Cannot write {path}"))
}
