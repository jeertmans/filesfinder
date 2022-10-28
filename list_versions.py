"""
This script prints all occurences of filesfinder versions
"""

import re
from pathlib import Path

PATTERN_1 = re.compile('filesfinder"\nversion = "(.*)"')
PATTERN_2 = re.compile("filesfinder[:@]v(.*)")

PATTERNS = [PATTERN_1, PATTERN_2]
GLOBS = ["*.rs", "*.toml", "*.yml", "*.lock", "*.md", "Dockerfile"]

if __name__ == "__main__":
    for glob in GLOBS:
        for file in Path(".").glob(f"**/{glob}"):
            with file.open() as f:
                try:
                    text = f.read()

                    for pattern in PATTERNS:
                        for m in pattern.finditer(text):
                            print(f"v{m[1]}")

                except Exception as e:
                    raise Exception(f"Some exeption has occurend when reading file {file}: {str(e)}")
