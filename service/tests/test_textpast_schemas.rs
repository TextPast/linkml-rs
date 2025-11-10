//! Integration tests for TextPast/RootReal LinkML schema conventions
//!
//! Tests:
//! 1. Schema files parse correctly with txp: imports
//! 2. Instance files have proper metadata
//! 3. txp: imports resolve local-first
//! 4. Slot usage with scoped imports works correctly
//! 5. ISO3166Entity ID validation against CountryCodeAlpha2Identifier

#![allow(missing_docs)]

use linkml_service::parser::{YamlParserSimple, SchemaLoader, SchemaParser};
use std::path::PathBuf;

/// Helper function to get the repository root path
fn get_repo_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Navigate from crates/model/symbolic/linkml/service to repository root
    path.pop(); // service
    path.pop(); // linkml
    path.pop(); // symbolic
    path.pop(); // model
    path.pop(); // crates
    path
}

/// Test that the hyperentity schema parses correctly using the LinkML service
#[tokio::test]
async fn test_parse_hyperentity_schema_with_service() {
    let loader = SchemaLoader::new();
    let schema_path = get_repo_root().join("crates/model/symbolic/schemata/meta/entity/hyperentity/schema.yaml");

    let schema = loader.load_file(&schema_path)
        .await
        .expect("Failed to load hyperentity schema");

    // Verify schema metadata
    assert_eq!(schema.id, "https://textpast.org/schema/meta/entity/hyperentity");
    assert_eq!(schema.name, "hyperentity");
    assert!(schema.version.is_some(), "Schema should have version");

    // Verify classes (after imports are merged, there will be more than just the 3 defined in this schema)
    assert!(schema.classes.len() >= 3, "Should have at least 3 classes (plus imported classes)");
    assert!(schema.classes.contains_key("Entity"), "Should have Entity class");
    assert!(schema.classes.contains_key("COT"), "Should have COT class");
    assert!(schema.classes.contains_key("Group"), "Should have Group class");

    println!("✓ Hyperentity schema parsed successfully with {} classes (including imports)", schema.classes.len());
}

/// Test that the country schema parses with txp: imports resolved
#[tokio::test]
async fn test_parse_country_schema_with_txp_imports() {
    let loader = SchemaLoader::new();
    let schema_path = get_repo_root().join("crates/model/symbolic/schemata/place/polity/country/schema.yaml");

    let schema = loader.load_file(&schema_path)
        .await
        .expect("Failed to load country schema");

    // Verify schema metadata
    assert_eq!(schema.id, "https://textpast.org/schema/place/polity/country");
    assert_eq!(schema.name, "country");
    assert!(schema.version.is_some(), "Schema should have version");

    // Check txp: imports are present in the original schema
    assert!(schema.imports.len() >= 2, "Schema should have at least 2 imports");
    assert!(
        schema.imports.iter().any(|i| i.starts_with("txp:")),
        "Schema should have txp: imports. Found: {:?}",
        schema.imports
    );

    // Check ISO3166Entity class exists
    assert!(
        schema.classes.contains_key("ISO3166Entity"),
        "Schema should have ISO3166Entity class"
    );
    let iso_class = &schema.classes["ISO3166Entity"];
    assert_eq!(
        iso_class.is_a.as_ref().unwrap(),
        "Entity",
        "ISO3166Entity should inherit from Entity"
    );

    println!("✓ Country schema parsed successfully with {} imports", schema.imports.len());
}

#[test]
fn test_parse_iso3166_instance_file() {
    let path = get_repo_root().join("crates/model/symbolic/schemata/place/polity/country/iso_3166_entity.yaml");
    let content = std::fs::read_to_string(&path)
        .expect("Failed to read instance file");

    let instance_data: serde_yaml::Value = serde_yaml::from_str(&content)
        .expect("Failed to parse instance YAML");

    // Check required metadata (NEW CONVENTION: real metadata, not commented out)
    assert_eq!(
        instance_data["id"].as_str().unwrap(),
        "https://textpast.org/instance/place/polity/country/iso_3166_entity"
    );
    assert_eq!(
        instance_data["schema"].as_str().unwrap(),
        "https://textpast.org/schema/place/polity/country"
    );
    assert_eq!(instance_data["name"].as_str().unwrap(), "iso_3166_entity");
    assert!(instance_data["version"].as_str().is_some(), "Version should be present");
    assert!(instance_data["created_on"].as_str().is_some(), "created_on should be present");
    assert!(instance_data["last_updated_on"].as_str().is_some(), "last_updated_on should be present");

    // Check instances array exists and is not empty (NEW CONVENTION: 'instances' key)
    let instances = instance_data["instances"].as_sequence().unwrap();
    assert!(!instances.is_empty());
    assert_eq!(instances.len(), 248); // All ISO 3166-1 countries

    // Verify first instance structure
    let first = &instances[0];
    assert!(first["id"].as_str().is_some());
    assert!(first["label"].as_str().is_some());
    assert!(first["tld"].as_str().is_some());
    assert!(first["exact_mappings"].as_sequence().is_some());
}

