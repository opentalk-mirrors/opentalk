#!/usr/bin/env python3
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

"""Measure line coverage of bash scripts by tracing execution.

Uses `bash -x` with PS4 set to `+@<file>:<line>` so each executed line is
emitted with its source file and line number. The trace output is parsed and
compared against the executable lines of the target scripts.

Usage:
  measure_shell_coverage.py --threshold 50 [--verbose] TARGET [ARG...]

`TARGET` is the script to trace; remaining arguments are forwarded to it.
Scripts sourced by TARGET (e.g. `lib/common.sh`) are included automatically
when they live under the project root and are not vendored.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

TRACE_RE = re.compile(r"^\++@([^:]+):(\d+)[ :]")
HEREDOC_START_RE = re.compile(r"<<[ \t]*-?[ \t]*(?:[\\'\"]([A-Za-z_][A-Za-z0-9_]*)[\\'\"])|<<[ \t]*-?[ \t]*([A-Za-z_][A-Za-z0-9_]*)")
# Pure control-flow lines: bash never emits a trace entry for them because they
# carry no command of their own (closing `if`/`for`/`case`, branches, case
# patterns and separators, and their here-string/file redirections).
CONTROL_WORDS = {"else", "fi", "then", "do", "done", "esac", ";;", "elif"}


def control_only_line(stripped: str) -> bool:
    """True if `stripped` is a pure control-flow line with no own command.

    Covers closing keywords (`fi`, `done`, `esac`, `;;`, `else`, bare `then`/
    `do`), case patterns without a command (`foo)`, `foo|bar)`, `*)`), trailing
    comments, here-strings and file redirections. Lines that spawn a process
    substitution (``done < <(cmd)``) or run a real command stay executable.
    """
    if "(<" in stripped:
        return False
    # Bare case pattern: `  foo)` / `  foo|bar)` / `  *)` (optionally with
    # comment), or an empty branch `  foo) ;;`. Case patterns never contain a
    # left paren, which keeps real array/subshell assignments like
    # `x+=("$y")` executable. A `;;` after a non-empty body (`*) safe=0; break
    # ;;`) stays executable because bash traces its commands.
    if "(" not in stripped and re.fullmatch(r"[ \t]*\S+[ \t]*\)(?:[ \t]*;;)?[ \t]*(?:#.*)?", stripped):
        return True
    first = stripped.split()[0]
    if first not in CONTROL_WORDS:
        return False
    rest = stripped[len(first):].lstrip()
    if not rest or rest.startswith("#"):
        return True
    if re.match(r"^(?:<<<.*|<<-?[ \t]*[A-Za-z_][A-Za-z0-9_]*)$", rest):
        return True
    if re.match(r"^<[ \t]*[^ \t(<]", rest):
        return True
    if re.match(r"^(?:2?>|>|2>&|>&)[ \t]*\S.*$", rest):
        return True
    return False


def heredoc_delimiter(raw: str) -> str | None:
    """Return the here-doc delimiter word of `raw` declared on one line, or None.

    Handles ``<<TAG``, ``<<-TAG`` and the quoted ``<<'TAG'``/``<<\\"TAG"``/
    ``<<\\TAG`` forms. Here-strings (``<<<``) and process substitution
    (``< <(...)``) are not matched. A single heredoc per line is assumed
    (multi-here-doc lines are uncommon); the start line itself remains
    executable.
    """
    match = HEREDOC_START_RE.search(raw)
    if not match:
        return None
    return match.group(1) or match.group(2)


def executable_lines(path: Path) -> set[int]:
    lines: set[int] = set()
    try:
        content = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return lines
    heredoc_delim: str | None = None
    prev_continuation = False
    in_array = False
    for lineno, raw in enumerate(content.splitlines(), start=1):
        if heredoc_delim is not None:
            # Here-doc body and terminator lines are read as data, never traced.
            if raw.strip() == heredoc_delim:
                heredoc_delim = None
            continue
        ends_continuation = raw.rstrip().endswith("\\")
        stripped = raw.strip()
        if not stripped:
            prev_continuation = False
            continue
        if stripped.startswith("#"):
            prev_continuation = False
            continue
        # A multi-line array assignment (`NAME=(` on its own line) is a single
        # statement: bash traces it once, at the line carrying the closing `)`.
        # Element lines never produce an entry of their own, so they stay out
        # of the denominator.
        if in_array:
            if stripped == ")":
                in_array = False
                lines.add(lineno)
            prev_continuation = ends_continuation
            continue
        if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*=\([ \t]*", stripped):
            in_array = True
            prev_continuation = False
            continue
        # Continuation lines of a multi-line command are traced only once, at
        # the command's starting line, so they never produce an entry of their
        # own.
        if prev_continuation:
            prev_continuation = ends_continuation
            continue
        # Brace-only and pure control-flow lines are not independently executable.
        if stripped in {"{", "}", ")"}:
            prev_continuation = False
            continue
        # Function definitions (`name() {`) open a block but never trace by
        # themselves; their bodies are traced via child lines.
        if re.fullmatch(r"[\w @]+\(\)[ \t]*\{[ \t]*(?:#.*)?", stripped):
            prev_continuation = False
            continue
        if control_only_line(stripped):
            prev_continuation = ends_continuation
            continue
        heredoc_delim = heredoc_delimiter(raw)
        lines.add(lineno)
        prev_continuation = ends_continuation
    return lines


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
    project_root = Path(__file__).resolve().parents[3]

    ps4 = r"+@${BASH_SOURCE}:${LINENO} "
    cmd = ["bash", "-x", str(target), *script_args]
    env = os.environ.copy()
    # Enable xtrace in every child (non-interactive) bash via BASH_ENV so that
    # subprocesses like `bash "$OTCTL"` inside the exercised script are traced
    # as well. PS4 propagates through the environment. Xtrace is diverted to a
    # dedicated file via BASH_XTRACEFD so that scripts capturing their own
    # stderr (e.g. command substitutions) are not corrupted by trace output.
    import tempfile
    bash_env_fd, bash_env_path = tempfile.mkstemp(prefix="otcov-env")
    trace_fd, trace_path = tempfile.mkstemp(prefix="otcov-trace", suffix=".xtrace")
    with os.fdopen(bash_env_fd, "w") as fh:
        fh.write(f"exec 3>>{trace_path}\n")
        fh.write("BASH_XTRACEFD=3\n")
        fh.write("set -x\n")
    with os.fdopen(trace_fd, "w"):
        pass
    env["BASH_ENV"] = bash_env_path
    env["PS4"] = ps4
    try:
        proc = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              text=True, errors="replace", env=env)
    finally:
        os.unlink(bash_env_path)
    trace = Path(trace_path).read_text(encoding="utf-8", errors="replace")
    os.unlink(trace_path)

    executed: dict[Path, set[int]] = {}
    for line in trace.splitlines():
        match = TRACE_RE.match(line)
        if not match:
            continue
        filepath = Path(match.group(1))
        if not filepath.is_absolute():
            continue
        try:
            filepath = filepath.resolve()
        except OSError:
            continue
        if project_root not in filepath.parents and filepath != project_root:
            continue
        if ".ansible" in filepath.parts or "/collections/" in filepath.parts:
            continue
        executed.setdefault(filepath, set()).add(int(match.group(2)))

    if not executed:
        print("error: no executable lines traced (is the script executable?)", file=sys.stderr)
        print(proc.stdout[:2000], file=sys.stderr)
        print(trace[:2000], file=sys.stderr)
        return 1

    executable_total = 0
    executed_total = 0
    for path in sorted(executed):
        total = executable_lines(path)
        hit = executed[path]
        exec_total = len(total)
        hit_total = len(hit & total)
        executable_total += exec_total
        executed_total += hit_total
        if args.verbose or exec_total:
            pct = (hit_total / exec_total * 100) if exec_total else 0.0
            print(f"{pct:5.1f}%  {hit_total:4d}/{exec_total:<4d}  {path}")

    overall = (executed_total / executable_total * 100) if executable_total else 0.0
    print(f"overall: {overall:.1f}% ({executed_total}/{executable_total} lines)")
    if args.threshold and overall < args.threshold:
        print(f"error: coverage {overall:.1f}% below threshold {args.threshold:.1f}%",
              file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
