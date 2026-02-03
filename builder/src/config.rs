use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static BASE_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Data directory containing JSON definitions for items, NPCs, etc.
pub const DATA_DIR: &str = "site/data";

/// Templates directory containing Tera HTML templates
pub const TEMPLATES_DIR: &str = "site/templates";

/// Source HTML directory (contains index.html template and markdown pages)
pub const HTML_DIR: &str = "site/html";

/// Output directory for generated HTML pages
pub const OUTPUT_DIR: &str = "out";

pub fn set_base_path(path: &Path) {
    let _ = BASE_PATH.set(path.to_path_buf());
}

fn resolve_path(relative: &str) -> PathBuf {
    match BASE_PATH.get() {
        Some(base) => base.join(relative),
        None => PathBuf::from(format!("../{}", relative)),
    }
}

pub fn data_dir() -> PathBuf {
    resolve_path(DATA_DIR)
}

pub fn templates_dir() -> PathBuf {
    resolve_path(TEMPLATES_DIR)
}

pub fn html_dir() -> PathBuf {
    resolve_path(HTML_DIR)
}

pub fn output_dir() -> PathBuf {
    resolve_path(OUTPUT_DIR)
}
