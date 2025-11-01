#!/usr/bin/env python3
"""
Test script for TextPast/RootReal LinkML schema conventions

This script validates that all schemas follow the new conventions:
1. Schema files have proper metadata and txp: imports
2. Instance files have explicit schema references and 'instances' key
3. All files are valid YAML
4. ISO3166Entity IDs conform to CountryCodeAlpha2Identifier pattern
"""

import yaml
import re
from pathlib import Path
from typing import List, Tuple

SCHEMA_BASE = Path("crates/model/symbolic/schemata")

def test_yaml_syntax() -> Tuple[int, int]:
    """Test that all YAML files parse correctly."""
    print("Test 1: Validating YAML syntax...")
    passed = 0
    failed = 0
    
    for yaml_file in SCHEMA_BASE.rglob("*.yaml"):
        try:
            with open(yaml_file) as f:
                yaml.safe_load(f)
            print(f"  ✓ {yaml_file.relative_to(SCHEMA_BASE.parent)}")
            passed += 1
        except Exception as e:
            print(f"  ✗ {yaml_file.relative_to(SCHEMA_BASE.parent)} - {e}")
            failed += 1
    
    return passed, failed

def test_schema_metadata() -> Tuple[int, int]:
    """Test that schema files have required metadata."""
    print("\nTest 2: Checking schema metadata conventions...")
    passed = 0
    failed = 0
    
    for schema_file in SCHEMA_BASE.rglob("schema.yaml"):
        try:
            with open(schema_file) as f:
                schema = yaml.safe_load(f)
            
            errors = []
            
            # Check required fields
            if not schema.get('id', '').startswith('https://textpast.org/schema/'):
                errors.append("ID should start with https://textpast.org/schema/")
            
            if not schema.get('name'):
                errors.append("Missing 'name' field")
            
            if not schema.get('version'):
                errors.append("Missing 'version' field")
            
            if not schema.get('created_on'):
                errors.append("Missing 'created_on' field")
            
            # Check for txp: imports
            imports = schema.get('imports', [])
            has_txp = any(imp.startswith('txp:') for imp in imports)
            
            if errors:
                print(f"  ✗ {schema_file.relative_to(SCHEMA_BASE.parent)}")
                for err in errors:
                    print(f"      - {err}")
                failed += 1
            else:
                txp_note = " (has txp: imports)" if has_txp else ""
                print(f"  ✓ {schema_file.relative_to(SCHEMA_BASE.parent)}{txp_note}")
                passed += 1
                
        except Exception as e:
            print(f"  ✗ {schema_file.relative_to(SCHEMA_BASE.parent)} - Error: {e}")
            failed += 1
    
    return passed, failed

def test_instance_metadata() -> Tuple[int, int]:
    """Test that instance files have required metadata."""
    print("\nTest 3: Checking instance file conventions...")
    passed = 0
    failed = 0
    
    # Find instance files (ending with _entity.yaml or Entity.yaml, but not schema.yaml)
    for instance_file in SCHEMA_BASE.rglob("*.yaml"):
        if instance_file.name == "schema.yaml":
            continue
        if not ("entity" in instance_file.name.lower() or "Entity" in instance_file.name):
            continue
            
        try:
            with open(instance_file) as f:
                data = yaml.safe_load(f)
            
            errors = []
            
            # Check required fields
            if not data.get('id', '').startswith('https://textpast.org/instance/'):
                errors.append("ID should start with https://textpast.org/instance/")
            
            if not data.get('schema', '').startswith('https://textpast.org/schema/'):
                errors.append("schema field should start with https://textpast.org/schema/")
            
            if not data.get('version'):
                errors.append("Missing 'version' field")
            
            if not data.get('created_on'):
                errors.append("Missing 'created_on' field")
            
            if 'instances' not in data:
                errors.append("Missing 'instances' key")
            
            if errors:
                print(f"  ✗ {instance_file.relative_to(SCHEMA_BASE.parent)}")
                for err in errors:
                    print(f"      - {err}")
                failed += 1
            else:
                instance_count = len(data.get('instances', []))
                print(f"  ✓ {instance_file.relative_to(SCHEMA_BASE.parent)} ({instance_count} instances)")
                passed += 1
                
        except Exception as e:
            print(f"  ✗ {instance_file.relative_to(SCHEMA_BASE.parent)} - Error: {e}")
            failed += 1
    
    return passed, failed

def test_iso3166_identifiers() -> Tuple[int, int]:
    """Test that ISO3166Entity IDs conform to CountryCodeAlpha2Identifier pattern."""
    print("\nTest 4: Validating ISO3166Entity identifiers...")
    
    iso_file = SCHEMA_BASE / "place/polity/country/iso_3166_entity.yaml"
    if not iso_file.exists():
        print(f"  ⚠ {iso_file.relative_to(SCHEMA_BASE.parent)} not found")
        return 0, 0
    
    try:
        with open(iso_file) as f:
            data = yaml.safe_load(f)
        
        instances = data.get('instances', [])
        pattern = re.compile(r'^[A-Z]{2}$')
        
        valid_count = 0
        invalid_ids = []
        
        for instance in instances:
            id_val = instance.get('id', '')
            if pattern.match(id_val):
                valid_count += 1
            else:
                invalid_ids.append(id_val)
        
        if invalid_ids:
            print(f"  ✗ Found {len(invalid_ids)} invalid IDs: {invalid_ids[:10]}")
            return 0, 1
        else:
            print(f"  ✓ All {valid_count} IDs conform to CountryCodeAlpha2Identifier pattern")
            return 1, 0
            
    except Exception as e:
        print(f"  ✗ Error: {e}")
        return 0, 1

def main():
    print("=== Testing TextPast/RootReal LinkML Schema Conventions ===\n")
    
    total_passed = 0
    total_failed = 0
    
    # Run all tests
    p, f = test_yaml_syntax()
    total_passed += p
    total_failed += f
    
    p, f = test_schema_metadata()
    total_passed += p
    total_failed += f
    
    p, f = test_instance_metadata()
    total_passed += p
    total_failed += f
    
    p, f = test_iso3166_identifiers()
    total_passed += p
    total_failed += f
    
    # Summary
    print("\n=== Test Summary ===")
    print(f"Passed: {total_passed}")
    print(f"Failed: {total_failed}")
    print()
    
    if total_failed == 0:
        print("✓ All tests passed!")
        return 0
    else:
        print("✗ Some tests failed")
        return 1

if __name__ == "__main__":
    exit(main())

