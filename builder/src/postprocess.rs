use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Regex pattern to match item markup: <item name="item-id">display text</item>
static ITEM_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<item\s+name="([^"]+)"[^>]*>([^<]*)</item>"#).unwrap()
});

/// Regex pattern to match NPC markup: <npc name="npc-id">display text</npc>
static NPC_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<npc\s+name="([^"]+)"[^>]*>([^<]*)</npc>"#).unwrap()
});

/// Regex pattern to match shorthand item markup: <item:item-id>
static ITEM_SHORT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<item:([a-z0-9-]+)>"#).unwrap()
});

/// Regex pattern to match shorthand NPC markup: <npc:npc-id>
static NPC_SHORT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<npc:([a-z0-9-]+)>"#).unwrap()
});

/// Regex pattern to match anchors produced by pulldown-cmark autolink when the
/// source contained `<item:foo>` or `<npc:bar>`.  The markdown renderer
/// converts the token into an `<a href="item:foo">item:foo</a>` link, which
/// would otherwise pass through unchanged.  Catch it here and replace it with
/// the same rich HTML as the shorthand syntax.
static AUTOLINK_ITEM_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<a\s+href=\"item:([a-z0-9-]+)\">[^<]*</a>"#).unwrap()
});

static AUTOLINK_NPC_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<a\s+href=\"npc:([a-z0-9-]+)\">[^<]*</a>"#).unwrap()
});

