# LinkML Parser V2 Migration Guide

## Overview

The legacy `Parser` struct in `linkml-service` has been deprecated in favor of the new V2 parser API (`YamlParserV2` and `JsonParserV2`). This guide explains why the migration is necessary and how to update your code.

## Why the Migration?

### The Core Problem

The original code pattern attempted to use `SchemaParser` as if it were a concrete struct, when it's actually a **trait**:

```rust
// ❌ BROKEN: This never worked!
use linkml_service::parser::SchemaParser;
let mut parser = SchemaParser::new();  // ERROR: SchemaParser is a trait, not a struct!
let schema = parser.parse(schema_yaml)?;  // ERROR: Method doesn't exist!
```

### Why It Was Broken

1. **`SchemaParser` is a trait** (defined at lines 58-75 in `mod.rs`), not an instantiable struct
2. **The legacy `Parser` struct exists** but has a different API:
   - Requires `parse_str(&self, content: &str, format: &str)` - needs format parameter
   - No simple `.parse()` method like the examples expected
3. **Architectural violation**: The legacy `Parser` uses direct `serde_yaml`/`serde_json`, violating RootReal's mandatory centralized parsing architecture

### The V2 Solution

The V2 parsers (`YamlParserV2`, `JsonParserV2`) provide:
- **Proper trait implementation** with correct API signature
- **Centralized parsing** using RootReal's `parse-linkml` specialized PEG parser
- **Better architecture** aligning with RootReal standards
- **The API that examples expected** - `parse_str(&self, content: &str)` without format parameter

## Migration Instructions

### Basic Migration Pattern

**Before (Broken):**
```rust
use linkml_service::parser::SchemaParser;

let mut parser = SchemaParser::new();  // ERROR!
let schema = parser.parse(schema_yaml)?;  // ERROR!
```

**After (V2 API):**
```rust
use std::sync::Arc;
use file_system_adapter::TokioFileSystemAdapter;
use linkml_service::parser::YamlParserV2;

let parser = YamlParserV2::new(Arc::new(TokioFileSystemAdapter::new()));
let schema = parser.parse_str(schema_yaml)?;
```

### For JSON Schemas

**Before (Broken):**
```rust
use linkml_service::parser::SchemaParser;

let parser = SchemaParser::new();
let schema = parser.parse(json_content)?;
```

**After (V2 API):**
```rust
use std::sync::Arc;
use file_system_adapter::TokioFileSystemAdapter;
use linkml_service::parser::JsonParserV2;

let parser = JsonParserV2::new(Arc::new(TokioFileSystemAdapter::new()));
let schema = parser.parse_str(json_content)?;
```

### If You Were Using the Legacy `Parser` Struct

**Before:**
```rust
use linkml_service::parser::Parser;

let parser = Parser::new();
let schema = parser.parse_str(content, "yaml")?;
```

**After:**
```rust
use std::sync::Arc;
use file_system_adapter::TokioFileSystemAdapter;
use linkml_service::parser::YamlParserV2;

let parser = YamlParserV2::new(Arc::new(TokioFileSystemAdapter::new()));
let schema = parser.parse_str(content)?;  // No format parameter needed!
```

## Migrated Examples (Reference)

The following examples have been successfully migrated:

1. **`crates/model/symbolic/linkml/service/examples/04_code_generation/database/sqlalchemy_generation.rs`**
   - Replaced `SchemaParser::new()` with `YamlParserV2::new()`
   - Updated to use `parse_str()` without format parameter

2. **`crates/model/symbolic/linkml/service/examples/05_visualization/schemas/project_generation.rs`**
   - Same migration pattern applied
   - Now uses V2 parser API throughout

3. **`crates/model/symbolic/linkml/service/examples/08_advanced/extensibility/plugin_system_demo.rs`**
   - Migrated to V2 parser
   - Fixed missing `logger` field in `PluginContext` initialization

## Benefits of V2 API

1. **Actually Works**: V2 parsers implement the `SchemaParser` trait correctly
2. **Centralized Architecture**: Uses RootReal's mandatory centralized parsing infrastructure
3. **Better Error Handling**: Leverages specialized PEG parser for LinkML
4. **Cleaner API**: No need to specify format for YAML/JSON parsers
5. **Type Safety**: Proper trait implementations with compile-time guarantees
6. **Future-Proof**: Aligns with RootReal's architectural evolution
7. **Import Resolution**: Built-in support for `txp:` prefix resolution via Cache Service
8. **Dependency Injection**: Follows RootReal's wiring-based DI pattern

## Deprecation Timeline

- **Version 0.2.0**: Legacy `Parser` struct marked as deprecated
- **Future versions**: Legacy `Parser` may be removed entirely
- **Recommendation**: Migrate all code to V2 API as soon as possible

## Integration with RootReal Services

### Using with Wiring Functions

For production code, use wiring functions to get properly configured parsers:

```rust
use linkml_service::wiring::wire_yaml_parser_v2;

// Get parser with all dependencies wired
let parser = wire_yaml_parser_v2();
let schema = parser.parse_str(schema_yaml)?;
```

