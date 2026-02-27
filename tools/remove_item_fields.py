import json
from pathlib import Path

ITEMS_DIR = Path('site/data/items')
TARGET_KEYS = ['armor', 'level_requirement', 'damage']

if not ITEMS_DIR.exists():
    print(f"Directory not found: {ITEMS_DIR}")
    raise SystemExit(1)

modified = 0
for path in sorted(ITEMS_DIR.glob('*.json')):
    try:
        text = path.read_text(encoding='utf-8')
        data = json.loads(text)
    except Exception as e:
        print(f"Skipping {path}: failed to parse JSON ({e})")
        continue

    if not isinstance(data, dict):
        continue

    removed_any = False
    for k in TARGET_KEYS:
        if k in data:
            del data[k]
            removed_any = True

    if removed_any:
        # write compact pretty JSON without leaving extra blank lines
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding='utf-8')
        modified += 1
        print(f"Updated {path}")

print(f"Done. Modified {modified} file(s).")
