#!/usr/bin/env python3
"""Conservative cleanup/checker for generated Markdown/code output.

Does not repair semantics. By default reports suspicious truncation/elision markers;
--clean only normalizes line endings and strips trailing whitespace.
"""
from __future__ import annotations
import argparse, re, sys
from pathlib import Path

PATTERNS = {
    'elision placeholder': re.compile(r'(?im)^\s*(?://|#|\*|-)\s*(?:\.\.\.|…|existing code|rest of the code|omitted|same as above)\b'),
    'unfinished code fence': None,
    'likely truncation': re.compile(r'(?i)(?:\bto be continued\b|\bcontinued in the next response\b|\[truncated\])'),
}

def inspect(text: str) -> list[str]:
    found=[]
    for label, pattern in PATTERNS.items():
        if pattern and pattern.search(text): found.append(label)
    fences=len(re.findall(r'(?m)^\s*(```|~~~)', text))
    if fences % 2: found.append('unbalanced fenced-code delimiter')
    if text and not text.endswith('\n'): found.append('missing final newline (not necessarily an error)')
    return found

def main() -> int:
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('file', nargs='?', help='input file; defaults to stdin')
    p.add_argument('--clean', action='store_true', help='normalize CRLF and trailing whitespace')
    p.add_argument('-o','--output', help='cleaned output path (default stdout when --clean)')
    a=p.parse_args()
    text=Path(a.file).read_text(encoding='utf-8') if a.file else sys.stdin.read()
    if a.clean:
        text='\n'.join(line.rstrip() for line in text.replace('\r\n','\n').replace('\r','\n').split('\n'))
        if text and not text.endswith('\n'): text+='\n'
        if a.output: Path(a.output).write_text(text,encoding='utf-8')
        else: sys.stdout.write(text)
    issues=inspect(text)
    if issues:
        print('Review flags: ' + ', '.join(issues), file=sys.stderr)
        return 1
    print('No configured structural flags found.', file=sys.stderr)
    return 0

if __name__ == '__main__': raise SystemExit(main())