### Custom File System Adapters

For testing or specialized use cases:

```rust
use std::sync::Arc;
use file_system_adapter::{TokioFileSystemAdapter, InMemoryFileSystemAdapter};
use linkml_service::parser::YamlParserV2;

// Production: Real file system
let parser = YamlParserV2::new(Arc::new(TokioFileSystemAdapter::new()));

// Testing: In-memory file system
let fs = Arc::new(InMemoryFileSystemAdapter::new());
fs.write("/test/schema.yaml", schema_content).await?;
let parser = YamlParserV2::new(fs);
let schema = parser.parse_file(Path::new("/test/schema.yaml")).await?;
```

## Error Handling

V2 parsers return `Result<LinkMLSchema, LinkMLError>`:

```rust
match parser.parse_str(schema_yaml) {
    Ok(schema) => {
        println!("Schema ID: {}", schema.id);
        println!("Schema name: {}", schema.name);
    }
    Err(LinkMLError::ParseError { message, context }) => {
        eprintln!("Parse error: {}", message);
        if let Some(ctx) = context {
            eprintln!("Context: {:?}", ctx);
        }
    }
    Err(LinkMLError::ValidationError { message, context }) => {
        eprintln!("Validation error: {}", message);
    }
    Err(e) => {
        eprintln!("Other error: {:?}", e);
    }
}
```

## Async Considerations

V2 parsers use async APIs for file operations:

```rust
// ✅ Async function with .await
async fn load_schema(path: &Path) -> Result<LinkMLSchema, LinkMLError> {
    let parser = YamlParserV2::new(Arc::new(TokioFileSystemAdapter::new()));
    parser.parse_file(path).await  // Note the .await
}

// ✅ String parsing is still sync-compatible
fn parse_schema_str(content: &str) -> Result<LinkMLSchema, LinkMLError> {
    let parser = YamlParserV2::new(Arc::new(TokioFileSystemAdapter::new()));
    parser.parse_str(content)  // No .await needed for string parsing
}
```

## Need Help?

If you encounter issues during migration:

1. Check if you're trying to instantiate `SchemaParser` (it's a trait!)
2. Verify you're using `YamlParserV2` or `JsonParserV2` (not `Parser`)
3. Ensure you're providing `Arc<TokioFileSystemAdapter>` to constructor
4. Use `parse_str()` method without format parameter
5. Add `.await` for `parse_file()` operations
6. Check that imports with `txp:` prefixes are resolving correctly

## Related Documentation

- **V2 Parser Implementation**: `crates/model/symbolic/linkml/service/src/parser/yaml_parser_v2.rs`
- **Centralized Parsing Architecture**: `crates/data/parsing/parse/linkml-parser/`
- **RootReal Parsing Standards**: See architecture docs in `docs/architecture/`

## Common Migration Patterns

### Pattern 1: File Parsing

**Before:**
```rust
let parser = Parser::new();
let content = std::fs::read_to_string(path)?;
let schema = parser.parse_str(&content, "yaml")?;
```

**After:**
```rust
use std::sync::Arc;
use file_system_adapter::TokioFileSystemAdapter;
use linkml_service::parser::YamlParserV2;

let parser = YamlParserV2::new(Arc::new(TokioFileSystemAdapter::new()));
let schema = parser.parse_file(&path).await?;  // Direct file parsing
```

### Pattern 2: With Import Resolution

**Before:**
```rust
// ❌ Old Parser doesn't support import resolution properly
let parser = Parser::new();
let schema = parser.parse_str(schema_yaml, "yaml")?;
// Imports with txp: prefix would fail
```

**After:**
```rust
use std::sync::Arc;
use file_system_adapter::TokioFileSystemAdapter;
use linkml_service::parser::YamlParserV2;

// ✅ V2 parsers automatically resolve txp: imports via Cache Service
let parser = YamlParserV2::new(Arc::new(TokioFileSystemAdapter::new()));
let schema = parser.parse_str(schema_yaml)?;
// txp:meta/identifier/identifier/schema imports are automatically resolved
```

## Migration Checklist

- [ ] Replace `SchemaParser::new()` with `YamlParserV2::new()` or `JsonParserV2::new()`
- [ ] Add `Arc<TokioFileSystemAdapter>` dependency injection
- [ ] Update `parse()` calls to `parse_str()`
- [ ] Remove format parameter if using legacy `Parser`
- [ ] Add `use std::sync::Arc;` and `use file_system_adapter::TokioFileSystemAdapter;`
- [ ] Add `.await` for `parse_file()` operations
- [ ] Update error handling for new `LinkMLError` types
- [ ] Test import resolution with `txp:` prefixes
- [ ] Verify all parsers use wiring functions in production code
- [ ] Verify compilation with `cargo check`
- [ ] Test schema parsing with your schemas

---

**Last Updated**: 2025-11-10 (Session: LinkML Parser V2 Migration)
