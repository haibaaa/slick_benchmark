#!/usr/bin/env python3
"""Download external datasets required by the string-key benchmarks."""
import urllib.request
from pathlib import Path

Path("data").mkdir(exist_ok=True)

# Norvig word-frequency list used by the `norvig` dataset loader.
url = "https://norvig.com/ngrams/count_1w.txt"
dest = "data/norvig_words.txt"
if not Path(dest).exists():
    print(f"Downloading {url}...")
    urllib.request.urlretrieve(url, dest)
    print(f"Saved to {dest}")
else:
    print(f"{dest} already exists, skipping.")

wiki_url = "https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-all-titles-in-ns0.gz"
wiki_dest = "data/wiki_titles.txt"
if not Path(wiki_dest).exists():
    import gzip
    import shutil

    # Download the compressed title dump and expand it into the format expected
    # by the Rust dataset loader.
    gz_dest = wiki_dest + ".gz"
    print("Downloading Wikipedia titles (this may be large)...")
    urllib.request.urlretrieve(wiki_url, gz_dest)
    with gzip.open(gz_dest, "rb") as f_in:
        with open(wiki_dest, "wb") as f_out:
            shutil.copyfileobj(f_in, f_out)
    Path(gz_dest).unlink()
    print(f"Saved to {wiki_dest}")
else:
    print(f"{wiki_dest} already exists, skipping.")
