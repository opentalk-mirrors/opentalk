#!/usr/bin/env python3
import sys
import yaml
import json

def find_new_keys(defaults, user, prefix=''):
    """Recursively find new keys in defaults that are missing in user"""
    new_keys = []
    
    if not isinstance(defaults, dict):
        return new_keys
        
    for key, value in defaults.items():
        current_path = f"{prefix}.{key}" if prefix else key
        
        if key not in user:
            # Completely new key
            new_keys.append(current_path)
        elif isinstance(value, dict) and isinstance(user.get(key), dict):
            # Both are dicts, check recursively
            nested_keys = find_new_keys(value, user[key], current_path)
            new_keys.extend(nested_keys)
    
    return new_keys

def main():
    if len(sys.argv) != 3:
        print("Usage: find_new_yaml_keys.py <defaults.yml> <user.yml>")
        sys.exit(1)
    
    defaults_file = sys.argv[1]
    user_file = sys.argv[2]
    
    try:
        with open(defaults_file, 'r') as f:
            defaults = yaml.safe_load(f) or {}
        
        with open(user_file, 'r') as f:
            user = yaml.safe_load(f) or {}
        
        new_keys = find_new_keys(defaults, user)
        
        # Output as JSON for Ansible
        result = {
            "new_keys": new_keys,
            "has_new_keys": len(new_keys) > 0
        }
        print(json.dumps(result))
        
    except Exception as e:
        print(json.dumps({"error": str(e), "new_keys": [], "has_new_keys": False}))
        sys.exit(1)

if __name__ == "__main__":
    main()