/// Convert an id to a display name (e.g., "iron-ore" -> "Iron Ore")
fn id_to_display_name(id: &str) -> String {
    id.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Helper data used during link lookup once the JSON files have been loaded.
struct ItemInfo {
    id: u16,
    wiki_name: String,
}

struct NpcInfo {
    id: u16,
    wiki_name: String,
}

struct LookupData {
    items: HashMap<String, ItemInfo>,
    npcs: HashMap<String, NpcInfo>,
}

impl LookupData {
    fn new(items: &[crate::data::Item], npcs: &[crate::data::Npc]) -> Self {
        let mut map_items = HashMap::new();
        for item in items {
            let name_lower = item.name.to_lowercase().replace(" ", "-");
            map_items.insert(name_lower, ItemInfo { id: item.id, wiki_name: item.wiki_name.clone() });
            map_items.insert(item.wiki_name.clone(), ItemInfo { id: item.id, wiki_name: item.wiki_name.clone() });
            // also allow lookup by numeric id as string
            map_items.insert(item.id.to_string(), ItemInfo { id: item.id, wiki_name: item.wiki_name.clone() });
        }

        let mut map_npcs = HashMap::new();
        for npc in npcs {
            let name_lower = npc.name.to_lowercase().replace(" ", "-");
            map_npcs.insert(name_lower, NpcInfo { id: npc.id, wiki_name: npc.wiki_name.clone() });
            map_npcs.insert(npc.wiki_name.clone(), NpcInfo { id: npc.id, wiki_name: npc.wiki_name.clone() });
            map_npcs.insert(npc.id.to_string(), NpcInfo { id: npc.id, wiki_name: npc.wiki_name.clone() });
        }

        LookupData { items: map_items, npcs: map_npcs }
    }

    fn find_item(&self, key: &str) -> Option<&ItemInfo> {
        let key_lower = key.to_lowercase();
        if let Some(info) = self.items.get(&key_lower) {
            return Some(info);
        }
        // if the caller passed a slug without the numeric suffix (e.g.
        // "bronze-bar" while the wiki_name is "bronze-bar-14"), try a
        // simple prefix search.  This helps on markdown pages where authors
        // might omit the id since it's not visually meaningful.
        self.items
            .iter()
            .find_map(|(k, info)| {
                if k.starts_with(&format!("{}-", key_lower)) {
                    Some(info)
                } else {
                    None
                }
            })
    }

    fn find_npc(&self, key: &str) -> Option<&NpcInfo> {
        let key_lower = key.to_lowercase();
        if let Some(info) = self.npcs.get(&key_lower) {
            return Some(info);
        }
        self.npcs
            .iter()
            .find_map(|(k, info)| {
                if k.starts_with(&format!("{}-", key_lower)) {
                    Some(info)
                } else {
                    None
                }
            })
    }
}

use std::sync::OnceLock;
static LOOKUP: OnceLock<LookupData> = OnceLock::new();

/// Initialize the lookup tables with the currently loaded items/npcs.  This must be
/// called once before any templates are rendered (we do it very early in the
/// builder).  Subsequent calls are ignored.
pub fn init_lookup(items: &[crate::data::Item], npcs: &[crate::data::Npc]) {
    LOOKUP.get_or_init(|| LookupData::new(items, npcs));
}

/// Generate an item link with icon using *explicit* wiki name and numeric id.
fn item_link_with_id(wiki_name: &str, icon_id: u16, display_text: &str) -> String {
    format!(
        r#"<a href="/items/{}.html" class="item-link"><img src="/assets/images/items/{}.png" alt="{}" class="inline-icon" />{}</a>"#,
        wiki_name, icon_id, display_text, display_text
    )
}

/// Fallback from the previous behaviour when we can't resolve an item to a
/// definition.  It simply uses the supplied string for both the slug and the
/// icon path (which may of course be incorrect).
fn item_link_basic(slug: &str, display_text: &str) -> String {
    format!(
        r#"<a href="/items/{}.html" class="item-link"><img src="/assets/images/items/{}.png" alt="{}" class="inline-icon" />{}</a>"#,
        slug, slug, display_text, display_text
    )
}

/// Generate an NPC link when we know both slug and numeric id.
fn npc_link_with_id(wiki_name: &str, icon_id: u16, display_text: &str) -> String {
    format!(
        r#"<a href="/npcs/{}.html" class="npc-link"><img src="/assets/images/npcs/{}.png" alt="{}" class="inline-icon" />{}</a>"#,
        wiki_name, icon_id, display_text, display_text
    )
}

/// Fallback for NPCs when we don't have a matching definition.
fn npc_link_basic(slug: &str, display_text: &str) -> String {
    format!(
        r#"<a href="/npcs/{}.html" class="npc-link"><img src="/assets/images/npcs/{}.png" alt="{}" class="inline-icon" />{}</a>"#,
        slug, slug, display_text, display_text
    )
}

/// Post-process text to convert item and NPC markup into links with icons.
pub fn linkify_references(text: &str) -> String {
    let mut result = text.to_string();

    // grab lookup once to avoid locking each replacement
    let lookup_opt = LOOKUP.get();

    // Handle verbose item syntax: <item name="id">text</item>
    result = ITEM_PATTERN
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            let display_text = &caps[2];
            if let Some(lookup) = lookup_opt {
                if let Some(info) = lookup.find_item(key) {
                    return item_link_with_id(&info.wiki_name, info.id, display_text);
                }
            }
            // fallback to previous behaviour
            item_link_basic(key, display_text)
        })
        .to_string();

    // Handle verbose NPC syntax: <npc name="id">text</npc>
    result = NPC_PATTERN
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            let display_text = &caps[2];
            if let Some(lookup) = lookup_opt {
                if let Some(info) = lookup.find_npc(key) {
                    return npc_link_with_id(&info.wiki_name, info.id, display_text);
                }
            }
            npc_link_basic(key, display_text)
        })
        .to_string();

    // Handle shorthand item syntax: <item:id>
    result = ITEM_SHORT_PATTERN
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            let display_text = id_to_display_name(key);
            if let Some(lookup) = lookup_opt {
                if let Some(info) = lookup.find_item(key) {
                    return item_link_with_id(&info.wiki_name, info.id, &display_text);
                }
            }
            item_link_basic(key, &display_text)
        })
        .to_string();

    // Handle shorthand NPC syntax: <npc:id>
    result = NPC_SHORT_PATTERN
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            let display_text = id_to_display_name(key);
            if let Some(lookup) = lookup_opt {
                if let Some(info) = lookup.find_npc(key) {
                    return npc_link_with_id(&info.wiki_name, info.id, &display_text);
                }
            }
            npc_link_basic(key, &display_text)
        })
        .to_string();

    // Handle any autolink anchors that were generated by the markdown
    // processor for our `<item:foo>`/`<npc:bar>` tokens.  These look like
    // `<a href="item:foo">item:foo</a>` and would otherwise be left
    // untouched, resulting in a useless bare link.  Re-run the same logic as
    // for the shorthand syntax.
    result = AUTOLINK_ITEM_PATTERN
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            let display_text = id_to_display_name(key);
            if let Some(lookup) = lookup_opt {
                if let Some(info) = lookup.find_item(key) {
                    return item_link_with_id(&info.wiki_name, info.id, &display_text);
                }
            }
            item_link_basic(key, &display_text)
        })
        .to_string();

    result = AUTOLINK_NPC_PATTERN
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            let display_text = id_to_display_name(key);
            if let Some(lookup) = lookup_opt {
                if let Some(info) = lookup.find_npc(key) {
                    return npc_link_with_id(&info.wiki_name, info.id, &display_text);
                }
            }
            npc_link_basic(key, &display_text)
        })
        .to_string();

    result
}

/// Create a Tera filter function for linkifying
pub fn make_linkify_filter() -> impl tera::Filter {
    |value: &tera::Value, _args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        match value.as_str() {
            Some(s) => Ok(tera::Value::String(linkify_references(s))),
            None => Ok(value.clone()),
        }
    }
}

/// Generate an item type link
fn item_type_link(item_type: &str) -> String {
    let slug = item_type.to_lowercase().replace(' ', "-");
    format!(
        r#"<a href="/items/?type={}" class="type-link">{}</a>"#,
        slug, item_type
    )
}

/// Create a Tera filter function for linking item types
pub fn make_item_type_link_filter() -> impl tera::Filter {
    |value: &tera::Value, _args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        match value.as_str() {
            Some(s) => Ok(tera::Value::String(item_type_link(s))),
            None => Ok(value.clone()),
        }
    }
}
