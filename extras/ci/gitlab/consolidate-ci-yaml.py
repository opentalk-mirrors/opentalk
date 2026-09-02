#!/usr/bin/env python3
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

"""
Transform GitLab CI YAML by extracting and consolidating OT_SETTINGS blocks.
This post-processor reduces YAML duplication by using a shared base configuration.

Usage: python3 consolidate-ci-yaml.py <input_file> <output_file>
"""

import sys
import re
from typing import List, Tuple


def extract_ot_settings_blocks(content: str) -> List[Tuple[int, int, str]]:
    """Extract OT_SETTINGS blocks with their line ranges and content."""
    blocks = []
    lines = content.split('\n')
    
    i = 0
    while i < len(lines):
        if 'OT_SETTINGS: |' in lines[i]:
            start_line = i
            base_indent = len(lines[i]) - len(lines[i].lstrip())
            block_lines = [lines[i]]
            i += 1
            
            # Collect all lines of the block (more indented than the declaration)
            while i < len(lines):
                line = lines[i]
                if line.strip() == '':
                    block_lines.append(line)
                    i += 1
                else:
                    line_indent = len(line) - len(line.lstrip())
                    if line_indent > base_indent:
                        block_lines.append(line)
                        i += 1
                    else:
                        break
            
            block_content = '\n'.join(block_lines)
            blocks.append((start_line, i, block_content))
        else:
            i += 1
    
    return blocks


def extract_ot_settings_yaml(block_content: str) -> dict:
    """Parse OT_SETTINGS YAML content into dictionary."""
    lines = block_content.split('\n')
    settings = {}
    
    # Skip the "OT_SETTINGS: |" line
    yaml_lines = [l for l in lines[1:] if l.strip()]
    
    for line in yaml_lines:
        if ': ' in line and not line.lstrip().startswith('-'):
            key, value = line.split(': ', 1)
            key = key.strip()
            value = value.strip()
            # Remove quotes if present
            if value.startswith("'") and value.endswith("'"):
                value = value[1:-1]
            elif value.startswith('"') and value.endswith('"'):
                value = value[1:-1]
            settings[key] = value
    
    return settings


def find_ot_settings_differences(blocks: List[Tuple[int, int, str]]) -> None:
    """Identify and print differences between OT_SETTINGS blocks."""
    all_settings = []
    for start, end, content in blocks:
        settings = extract_ot_settings_yaml(content)
        all_settings.append(settings)
    
    if not all_settings:
        return
    
    # Find common and differing keys
    base = all_settings[0]
    all_keys = set(base.keys())
    for settings in all_settings[1:]:
        all_keys.update(settings.keys())
    
    differences = {}
    for key in all_keys:
        values = [s.get(key) for s in all_settings]
        if len(set(values)) > 1:
            differences[key] = values
    
    if differences:
        print(f"\n📊 OT_SETTINGS Differences across {len(all_settings)} jobs:")
        print("=" * 70)
        for key, values in sorted(differences.items()):
            print(f"\n  {key}:")
            for i, val in enumerate(values, 1):
                print(f"    Block {i}: {val}")


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <input_yaml>")
        sys.exit(1)
    
    input_file = sys.argv[1]
    
    with open(input_file, 'r') as f:
        content = f.read()
    
    blocks = extract_ot_settings_blocks(content)
    
    if blocks:
        print(f"📋 Found {len(blocks)} OT_SETTINGS blocks in {input_file}")
        find_ot_settings_differences(blocks)
    else:
        print(f"No OT_SETTINGS blocks found in {input_file}")
