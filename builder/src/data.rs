use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::config;

#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(alias = "type")]
    pub item_type: String,
    #[serde(default)]
    pub healing: Option<i32>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub stats: HashMap<String, i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirements: Option<HashMap<String, i32>>,
    #[serde(default)]
    pub acquisition: String,
    #[serde(default)]
    pub sell_price: Option<u64>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Npc {
    pub id: String,
    pub name: String,
    pub location: String,
    pub role: String,
    pub description: String,
    #[serde(default)]
    pub level: Option<i32>,
    #[serde(default)]
    pub hitpoints: Option<i32>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub stats: HashMap<String, i32>,
    #[serde(default)]
    pub drops: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Serialize)]
pub struct Page {
    pub slug: String,
    pub title: String,
    pub body_html: String,
}

/// Enriched drop information for rendering NPC drops with item links and prices
#[derive(Debug, Serialize)]
pub struct EnrichedDrop {
    pub item_id: String,
    pub item_name: String,
    pub item_type: String,
    pub sell_price: Option<u64>,
    pub link_html: String,
}

pub fn load_items() -> Result<Vec<Item>> {
    let mut items = Vec::new();
    let dir = config::data_dir().join("items");

    if !dir.exists() {
        return Ok(items);
    }

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let mut file =
            fs::File::open(&path).with_context(|| format!("failed to open item file {:?}", path))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        let item: Item = serde_json::from_str(&buf)
            .with_context(|| format!("failed to parse item JSON {:?}", path))?;
        items.push(item);
    }

    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

pub fn load_npcs() -> Result<Vec<Npc>> {
    let mut npcs = Vec::new();
    let dir = config::data_dir().join("npcs");

    if !dir.exists() {
        return Ok(npcs);
    }

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let mut file =
            fs::File::open(&path).with_context(|| format!("failed to open npc file {:?}", path))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        let npc: Npc = serde_json::from_str(&buf)
            .with_context(|| format!("failed to parse npc JSON {:?}", path))?;
        npcs.push(npc);
    }

    npcs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(npcs)
}

// Previously we limited page loading to a handful of subdirectories.  The
// user now wants *all* markdown files beneath the HTML output directory to be
// loaded, except those under the `assets` folder (images, javascript, etc.).

pub fn load_pages() -> Result<Vec<Page>> {
    let mut pages = Vec::new();
    let base = config::html_dir();

    if !base.exists() {
        return Ok(pages);
    }

    // walk the entire base directory recursively, but skip anything inside
    // "assets" (or a directory named "assets" anywhere in the path).
    for entry in WalkDir::new(&base)
        .into_iter()
        .filter_entry(|e| {
            // skip assets directories
            if e.file_type().is_dir() {
                if let Some(name) = e.file_name().to_str() {
                    return name != "assets";
                }
            }
            true
        })
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let mut file = fs::File::open(path)
            .with_context(|| format!("failed to open page file {:?}", path))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        let body_html = markdown_to_html(&buf);
        let slug = page_slug(&base, path)?;
        let title = derive_title_from_path(path);

        pages.push(Page {
            slug,
            title,
            body_html,
        });
    }

    pages.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(pages)
}

pub fn load_npc_notes(_id: &str) -> Result<String> {
    Ok(String::new())
}

pub fn markdown_to_html(src: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(src, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

fn page_slug(base: &Path, full: &Path) -> Result<String> {
    let rel = full
        .strip_prefix(base)
        .with_context(|| format!("failed to strip prefix {:?} from {:?}", base, full))?;

    let mut slug = PathBuf::new();
    for comp in rel.components() {
        if let std::path::Component::Normal(os) = comp {
            let part = os.to_string_lossy();
            let part = if let Some((stem, _)) = part.rsplit_once('.') {
                stem.to_string()
            } else {
                part.to_string()
            };
            slug.push(part);
        }
    }

    let slug_str = slug.to_string_lossy().replace('\\', "/");
    Ok(slug_str)
}

fn derive_title_from_path(path: &Path) -> String {
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Page");

    let cleaned = file_name.replace('_', " ").replace('-', " ");

    cleaned
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert a drop name to an enriched drop with item link and price
pub fn enrich_drop(drop_name: &str, items: &[Item]) -> EnrichedDrop {
    let drop_lower = drop_name.to_lowercase();

    if let Some(item) = items.iter().find(|i| i.name.to_lowercase() == drop_lower) {
        EnrichedDrop {
            item_id: item.id.clone(),
            item_name: item.name.clone(),
            item_type: item.item_type.clone(),
            sell_price: item.sell_price,
            link_html: format!(
                r#"<a href="/items/{}.html" class="item-link"><img src="/assets/images/items/{}.png" alt="{}" class="inline-icon" />{}</a>"#,
                item.id, item.id, item.name, item.name
            ),
        }
    } else {
        EnrichedDrop {
            item_id: String::new(),
            item_name: drop_name.to_string(),
            item_type: String::new(),
            sell_price: None,
            link_html: drop_name.to_string(),
        }
    }
}
