#!/usr/bin/env python3
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

"""
Generates a markdown reference for Ansible role variables.

This script scans Ansible roles for variables defined in `defaults/*.yml` and `vars/*.yml`
and creates a markdown document summarizing them.
"""

import argparse
import datetime
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Set

SETTINGS_SAMPLE_REL_PATH = "extras/packaging/config-dist/settings.yml-sample"

try:
    import yaml
except ImportError:
    print("PyYAML not installed. Please install with: pip install PyYAML", file=sys.stderr)
    sys.exit(1)

def _parse_commented_variables(text: str, variables: Dict[str, Any]) -> None:
    """Extracts commented-out variable definitions from YAML text."""
    commented_var_re = re.compile(r"^#\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*:(.*)$")
    for line in text.splitlines():
        m = commented_var_re.match(line)
        if m:
            var_name = m.group(1)
            if var_name not in variables:
                raw_value = m.group(2).strip()
                raw_value = re.sub(r"\s+#.*$", "", raw_value)
                if len(raw_value) >= 2 and raw_value[0] == raw_value[-1] and raw_value[0] in ('"', "'"):
                    raw_value = raw_value[1:-1]
                variables[var_name] = raw_value if raw_value else None


def get_role_variables(role_path: Path) -> Dict[str, Any]:
    """
    Collects variables from a role's defaults and vars directories.
    Variables in 'vars' will override those in 'defaults'.
    Also includes commented-out variable definitions.
    """
    variables: Dict[str, Any] = {}
    
    # Load from defaults
    defaults_dir = role_path / "defaults"
    if defaults_dir.is_dir():
        for yml_file in sorted(defaults_dir.glob("*.yml")):
            try:
                with yml_file.open(encoding="utf-8") as f:
                    text = f.read()
                data = yaml.safe_load(text) or {}
                if isinstance(data, dict):
                    variables.update(data)
                _parse_commented_variables(text, variables)
            except Exception as e:
                print(f"Warning: Could not parse {yml_file}: {e}", file=sys.stderr)

    # Load from vars, overriding defaults
    vars_dir = role_path / "vars"
    if vars_dir.is_dir():
        for yml_file in sorted(vars_dir.glob("*.yml")):
            try:
                with yml_file.open(encoding="utf-8") as f:
                    text = f.read()
                data = yaml.safe_load(text) or {}
                if isinstance(data, dict):
                    variables.update(data)
                _parse_commented_variables(text, variables)
            except Exception as e:
                print(f"Warning: Could not parse {yml_file}: {e}", file=sys.stderr)
    
    return variables

def get_settings_sample_variables(sample_path: Path) -> Dict[str, Any]:
    """
    Collects variable names from a settings.yml-sample file,
    including commented-out variable definitions.
    """
    variables: Dict[str, Any] = {}

    try:
        text = sample_path.read_text(encoding="utf-8")
    except Exception as e:
        print(f"Warning: Could not read {sample_path}: {e}", file=sys.stderr)
        return variables

    # Parse uncommented variables via YAML
    try:
        data = yaml.safe_load(text) or {}
        if isinstance(data, dict):
            variables.update(data)
    except Exception as e:
        print(f"Warning: Could not parse {sample_path}: {e}", file=sys.stderr)

    _parse_commented_variables(text, variables)

    return variables


def infer_type(value: Any) -> str:
    """Infers the type of a value and returns a string representation."""
    if isinstance(value, bool):
        return "boolean"
    if isinstance(value, int):
        return "integer"
    if isinstance(value, float):
        return "float"
    if isinstance(value, list):
        return "list"
    if isinstance(value, dict):
        return "dictionary"
    if value is None:
        return "null"
    return "string"

def format_value(value: Any) -> str:
    """Formats a Python value for display in a Markdown table."""
    if isinstance(value, bool):
        return f"`{str(value).lower()}`"
    if isinstance(value, str):
        if not value:
            return "``"
        # Escape pipe characters and show newlines as <br>
        escaped_value = value.replace("|", "\\|")
        lines = escaped_value.splitlines()
        if len(lines) > 1:
            return "<br>".join(f"`{line}`" for line in lines)
        return f"`{escaped_value}`"
    if value is None:
        return "`null`"
    return f"`{value}`"

# REUSE-IgnoreStart
def generate_markdown(roles_root: Path) -> str:
    """Generates the full markdown content for all roles."""
    content: List[str] = [
        "<!--",
        "SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>",
        "",
        "SPDX-License-Identifier: EUPL-1.2",
        "-->",
        "",
        "# Configuration Reference (Auto-generated)",
        "",
        f"Last updated: {datetime.date.today().isoformat()}",
        "",
        "This document lists the configuration variables for each Ansible role.",
        "Variables are collected from `defaults/*.yml` and `vars/*.yml`.",
        "Default values should be overridden in `/etc/opentalk-compose/settings.yml`.",
        ""
    ]

    roles = sorted([d for d in roles_root.iterdir() if d.is_dir()])
    all_role_vars: Set[str] = set()
    role_sections: List[str] = []

    for role_path in roles:
        role_sections.append(f"## {role_path.name}")
        role_sections.append("")
        
        variables = get_role_variables(role_path)
        all_role_vars.update(variables.keys())
        if not variables:
            role_sections.append("(No variables found)")
            role_sections.append("")
            continue

        role_sections.append("| Variable | Type | Default Value |")
        role_sections.append("|----------|------|---------------|")
        for name, value in sorted(variables.items()):
            var_type = infer_type(value)
            role_sections.append(f"| `{name}` | {var_type} | {format_value(value)} |")
        role_sections.append("")

    # Check settings.yml-sample for variables not defined in any role
    project_root = roles_root.parent.parent
    settings_sample = project_root / SETTINGS_SAMPLE_REL_PATH
    if settings_sample.is_file():
        sample_vars = get_settings_sample_variables(settings_sample)
        extra_vars = {k: v for k, v in sample_vars.items() if k not in all_role_vars}
        content.append("## Global Variables in settings.yml")
        content.append("")
        if extra_vars:
            content.append("| Variable | Type | Default Value |")
            content.append("|----------|------|---------------|")
            for name, value in sorted(extra_vars.items()):
                var_type = infer_type(value)
                content.append(f"| `{name}` | {var_type} | {format_value(value)} |")
            content.append("")
        else:
            content.append("(All variables are defined in roles)")
            content.append("")
    else:
        print(f"Warning: settings.yml-sample not found at '{settings_sample}'", file=sys.stderr)

    content.extend(role_sections)

    return "\n".join(content)
# REUSE-IgnoreEnd

def main():
    """Main function to parse arguments and run the generator."""
    parser = argparse.ArgumentParser(
        description="Generate a markdown reference for Ansible role variables."
    )
    parser.add_argument(
        "--roles-root",
        default="automation/roles",
        help="Path to the directory containing Ansible roles."
    )
    parser.add_argument(
        "--output",
        default="docs/5-configuration-options-reference.md",
        help="Path to the output markdown file."
    )
    args = parser.parse_args()

    roles_root = Path(args.roles_root)
    output_file = Path(args.output)

    if not roles_root.is_dir():
        print(f"Error: Roles directory not found at '{roles_root}'", file=sys.stderr)
        sys.exit(1)

    print("Generating configuration reference...")
    markdown_content = generate_markdown(roles_root)
    
    try:
        output_file.write_text(markdown_content, encoding="utf-8")
        print(f"Successfully wrote reference to {output_file}")
    except Exception as e:
        print(f"Error: Could not write to {output_file}: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
