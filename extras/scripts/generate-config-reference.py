#!/usr/bin/env python3
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

"""
Generate the configuration options reference markdown by parsing Ansible role default files
AND scanning Jinja2 templates for additional referenced variables that lack defaults.

Usage:
    python extras/scripts/generate-config-reference.py \
        --roles-root automation/roles \
        --output docs/5-configuration-options-reference.md

Features:
* Scans each role under roles-root for defaults/main.yml.
* Scans Jinja2 templates (*.j2) under each role's templates/ directory to discover referenced variables.
* Merges template-only variables (those used in templates but missing from defaults) into the same role table.
* Type inference: bool/int/float/list/dict/null/string; simple single-line Jinja-only values => derived.
* Secret detection: variable value with Jinja referencing token ending in PASSWORD/SECRET/KEY OR template-only variable name ends with those suffixes.
* Template-only variables are marked with type "template" (or "secret[template]"). Default column uses "—".
* Multiline scalar defaults shown as first line + ellipsis if >1 line.

Idempotent: rewrites the section below the heading '## Overview'.

Limitations:
* Does not parse vars/ or tasks (only defaults + templates).
* Template variable extraction is heuristic (regex-based); loop iterator variable names are excluded when possible.
"""
from __future__ import annotations
import argparse
import datetime
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Set

try:
    import yaml  # PyYAML
except ImportError:
    print("PyYAML not installed. Please install with: pip install PyYAML", file=sys.stderr)
    sys.exit(1)

SECRET_PAT = re.compile(r"{{\s*([A-Za-z0-9_]+)\s*}}")
SECRET_KEY_HINT = re.compile(r"(PASSWORD|SECRET|KEY)$", re.IGNORECASE)

JINJA_EXPR = re.compile(r"{{.*}}")

def infer_type(value: Any) -> str:
    if isinstance(value, bool):
        return "bool"
    if isinstance(value, int):
        return "int"
    if isinstance(value, float):
        return "float"
    if isinstance(value, list):
        # refine list element type for simple homogenous lists
        if value and all(isinstance(v, (str,int,float,bool)) for v in value):
            elem_types = {type(v).__name__ for v in value}
            if len(elem_types) == 1:
                return f"list[{next(iter(elem_types))}]"
        return "list"
    if isinstance(value, dict):
        return "dict"
    if value is None:
        return "null"
    if isinstance(value, str):
        if value.strip() == "":
            return "string"
        if JINJA_EXPR.search(value) and not any(ch in value for ch in ['\n','\r']):
            return "derived"
        return "string"
    return "string"

def is_secret(value: Any) -> bool:
    if not isinstance(value, str):
        return False
    m = SECRET_PAT.search(value)
    if not m:
        return False
    token = m.group(1)
    return bool(SECRET_KEY_HINT.search(token))

def format_default(value: Any) -> str:
    if isinstance(value, str):
        if value == "":
            return "``"
        lines = value.splitlines()
        if len(lines) > 1:
            return f"`{lines[0]} …`"
        return f"`{value}`"
    if isinstance(value, list):
        if not value:
            return "`[]`"
        rendered = "<br>".join(f"{v}" for v in value)
        return rendered
    if isinstance(value, dict):
        if not value:
            return "`{}`"
        # compact representation
        items = ", ".join(f"{k}={repr(v)}" for k,v in list(value.items())[:5])
        if len(value) > 5:
            items += ", …"
        return f"`{{{items}}}`"
    if value is None:
        return "`null`"
    return f"`{value}`"

def collect_role_defaults(role_dir: Path) -> Dict[str, Any]:
    defaults = role_dir / "defaults" / "main.yml"
    if not defaults.exists():
        return {}
    try:
        with defaults.open() as f:
            data = yaml.safe_load(f) or {}
        if not isinstance(data, dict):
            return {}
        return data
    except Exception as e:
        print(f"Warning: failed to parse {defaults}: {e}", file=sys.stderr)
        return {}

def role_display_name(role_dir: Path) -> str:
    # role folder name like opentalk_compose.opentalk_controller
    return role_dir.name

def build_table_for_role(name: str, vars_map: Dict[str, Any], template_only: Set[str]) -> str:
    if not vars_map and not template_only:
        return f"### {name}\n\n(No default or template-only variables detected)\n"
    lines = [f"### {name}", "", "| Variable | Type | Default | Description |", "|---|---|---|---|"]
    # Merge: defaults first, then template-only (ensure deterministic order)
    for var, val in sorted(vars_map.items()):
        vtype = infer_type(val)
        if is_secret(val):
            vtype = f"secret[{vtype}]"
        default_render = format_default(val)
        desc = describe_variable(var)
        lines.append(f"| {var} | {vtype} | {default_render} | {desc}. |")
    for var in sorted(template_only):
        # classify potential secret by name suffix
        vtype = "template"
        if SECRET_KEY_HINT.search(var):
            vtype = f"secret[{vtype}]"
        desc = describe_variable(var) + " (referenced in templates; no default defined)"
        lines.append(f"| {var} | {vtype} | — | {desc}. |")
    lines.append("")
    return "\n".join(lines)

