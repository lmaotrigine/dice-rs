#!/usr/bin/env python3

from __future__ import annotations

import pathlib
import re
import sys
from itertools import takewhile

ROOT = pathlib.Path(__file__).parent
LIB_RS = ROOT / 'src' / 'lib.rs'
README = ROOT / 'README.md'
INTRADOC_LINK = re.compile(r'\[(`[^`]+`)\](?:\[[^]]+\])?')


def generate_readme() -> str:
    with LIB_RS.open('r', encoding='utf-8') as fp:
        return ''.join(
            # strip `//! `, clean intra-doc links
            INTRADOC_LINK.sub(lambda m: m.group(1), line[4:] or '\n')
            # only from the crate level documentation
            for line in takewhile(lambda line: line.startswith('//!'), fp)
            # skip rust codeblock comments
            if not line.startswith(('//! # use', '//! # fn', '//! # Ok', '//! # }'))
        )


if __name__ == '__main__':
    readme = generate_readme()
    try:
        check = sys.argv[1] == 'check'
    except IndexError:
        check = False
    if check:
        sys.exit(not README.exists() or readme != README.read_text(encoding='utf-8'))
    else:
        README.write_text(readme, encoding='utf-8', newline='\n')