#[test]
fn test_iso3166_identifier_validation() {
    let path = get_repo_root().join("crates/model/symbolic/schemata/place/polity/country/iso_3166_entity.yaml");
    let content = std::fs::read_to_string(&path)
        .expect("Failed to read instance file");

    let instance_data: serde_yaml::Value = serde_yaml::from_str(&content)
        .expect("Failed to parse instance YAML");

    let instances = instance_data["instances"].as_sequence().unwrap();

    // All IDs should be exactly 2 uppercase letters (ISO 3166-1 alpha-2)
    // This validates the CountryCodeAlpha2Identifier type constraint
    let pattern = regex::Regex::new(r"^[A-Z]{2}$").unwrap();

    let mut valid_count = 0;
    let mut invalid_ids = Vec::new();

    for instance in instances {
        let id = instance["id"].as_str().unwrap();
        if pattern.is_match(id) {
            valid_count += 1;
        } else {
            invalid_ids.push(id.to_string());
        }
    }

    assert_eq!(
        valid_count,
        instances.len(),
        "All IDs should match CountryCodeAlpha2Identifier pattern. Invalid IDs: {:?}",
        invalid_ids
    );
}

#[test]
fn test_slot_usage_scoped_imports() {
    let parser = create_test_parser();
    let path = get_repo_root().join("crates/model/symbolic/schemata/place/polity/country/schema.yaml");
    let content = std::fs::read_to_string(&path)
        .expect("Failed to read country schema");

    let schema = parser.parse_str(&content, "yaml").expect("Failed to parse schema");
    let iso_class = &schema.classes["ISO3166Entity"];

    // Check that identifier slot usage has proper configuration
    // NEW CONVENTION: slot_usage can optionally specify which imports to search for range types
    if let Some(identifier_usage) = iso_class.slot_usage.get("identifier") {
        assert_eq!(
            identifier_usage.range.as_ref().unwrap(),
            "CountryCodeAlpha2Identifier",
            "Identifier should use CountryCodeAlpha2Identifier type"
        );
        assert_eq!(
            identifier_usage.required,
            Some(true),
            "Identifier should be required"
        );

        // The schema YAML has:
        // slot_usage:
        //   identifier:
        //     range: CountryCodeAlpha2Identifier
        //     imports:
        //       - txp:meta/identifier/identifier/schema
        //     required: true
        //
        // This means the LinkML service should only look for CountryCodeAlpha2Identifier
        // in txp:meta/identifier/identifier/schema, not in other imports.
        //
        // Note: The SlotDefinition type may need an 'imports' field to fully support this.
        // For now, we verify the range and required fields are correct.
    }
}

#[test]
fn test_all_schemas_parse_successfully() {
    // Test that all schema files in the schemata directory parse without errors
    let schema_paths = vec![
        "crates/model/symbolic/schemata/meta/entity/hyperentity/schema.yaml",
        "crates/model/symbolic/schemata/meta/identifier/identifier/schema.yaml",
        "crates/model/symbolic/schemata/meta/label/label/schema.yaml",
        "crates/model/symbolic/schemata/meta/description/description/schema.yaml",
        "crates/model/symbolic/schemata/meta/description/note/schema.yaml",
        "crates/model/symbolic/schemata/place/polity/country/schema.yaml",
    ];

    let parser = create_test_parser();
    let repo_root = get_repo_root();

    for path_str in schema_paths {
        let path = repo_root.join(path_str);
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Failed to read schema: {}", path_str));

        let schema = parser.parse_str(&content, "yaml")
            .unwrap_or_else(|_| panic!("Failed to parse schema: {}", path_str));

        // Verify basic schema structure
        assert!(!schema.id.is_empty(), "Schema {} should have an ID", path_str);
        assert!(!schema.name.is_empty(), "Schema {} should have a name", path_str);
        assert!(schema.id.starts_with("https://textpast.org/schema/"),
            "Schema {} ID should start with https://textpast.org/schema/", path_str);
    }
}

#[test]
fn test_schema_metadata_conventions() {
    // Test that schemas follow the new metadata conventions
    let parser = create_test_parser();
    let path = get_repo_root().join("crates/model/symbolic/schemata/place/polity/country/schema.yaml");
    let content = std::fs::read_to_string(&path)
        .expect("Failed to read country schema");

    let schema = parser.parse_str(&content, "yaml").expect("Failed to parse schema");

    // Check required metadata fields
    assert_eq!(schema.id, "https://textpast.org/schema/place/polity/country");
    assert_eq!(schema.name, "country");
    assert!(schema.version.is_some(), "Schema should have version");

    // Check that default_prefix is set
    assert_eq!(schema.default_prefix.as_deref(), Some("txp"), "Default prefix should be 'txp'");

    // Check that prefixes include txp
    assert!(schema.prefixes.contains_key("txp"), "Schema should define txp prefix");

    // PrefixDefinition is an enum, so we need to match on it
    use linkml_core::types::PrefixDefinition;
    match schema.prefixes.get("txp").unwrap() {
        PrefixDefinition::Simple(url) => {
            assert_eq!(url, "https://textpast.org/", "txp prefix should map to https://textpast.org/");
        }
        PrefixDefinition::Complex { prefix_reference, .. } => {
            assert_eq!(
                prefix_reference.as_deref(),
                Some("https://textpast.org/"),
                "txp prefix should map to https://textpast.org/"
            );
        }
    }
}

