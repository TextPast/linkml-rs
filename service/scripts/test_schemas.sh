#!/usr/bin/env bash
# Test script for TextPast/RootReal LinkML schema conventions
# This script validates that all schemas follow the new conventions

set -e

SCHEMA_BASE="crates/model/symbolic/schemata"
PASSED=0
FAILED=0

echo "=== Testing TextPast/RootReal LinkML Schema Conventions ==="
echo ""

# Test 1: Verify all schema files are valid YAML
echo "Test 1: Validating YAML syntax..."
for schema in $(find "$SCHEMA_BASE" -name "schema.yaml" -o -name "*_entity.yaml" -o -name "*.yaml"); do
    if python3 -c "import yaml; yaml.safe_load(open('$schema'))" 2>/dev/null; then
        echo "  ✓ $schema"
        ((PASSED++))
    else
        echo "  ✗ $schema - YAML parsing failed"
        ((FAILED++))
    fi
done

echo ""
echo "Test 2: Checking schema metadata conventions..."

# Test 2: Check schema files have required metadata
for schema in $(find "$SCHEMA_BASE" -name "schema.yaml"); do
    python3 << EOF
import yaml
import sys

schema_file = "$schema"
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
        print(f"  ✗ {schema_file}")
        for err in errors:
            print(f"      - {err}")
        sys.exit(1)
    else:
        txp_note = " (has txp: imports)" if has_txp else ""
        print(f"  ✓ {schema_file}{txp_note}")
        sys.exit(0)
        
except Exception as e:
    print(f"  ✗ {schema_file} - Error: {e}")
    sys.exit(1)
EOF
    if [ $? -eq 0 ]; then
        ((PASSED++))
    else
        ((FAILED++))
    fi
done

echo ""
echo "Test 3: Checking instance file conventions..."

# Test 3: Check instance files have required metadata
for instance in $(find "$SCHEMA_BASE" -name "*_entity.yaml" -o -name "*Entity.yaml"); do
    python3 << EOF
import yaml
import sys

instance_file = "$instance"
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
        print(f"  ✗ {instance_file}")
        for err in errors:
            print(f"      - {err}")
        sys.exit(1)
    else:
        instance_count = len(data.get('instances', []))
        print(f"  ✓ {instance_file} ({instance_count} instances)")
        sys.exit(0)
        
except Exception as e:
    print(f"  ✗ {instance_file} - Error: {e}")
    sys.exit(1)
EOF
    if [ $? -eq 0 ]; then
        ((PASSED++))
    else
        ((FAILED++))
    fi
done

echo ""
echo "=== Test Summary ==="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✓ All tests passed!"
    exit 0
else
    echo "✗ Some tests failed"
    exit 1
fi

