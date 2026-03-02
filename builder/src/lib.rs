mod config;
mod data;
mod output;
mod postprocess;
mod render;
mod watch;
mod server;

use std::path::Path;

pub fn build_wiki(base_path: Option<&Path>) -> anyhow::Result<()> {
    if let Some(path) = base_path {
        config::set_base_path(path);
    }

    println!("Wiki Builder - Legends of Taria");
    println!("================================");

    output::copy_static_assets()?;
    output::copy_root_files()?;

    // load everything first so that our link filter can look up the correct
    // wiki_name / id from the JSON definitions
    let items = data::load_items()?;
    println!("Loaded {} items", items.len());

    let npcs = data::load_npcs()?;
    println!("Loaded {} npcs", npcs.len());

    // populate lookup tables early so that page loading (which calls
    // `linkify_references`) can resolve tokens correctly.  previously the
    // lookup was initialized after `load_pages`, causing markdown pages to
    // generate basic unfriendly links.
    postprocess::init_lookup(&items, &npcs);

    let pages = data::load_pages()?;
    println!("Loaded {} pages", pages.len());

    let tera = render::init_tera()?;

    render::render_items(&tera, &items)?;
    render::render_npcs(&tera, &npcs, &items)?;
    render::render_regular_pages(&tera, &pages)?;
    render::render_indexes(&tera, &items, &npcs, &pages)?;

    println!("================================");
    println!("Wiki generated successfully!");
    Ok(())
}

pub fn watch_mode(base_path: &Path) -> anyhow::Result<()> {
    watch::watch_mode(base_path)
}