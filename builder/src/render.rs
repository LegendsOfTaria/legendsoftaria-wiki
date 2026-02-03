use std::fs;

use anyhow::{Context, Result};
use tera::{Context as TeraContext, Tera};

use crate::config;
use crate::data::{enrich_drop, Item, Npc, Page};
use crate::postprocess;

pub fn init_tera() -> Result<Tera> {
    let templates_dir = config::templates_dir();
    let glob = format!("{}/**/*.html", templates_dir.display());
    let mut tera = Tera::new(&glob).context("failed to initialize Tera templates")?;

    // Load index.html from the source html folder (child template that extends base.html)
    let index_path = config::html_dir().join("index.html");
    let index_content =
        fs::read_to_string(&index_path).context("failed to read index.html from html folder")?;
    tera.add_raw_template("index.html", &index_content)
        .context("failed to add index.html template")?;

    tera.register_filter("linkify", postprocess::make_linkify_filter());
    tera.register_filter("type_link", postprocess::make_item_type_link_filter());

    Ok(tera)
}

pub fn render_items(tera: &Tera, items: &[Item]) -> Result<()> {
    let base = config::output_dir().join("items");
    fs::create_dir_all(&base)?;

    for item in items {
        let mut ctx = TeraContext::new();
        ctx.insert("item", item);

        let html = tera
            .render("item.html", &ctx)
            .with_context(|| format!("failed to render item {}", item.id))?;

        let out_path = base.join(format!("{}.html", item.id));
        fs::write(&out_path, html)
            .with_context(|| format!("failed to write item page {:?}", out_path))?;

        println!("  → items/{}.html", item.id);
    }

    Ok(())
}

pub fn render_npcs(tera: &Tera, npcs: &[Npc], items: &[Item]) -> Result<()> {
    let base = config::output_dir().join("npcs");
    fs::create_dir_all(&base)?;

    for npc in npcs {
        let notes_html = crate::data::load_npc_notes(&npc.id)?;

        let enriched_drops: Vec<_> = npc
            .drops
            .iter()
            .map(|drop_name| enrich_drop(drop_name, items))
            .collect();

        let mut ctx = TeraContext::new();
        ctx.insert("npc", npc);
        ctx.insert("notes_html", &notes_html);
        ctx.insert("enriched_drops", &enriched_drops);

        let html = tera
            .render("npc.html", &ctx)
            .with_context(|| format!("failed to render npc {}", npc.id))?;

        let out_path = base.join(format!("{}.html", npc.id));
        fs::write(&out_path, html)
            .with_context(|| format!("failed to write npc page {:?}", out_path))?;

        println!("  → npcs/{}.html", npc.id);
    }

    Ok(())
}

pub fn render_regular_pages(tera: &Tera, pages: &[Page]) -> Result<()> {
    for page in pages {
        if page.slug.starts_with("npcs/") {
            continue;
        }

        let mut ctx = TeraContext::new();
        ctx.insert("title", &page.title);
        ctx.insert("body_html", &page.body_html);

        let html = tera
            .render("page.html", &ctx)
            .with_context(|| format!("failed to render page {}", page.slug))?;

        let out_path = config::output_dir()
            .join(&page.slug)
            .with_extension("html");

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&out_path, html)
            .with_context(|| format!("failed to write page {:?}", out_path))?;

        println!("  → {}.html", page.slug);
    }

    Ok(())
}

pub fn render_indexes(tera: &Tera, items: &[Item], npcs: &[Npc], pages: &[Page]) -> Result<()> {
    let regular_pages: Vec<&Page> = pages
        .iter()
        .filter(|p| !p.slug.starts_with("npcs/"))
        .collect();

    let mut ctx = TeraContext::new();
    ctx.insert("items", items);
    ctx.insert("npcs", npcs);
    ctx.insert("pages", &regular_pages);

    // Render root index.html (child template living in html/ folder)
    let html = tera
        .render("index.html", &ctx)
        .context("failed to render root index")?;
    let out_path = config::output_dir().join("index.html");
    fs::write(&out_path, html).with_context(|| format!("failed to write index {:?}", out_path))?;
    println!("  → index.html");

    let items_html = tera
        .render("items_index.html", &ctx)
        .context("failed to render items index")?;
    let items_index_path = config::output_dir().join("items").join("index.html");
    if let Some(parent) = items_index_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&items_index_path, items_html)
        .with_context(|| format!("failed to write items index {:?}", items_index_path))?;
    println!("  → items/index.html");

    let npcs_html = tera
        .render("npcs_index.html", &ctx)
        .context("failed to render npcs index")?;
    let npcs_index_path = config::output_dir().join("npcs").join("index.html");
    if let Some(parent) = npcs_index_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&npcs_index_path, npcs_html)
        .with_context(|| format!("failed to write npcs index {:?}", npcs_index_path))?;
    println!("  → npcs/index.html");

    Ok(())
}
