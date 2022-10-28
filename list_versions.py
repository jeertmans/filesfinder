"""
This script prints all occurences of filesfinder versions
"""

import re
from pathlib import Path

PATTERN_1 = re.compile('filesfinder"\nversion = "(.*)"')
PATTERN_2 = re.compile("filesfinder[:@]v(.*)")

PATTERNS = [PATTERN_1, PATTERN_2]
EXTS = ["rs", "toml", "yml", "lock"]

if __name__ == "__main__":
    for ext in EXTS:
        for file in Path(".").glob(f"**/*.{ext}"):
            with file.open() as f:
                text = f.read()

                for pattern in PATTERNS:
                    for m in pattern.finditer(text):
                        print(m[1])
