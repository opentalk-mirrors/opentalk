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
import sys
from pathlib import Path
from typing import Any, Dict, List

try:
    import yaml
except ImportError:
    print("PyYAML not installed. Please install with: pip install PyYAML", file=sys.stderr)
    sys.exit(1)

def get_role_variables(role_path: Path) -> Dict[str, Any]:
    """
    Collects variables from a role's defaults and vars directories.
    Variables in 'vars' will override those in 'defaults'.
    """
    variables: Dict[str, Any] = {}
    
    # Load from defaults
    defaults_dir = role_path / "defaults"
    if defaults_dir.is_dir():
        for yml_file in sorted(defaults_dir.glob("*.yml")):
            try:
                with yml_file.open(encoding="utf-8") as f:
                    data = yaml.safe_load(f) or {}
                    if isinstance(data, dict):
                        variables.update(data)
            except Exception as e:
                print(f"Warning: Could not parse {yml_file}: {e}", file=sys.stderr)

    # Load from vars, overriding defaults
    vars_dir = role_path / "vars"
    if vars_dir.is_dir():
        for yml_file in sorted(vars_dir.glob("*.yml")):
            try:
                with yml_file.open(encoding="utf-8") as f:
                    data = yaml.safe_load(f) or {}
                    if isinstance(data, dict):
                        variables.update(data)
            except Exception as e:
                print(f"Warning: Could not parse {yml_file}: {e}", file=sys.stderr)
    
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

    for role_path in roles:
        content.append(f"## {role_path.name}")
        content.append("")
        
        variables = get_role_variables(role_path)
        if not variables:
            content.append("(No variables found)")
            content.append("")
            continue

        content.append("| Variable | Type | Default Value |")
        content.append("|----------|------|---------------|")
        for name, value in sorted(variables.items()):
            var_type = infer_type(value)
            content.append(f"| `{name}` | {var_type} | {format_value(value)} |")
        content.append("")

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
