# LinkML Parser V2 Migration - Quick Reference

## At a Glance

| Old (Broken) | New (V2 API) |
|--------------|--------------|
| `SchemaParser::new()` ❌ | `YamlParserV2::new(Arc::new(TokioFileSystemAdapter::new()))` ✅ |
| `Parser::new()` ⚠️ Deprecated | `YamlParserV2` or `JsonParserV2` ✅ |
| `.parse(content)` ❌ | `.parse_str(content)` ✅ |
| `parse_str(content, "yaml")` ⚠️ | `.parse_str(content)` ✅ |

## Quick Migration

### YAML Parsing

```rust
// Add imports
use std::sync::Arc;
use file_system_adapter::TokioFileSystemAdapter;
use linkml_service::parser::YamlParserV2;

// Create parser
let parser = YamlParserV2::new(Arc::new(TokioFileSystemAdapter::new()));

// Parse string
let schema = parser.parse_str(yaml_content)?;

// Parse file (async)
let schema = parser.parse_file(path).await?;
```

### JSON Parsing

```rust
// Add imports
use std::sync::Arc;
use file_system_adapter::TokioFileSystemAdapter;
use linkml_service::parser::JsonParserV2;

// Create parser
let parser = JsonParserV2::new(Arc::new(TokioFileSystemAdapter::new()));

// Parse string
let schema = parser.parse_str(json_content)?;
```

## Cargo.toml Dependencies

```toml
[dependencies]
linkml_service = { workspace = true }
file_system_adapter = { workspace = true }
```

## Common Errors & Solutions

| Error | Solution |
|-------|----------|
| `SchemaParser` is a trait | Use `YamlParserV2` or `JsonParserV2` instead |
| `.parse()` method not found | Use `.parse_str()` instead |
| Format parameter required | Use type-specific parser (YamlParserV2 for YAML, JsonParserV2 for JSON) |
| Missing file system adapter | Add `Arc::new(TokioFileSystemAdapter::new())` to constructor |

## Why Migrate?

1. ✅ **Actually works** - V2 implements trait correctly
2. ✅ **Centralized parsing** - Uses RootReal architecture
3. ✅ **Import resolution** - Supports `txp:` prefixes
4. ✅ **Better errors** - Comprehensive LinkMLError types
5. ✅ **Dependency injection** - Follows RootReal standards

## Full Documentation

📖 See `crates/model/symbolic/linkml/service/PARSER_V2_MIGRATION.md` for:
- Detailed migration patterns
- Integration with RootReal services
- Error handling examples
- Async usage patterns
- Complete migration checklist

---

**Status**: Legacy Parser deprecated (0.2.0), will be removed in 2026-Q1
