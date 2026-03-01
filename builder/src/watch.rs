use anyhow::Result;
use notify::Watcher;
use std::path::Path;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc,
    Arc,
};
use std::time::{Duration, Instant};

use crate::config;
use crate::server;

/// Watch mode: monitors source files and rebuilds on changes
pub fn watch_mode(base_path: &Path) -> Result<()> {
    config::set_base_path(base_path);

    println!();
    println!("Wiki Builder - WATCH MODE");
    println!("Listening for file changes...");
    println!("Press Ctrl+C to stop");
    println!("Hint: Run with '--build' for one-time");
    println!("Server: http://127.0.0.1:8080/");
    println!();

    let build_counter = Arc::new(AtomicU64::new(0));
    server::start_server(base_path, Arc::clone(&build_counter))?;

    let (tx, rx) = mpsc::channel();

    // Create a debounced watcher - waits for file changes to settle
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if let Ok(event) = event {
            // Debug: print all incoming events and paths
            // if !event.paths.is_empty() {
            //     let paths: Vec<String> = event
            //         .paths
            //         .iter()
            //         .map(|p| p.to_string_lossy().replace('\\', "/"))
            //         .collect();
            //     println!("[watch] event: {:?} | paths: {}", event.kind, paths.join(", "));
            // } else {
            //     println!("[watch] event: {:?}", event.kind);
            // }
            // Filter out non-relevant events and emit on file operations
            match event.kind {
                notify::EventKind::Create(_)
                | notify::EventKind::Modify(_)
                | notify::EventKind::Remove(_) => {
                    // Watch the entire site directory (recursive)
                    let _ = tx.send(());
                }
                _ => {}
            }
        }
    })?;

    // Watch the site directory for changes
    let site_dir = base_path.join("site");
    if site_dir.exists() {
        watcher.watch(&site_dir, notify::RecursiveMode::Recursive)?;
    }

    println!("Watching for changes in site/\n");

    if let Err(err) = run_build(base_path) {
        println!("Initial build failed: {}\n", err);
        print_error_box();
    } else {
        build_counter.fetch_add(1, Ordering::SeqCst);
        println!("Initial build successful!\n");
    }

    let mut last_build = Instant::now();
    let debounce_duration = Duration::from_millis(600);

    loop {
        // Wait for file change events with debouncing
        if let Ok(()) = rx.recv() {
            // Debounce: wait for a quiet period to avoid duplicate rebuilds
            if last_build.elapsed() < debounce_duration {
                std::thread::sleep(debounce_duration);
            }

            // Keep waiting until no events arrive within the debounce window
            loop {
                match rx.recv_timeout(debounce_duration) {
                    Ok(()) => continue,
                    Err(mpsc::RecvTimeoutError::Timeout) => break,
                    Err(_) => break,
                }
            }

            last_build = Instant::now();
            println!("Changes detected...");

            // Run the build
            match run_build(base_path) {
                Ok(()) => {
                    build_counter.fetch_add(1, Ordering::SeqCst);
                    println!("Build successful!\n");
                }
                Err(e) => {
                    println!("Build failed: {}\n", e);
                    print_error_box();
                    // Continue watching instead of crashing
                }
            }
        }
    }
}

fn run_build(base_path: &Path) -> Result<()> {
    config::set_base_path(base_path);

    crate::output::copy_static_assets()?;
    crate::output::copy_root_files()?;

    // load first so that postprocess can resolve names
    let items = crate::data::load_items()?;
    let npcs = crate::data::load_npcs()?;
    let pages = crate::data::load_pages()?;

    crate::postprocess::init_lookup(&items, &npcs);

    let tera = crate::render::init_tera()?;

    crate::render::render_items(&tera, &items)?;
    crate::render::render_npcs(&tera, &npcs, &items)?;
    crate::render::render_regular_pages(&tera, &pages)?;
    crate::render::render_indexes(&tera, &items, &npcs, &pages)?;

    Ok(())
}

fn print_error_box() {
    println!("----------------------------------------");
    println!("The build has errors above");
    println!("Previous build remains intact");
    println!("Fix the issues and save files");
    println!("----------------------------------------");
    println!();
}
