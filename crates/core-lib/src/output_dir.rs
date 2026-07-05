use crate::error::{AgError, AgResult};
use std::env;
use std::path::{Path, PathBuf};

const EXPORT_DIR_NAME: &str = "augmented-gaussian";

pub fn resolve_output_dir(
    input_path: impl AsRef<Path>,
    chosen_dir: impl AsRef<Path>,
    now_millis: u128,
) -> AgResult<PathBuf> {
    let input_path = input_path.as_ref();
    let chosen_raw = chosen_dir.as_ref();
    let default_root = default_export_root()?;
    let chosen = if chosen_raw.as_os_str().is_empty() {
        default_root.clone()
    } else {
        expand_tilde(chosen_raw)?
    };

    let should_create_child =
        chosen_raw.as_os_str().is_empty() || is_export_root(&chosen, &default_root);
    if !should_create_child {
        return Ok(chosen);
    }

    let base = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("scene");
    let child_base = format!("{}_{}", sanitize_file_stem(base), now_millis);
    let mut candidate = chosen.join(&child_base);
    let mut suffix = 1u32;
    while candidate.exists() {
        candidate = chosen.join(format!("{child_base}_{suffix}"));
        suffix += 1;
    }
    Ok(candidate)
}

pub fn default_export_root() -> AgResult<PathBuf> {
    Ok(home_dir()?.join("Downloads").join(EXPORT_DIR_NAME))
}

pub fn expand_tilde(path: &Path) -> AgResult<PathBuf> {
    let raw = path.to_string_lossy();
    if raw == "~" {
        return Ok(home_dir()?);
    }
    if let Some(rest) = raw.strip_prefix("~/").or_else(|| raw.strip_prefix("~\\")) {
        return Ok(home_dir()?.join(rest));
    }
    Ok(path.to_path_buf())
}

fn home_dir() -> AgResult<PathBuf> {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| AgError::InvalidConfig("home directory is not available".to_string()))
}

fn normalize_path(path: &Path) -> String {
    path.components()
        .collect::<PathBuf>()
        .to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

fn is_export_root(path: &Path, default_root: &Path) -> bool {
    let normalized = normalize_path(path);
    normalized == normalize_path(default_root)
        || normalized.ends_with("/downloads/augmented-gaussian")
}

fn sanitize_file_stem(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "scene".to_string()
    } else {
        sanitized
    }
}
