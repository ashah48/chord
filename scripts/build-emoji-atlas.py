"""Build offline color emoji assets from the pinned Twemoji 17.0.3 archive.

Run in nix-shell -p python3Packages.pillow --run
'python scripts/build-emoji-atlas.py private/twemoji-17.0.3.tar.gz'.
Pass Unicode's emoji-test.txt as a second argument to regenerate the catalog.
"""
import io
import json
import math
from pathlib import Path
import sys
import tarfile
from PIL import Image

root = Path(__file__).resolve().parents[1] / 'assets'
with tarfile.open(sys.argv[1]) as archive:
    entries = sorted((m for m in archive.getmembers() if '/assets/72x72/' in m.name and m.name.endswith('.png')), key=lambda m: m.name)
    assert len(entries) > 3500
    size, columns = 32, 64
    atlas = Image.new('RGBA', (size * columns, size * math.ceil(len(entries) / columns)))
    index = {}
    for i, entry in enumerate(entries):
        image = Image.open(io.BytesIO(archive.extractfile(entry).read())).convert('RGBA')
        atlas.paste(image.resize((size, size), Image.Resampling.LANCZOS), ((i % columns) * size, (i // columns) * size))
        index[Path(entry.name).stem] = i
    atlas.save(root / 'emoji-atlas.png', optimize=True)
    (root / 'emoji-atlas.json').write_text(json.dumps(index, separators=(',', ':')) + '\n')
    license_entry = next(m for m in archive.getmembers() if m.name.endswith('/LICENSE-GRAPHICS'))
    (root / 'Twemoji-LICENSE.txt').write_bytes(archive.extractfile(license_entry).read())
    print(f'{len(entries)} emoji; atlas {atlas.size}')

if len(sys.argv) > 2:
    with (root / 'emoji-catalog.tsv').open('w', encoding='utf-8') as output:
        print('# Unicode Emoji 17.0 — https://www.unicode.org/Public/17.0.0/emoji/emoji-test.txt', file=output)
        print('# Copyright © 2025 Unicode, Inc. See Unicode-Emoji-LICENSE.txt.', file=output)
        for line in Path(sys.argv[2]).read_text(encoding='utf-8').splitlines():
            if line.startswith('# group: '):
                print(f'[{line[9:]}]', file=output)
            elif ';' in line and not line.startswith('#'):
                _, rest = line.split(';', 1)
                status, _, item = rest.partition('#')
                if status.strip() in ('fully-qualified', 'component'):
                    glyph, _, name = item.split(maxsplit=2)
                    print(glyph, name, sep='\t', file=output)
