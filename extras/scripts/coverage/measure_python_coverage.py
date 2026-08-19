#!/usr/bin/env python3
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

"""Measure line coverage of a Python script by tracing a run.

Runs `python3 -m trace --count --coverdir` on the target script and parses
the resulting `.cover` file. The set of executable lines is derived from the
AST (statement start lines plus the span of compound statements). Lines that
can never carry bytecode (`else:`, `except:`, `try:`, `def `/`class `
headers, decorators) are excluded so the denominator reflects real
executable lines only.

Usage:
  measure_python_coverage.py --threshold 50 [--verbose] TARGET [ARG...]

`TARGET` is the script to trace; remaining arguments are forwarded to it.
"""

from __future__ import annotations

import argparse
import ast
import re
import subprocess
import sys
import tempfile
from pathlib import Path

COVER_COUNT_RE = re.compile(r"^ *(\d+):")
COVER_MISSED_RE = re.compile(r"^ *>>>>>>")

# Keywords that have no bytecode on their own line; trace never marks them.
NON_BYTECODE_FIRST = {"def", "class", "async", "elif", "else", "except", "finally", "for", "while", "with", "if", "try", "return", "raise", "yield", "del", "pass", "break", "continue", "global", "nonlocal"}

# Lines that only close a brace from a previous line carry no line event.
BRACE_ONLY = {")", "]", "}", ",", ")", "]", "};", ");"}


def executable_lines(path: Path) -> set[int]:
    try:
        tree = ast.parse(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, SyntaxError):
        return set()
    src = path.read_text(encoding="utf-8").splitlines()

    # Docstring bodies hold no executable code; collect their line spans.
    docstring_lines: set[int] = set()
    for node in ast.walk(tree):
        if isinstance(node, (ast.Module, ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            doc = ast.get_docstring(node, clean=False)
            if doc and node.body and isinstance(node.body[0], ast.Expr):
                first = node.body[0]
                docstring_lines.update(range(first.lineno, (getattr(first, "end_lineno", None) or first.lineno) + 1))

    exe: set[int] = set()
    for node in ast.walk(tree):
        if not isinstance(node, ast.stmt):
            continue
        start = node.lineno
        end = getattr(node, "end_lineno", None) or start
        for lineno in range(start, end + 1):
            if lineno in docstring_lines:
                continue
            stripped = src[lineno - 1].strip()
            if not stripped or stripped.startswith("#"):
                continue
            if stripped.startswith(('"""', "'''")):
                continue
            if stripped in BRACE_ONLY:
                continue
            first = stripped.split()[0].rstrip(":")
            if first in NON_BYTECODE_FIRST:
                continue
            exe.add(lineno)
    return exe


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--threshold", type=float, default=0.0,
                        help="fail if overall coverage is below this percentage")
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument("target", nargs=argparse.REMAINDER,
                        help="script to trace followed by its arguments")
    args = parser.parse_args()

    if not args.target:
        parser.error("a target script is required")
    target = Path(args.target[0]).resolve()
    script_args = args.target[1:]

    cover_dir = Path(tempfile.mkdtemp(prefix="otcov-python-"))
    try:
        cmd = ["python3", "-m", "trace", "--count", "--coverdir", str(cover_dir),
               str(target), *script_args]
        proc = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              text=True, errors="replace")
        cover_file = cover_dir / f"{target.stem}.cover"
        if not cover_file.exists():
            print(f"error: no coverage file produced for {target.name}", file=sys.stderr)
            if proc.returncode:
                print(proc.stdout[:4000], file=sys.stderr)
                print(proc.stderr[:4000], file=sys.stderr)
            return 1
        cover_lines = cover_file.read_text(encoding="utf-8", errors="replace").splitlines()
    finally:
        for entry in cover_dir.iterdir():
            entry.unlink()
        cover_dir.rmdir()

    exercised: set[int] = set()
    for lineno, raw in enumerate(cover_lines, start=1):
        if COVER_COUNT_RE.match(raw):
            exercised.add(lineno)

    exe_lines = executable_lines(target)
    if not exe_lines:
        print(f"error: no executable lines found in {target}", file=sys.stderr)
        return 1

    # Only count lines belonging to the target script itself (the .cover file
    # mirrors the target's own line numbering).
    hit = exercised & exe_lines
    total = len(exe_lines)
    hit_total = len(hit)
    pct = (hit_total / total * 100) if total else 0.0

    if args.verbose:
        print(f"{pct:5.1f}%  {hit_total:4d}/{total:<4d}  {target}")
        for lineno in sorted(exe_lines - hit):
            print(f"  missed {lineno}: {target.read_text().splitlines()[lineno-1].strip()}")

    print(f"overall: {pct:.1f}% ({hit_total}/{total} lines)")
    if args.threshold and pct < args.threshold:
        print(f"error: coverage {pct:.1f}% below threshold {args.threshold:.1f}%",
              file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())