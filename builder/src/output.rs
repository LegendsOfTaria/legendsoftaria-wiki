use std::fs;

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::config;

/// Copy static assets (CSS, images) from html/assets to out/assets
pub fn copy_static_assets() -> Result<()> {
    let src_assets = config::html_dir().join("assets");
    let dest_assets = config::output_dir().join("assets");

    if !src_assets.exists() {
        println!("No assets directory found, skipping asset copy");
        return Ok(());
    }

    let mut count = 0;

    for entry in WalkDir::new(&src_assets) {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let rel = path.strip_prefix(&src_assets).unwrap();
        let dest_path = dest_assets.join(rel);

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(path, &dest_path).with_context(|| format!("failed to copy {:?}", path))?;
        count += 1;
    }

    println!("Copied {} static assets", count);
    Ok(())
}

/// Copy robots.txt and sitemap.txt from html/ to out/
pub fn copy_root_files() -> Result<()> {
    let root_files = ["robots.txt", "sitemap.txt"];
    let mut count = 0;

    for filename in &root_files {
        let src = config::html_dir().join(filename);
        let dest = config::output_dir().join(filename);

        if src.exists() {
            fs::create_dir_all(config::output_dir())?;
            fs::copy(&src, &dest).with_context(|| format!("failed to copy {:?}", src))?;
            count += 1;
        }
    }

    println!("Copied {} root files (robots.txt, sitemap.txt)", count);
    Ok(())
}