def describe_variable(var: str) -> str:
    desc = var.replace('_', ' ').strip()
    desc = re.sub(r"\b(opentalk|compose)\b", '', desc, flags=re.IGNORECASE).strip()
    prefix_rewrites = {
        'controller_': 'Controller ',
        'keycloak_': 'Keycloak ',
        'livekit_': 'LiveKit ',
        'obelisk_': 'Obelisk ',
        'mailer_': 'Mailer ',
        'recorder_': 'Recorder ',
        'rabbitmq_': 'RabbitMQ ',
        'traefik_': 'Traefik ',
        'etherpad_': 'Etherpad ',
        'spacedeck_': 'Spacedeck ',
    }
    for pre, title in prefix_rewrites.items():
        if var.startswith(pre):
            desc = title + desc[len(pre):].strip()
            break
    if desc:
        desc = desc[:1].upper() + desc[1:]
    return desc or var

def extract_template_vars(role_dir: Path) -> Set[str]:
    """Heuristically extract variable names referenced in Jinja2 templates (*.j2)."""
    templates_root = role_dir / "templates"
    if not templates_root.exists():
        return set()
    vars_found: Set[str] = set()
    loop_iterators: Set[str] = set()
    jinja_keywords = {"for","in","if","else","endif","endfor","True","False","true","false","loop","and","or","not"}
    tag_for_pat = re.compile(r"{%\s*for\s+([A-Za-z_][A-Za-z0-9_]*)\s+in\s+([A-Za-z_][A-Za-z0-9_]*)")
    tag_block_pat = re.compile(r"{%[^%]*%}")
    expr_pat = re.compile(r"{{[^}]*}}")
    token_pat = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")

    for tpl in templates_root.rglob("*.j2"):
        try:
            content = tpl.read_text(encoding="utf-8")
        except Exception:
            continue
        # For loops
        for m in tag_for_pat.finditer(content):
            loop_iterators.add(m.group(1))
            vars_found.add(m.group(2))
        # Other tag blocks (conditions etc.)
        for blk in tag_block_pat.findall(content):
            for tok in token_pat.findall(blk):
                if tok in jinja_keywords:
                    continue
                # Skip loop iterator variable names
                if tok in loop_iterators:
                    continue
                vars_found.add(tok)
        # Expression blocks
        for expr in expr_pat.findall(content):
            for tok in token_pat.findall(expr):
                if tok in jinja_keywords:
                    continue
                if tok in loop_iterators:
                    continue
                vars_found.add(tok)
    return vars_found

def generate(roles_root: Path) -> str:
    sections: List[str] = []
    for role in sorted(roles_root.iterdir()):
        if not role.is_dir():
            continue
        defaults = collect_role_defaults(role)
        template_vars = extract_template_vars(role)
        template_only = {v for v in template_vars if v not in defaults}
        sections.append(build_table_for_role(role_display_name(role), defaults, template_only))
    timestamp = datetime.date.today().isoformat()
    header = [
        "## Overview (Auto-generated)",
        "",
        "Run extras/scripts/generate-config-reference.py to refresh.",
        "",
        "Template-only variables: referenced in Jinja2 templates but lacking a default; these may need explicit definition in settings.yml.",
        ""
    ]
    footer = ["---", f"Generated on: {timestamp}"]
    return "\n".join(header + sections + footer) + "\n"

def replace_overview(output_file: Path, new_content: str) -> None:
    if not output_file.exists():
        print(f"Error: output file {output_file} does not exist", file=sys.stderr)
        sys.exit(2)
    text = output_file.read_text()
    # Replace starting at first '## Overview' block
    pattern = re.compile(r"## Overview[\s\S]*", re.MULTILINE)
    result = pattern.sub(new_content, text)
    output_file.write_text(result)
    print(f"Updated {output_file}")

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--roles-root', default='automation/roles', help='Root directory containing Ansible roles')
    p.add_argument('--output', default='docs/5-configuration-options-reference.md', help='Markdown file to update')
    p.add_argument('--list', action='store_true', help='List roles and exit')
    args = p.parse_args()
    roles_root = Path(args.roles_root)
    if not roles_root.exists():
        print(f"Roles root not found: {roles_root}", file=sys.stderr)
        sys.exit(2)
    roles = [r.name for r in roles_root.iterdir() if r.is_dir()]
    if args.list:
        for r in sorted(roles):
            print(r)
        return
    generated = generate(roles_root)
    output_file = Path(args.output)
    replace_overview(output_file, generated)

if __name__ == '__main__':
    main()
