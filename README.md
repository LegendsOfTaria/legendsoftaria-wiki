# Legends of Taria Wiki Builder

A fast, safe wiki builder for managing game content. Automatically generates HTML from JSON data and Tera templates.

## Quick Start

**Build once:**
```bash
./build.sh          # Linux/macOS
build.bat           # Windows
```

**Watch mode (auto-rebuild):**
```bash
./build-watch.sh    # Linux/macOS
build-watch.bat     # Windows
```

Press `Ctrl+C` to stop.
Watch mode also starts a local server at http://127.0.0.1:8080/ with live reload.

## Making Changes

### Add an item
1. Create file: `site/data/items/my-item.json`
2. (If using watch mode, wiki rebuilds automatically)
3. Output: `out/items/my-item.html`

### Add a page
1. Create file: `site/html/my-page.md` (Markdown format)
2. Output: `out/my-page.html`

### Edit templates
1. Modify: `site/templates/*.html`
2. All affected pages rebuild

### Edit JSON structure
Edit your JSON files in `site/data/`. Structure:
```json
{
  "id": "item-name",
  "name": "Display Name",
  "description": "Item description",
  "properties": { ... }
}
```

## How It Works

**Build directories:**
- `site/data/` - JSON definitions (items, npcs)
- `site/html/` - Pages and markdown content
- `site/templates/` - Tera HTML templates
- `out/` - Generated output (don't edit)

**Build process:**
1. Copy static assets (`site/html/assets/`)
2. Load all data from JSON files
3. Render pages using templates
4. Output to `out/`

## Build Modes

| Command | What it does | When to use |
|---------|------------|-----------|
| `build.sh` / `build.bat` | Build once | CI/CD, production deployment |
| `build-watch.sh` / `build-watch.bat` | Watch files & auto-rebuild | Development, testing changes |

**Note:** Both scripts pass arguments to the builder:
- `build.sh --watch` = same as `build-watch.sh`
- `build.bat --build` = one-time build
- `build.sh --help` = show all options

## Error Handling

- ❌ Build errors don't crash the process
- 🛡️ Previous working build stays intact until new build succeeds
- 📋 Detailed error messages show what to fix

## File Locations

```
site/
  ├─ data/
  │  ├─ items/        (*.json)
  │  └─ npcs/         (*.json)
  ├─ html/
  │  ├─ assets/       (CSS, images)
  │  ├─ index.html
  │  ├─ robots.txt
  │  └─ *.md          (markdown pages)
  └─ templates/       (*.html)

out/                   (generated - do not edit)
```

## Requirements

- Rust 1.70+ (installs automatically if missing, or you can install latest version here: https://rust-lang.org/tools/install/)
- Linux, macOS, or Windows

## CLI Options

```bash
./build.sh --build      # One-time build (default)
./build.sh --watch      # Watch mode
./build.sh --help       # Show help
```