/// Comprehensive test: Load country schema with SchemaLoader and verify txp: import resolution
/// This is the MAIN test that validates the new TextPast/RootReal conventions
#[tokio::test]
async fn test_comprehensive_txp_import_resolution() {
    println!("\n=== Testing Comprehensive txp: Import Resolution ===\n");

    let loader = SchemaLoader::new();
    let schema_path = get_repo_root().join("crates/model/symbolic/schemata/place/polity/country/schema.yaml");

    // Load the schema - this should resolve all txp: imports
    let result = loader.load_file(&schema_path).await;

    match &result {
        Ok(schema) => {
            println!("✓ Schema loaded successfully");
            println!("  ID: {}", schema.id);
            println!("  Name: {}", schema.name);
            println!("  Version: {:?}", schema.version);
            println!("  Imports: {} total", schema.imports.len());

            // List all imports
            for (i, import) in schema.imports.iter().enumerate() {
                println!("    {}. {}", i + 1, import);
            }

            // Verify txp: imports are present
            let txp_imports: Vec<_> = schema.imports.iter()
                .filter(|i| i.starts_with("txp:"))
                .collect();
            println!("\n  txp: imports: {}", txp_imports.len());
            for imp in &txp_imports {
                println!("    - {}", imp);
            }

            // Verify classes
            println!("\n  Classes: {}", schema.classes.len());
            for class_name in schema.classes.keys() {
                println!("    - {}", class_name);
            }

            // Verify ISO3166Entity class and its slot_usage
            assert!(schema.classes.contains_key("ISO3166Entity"), "Should have ISO3166Entity class");
            let iso_class = &schema.classes["ISO3166Entity"];

            println!("\n  ISO3166Entity details:");
            println!("    Parent class: {:?}", iso_class.is_a);
            println!("    Slots: {}", iso_class.slots.len());

            println!("    Slot usage:");
            for (slot_name, usage) in &iso_class.slot_usage {
                println!("      - {}: range={:?}, required={:?}",
                    slot_name,
                    usage.range,
                    usage.required
                );
            }

            // NEW CONVENTION: Check scoped imports for slot ranges
            if let Some(identifier_usage) = iso_class.slot_usage.get("identifier") {
                println!("\n  Identifier slot configuration:");
                println!("    Range: {:?}", identifier_usage.range);
                println!("    Required: {:?}", identifier_usage.required);

                // Verify the range is CountryCodeAlpha2Identifier
                assert_eq!(
                    identifier_usage.range.as_deref(),
                    Some("CountryCodeAlpha2Identifier"),
                    "Identifier should use CountryCodeAlpha2Identifier type"
                );
                assert_eq!(
                    identifier_usage.required,
                    Some(true),
                    "Identifier should be required"
                );
            }

            println!("\n✓ All validations passed!");
        }
        Err(e) => {
            eprintln!("✗ Failed to load schema: {:?}", e);
            panic!("Schema loading failed: {}", e);
        }
    }

    result.expect("Schema should load successfully");
}

/// Test that all schemas in the schemata directory can be loaded
#[tokio::test]
async fn test_all_schemas_load_successfully() {
    println!("\n=== Testing All Schemas Load Successfully ===\n");

    let loader = SchemaLoader::new();
    let schema_paths = vec![
        "crates/model/symbolic/schemata/meta/entity/hyperentity/schema.yaml",
        "crates/model/symbolic/schemata/meta/identifier/identifier/schema.yaml",
        "crates/model/symbolic/schemata/meta/label/label/schema.yaml",
        "crates/model/symbolic/schemata/place/polity/country/schema.yaml",
    ];

    let mut passed = 0;
    let mut failed = 0;
    let repo_root = get_repo_root();

    for path_str in schema_paths {
        let path = repo_root.join(path_str);
        print!("  Loading {}... ", path.file_name().unwrap().to_string_lossy());

        match loader.load_file(&path).await {
            Ok(schema) => {
                println!("✓ (ID: {})", schema.id);
                passed += 1;
            }
            Err(e) => {
                println!("✗ Error: {}", e);
                failed += 1;
            }
        }
    }

    println!("\n  Results: {} passed, {} failed", passed, failed);
    assert_eq!(failed, 0, "All schemas should load successfully");
}

