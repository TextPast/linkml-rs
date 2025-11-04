//! Example demonstrating validation of country schema and instances
//!
//! This example shows how the LinkML service validates:
//! 1. The ISO3166Entity instances against their schema
//! 2. CountryCodeAlpha2Identifier values against permissible values from ISO3166Entity instances
//!
//! The key pattern demonstrated here is how instance files (iso_3166_entity.yaml) provide
//! permissible values for schema validation when referenced as ranges.
//!
//! Key Conventions:
//! - Instance imports: txp:place/polity/country/iso_3166_entity/instance
//! - Schema imports: txp:place/polity/country/schema
//! - range_type: instance triggers instance-based validation
//! - range_properties: [id] specifies which field to validate against

use linkml_core::error::Result;
use linkml_service::parser::factory::create_dev_schema_loader;
use linkml_service::validator::{InstanceLoader, InstanceResolver, ValidationEngine};
use timestamp_service::wiring::wire_timestamp;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== LinkML Country Schema Validation Example ===\n");

    let workspace_root = std::env::current_dir()?;
    let schemata_dir = workspace_root.join("crates/model/symbolic/schemata");

    if !schemata_dir.exists() {
        println!("⚠ Schemata directory not found. Skipping example.");
        return Ok(());
    }

    let country_schema_path = schemata_dir.join("place/polity/country/schema.yaml");
    let country_instances_path = schemata_dir.join("place/polity/country/iso_3166_entity.yaml");
    let identifier_schema_path = schemata_dir.join("meta/identifier/identifier/schema.yaml");

    println!("Loading schemas and instances:");
    println!("  - Country schema: {}", country_schema_path.display());
    println!("  - Country instances: {}", country_instances_path.display());
    println!("  - Identifier schema: {}", identifier_schema_path.display());
    println!();

    // Initialize services
    let loader = create_dev_schema_loader();
    let timestamp_service = wire_timestamp().into_arc();
    let instance_loader = std::sync::Arc::new(InstanceLoader::new(timestamp_service));
    let resolver = InstanceResolver::new(schemata_dir.clone(), instance_loader);

    // ========================================================================
    // Part 1: Validate ISO3166Entity instances against their schema
    // ========================================================================

    println!("Part 1: Validating ISO3166Entity instances");
    println!("{}", "=".repeat(50));

    // Load the country schema
    let country_schema = loader.load_file(&country_schema_path).await?;

    // Load the country instances manually
    let instances_content = tokio::fs::read_to_string(&country_instances_path).await?;
    let instances_yaml: serde_yaml::Value = serde_yaml::from_str(&instances_content)?;

    let instances_array = instances_yaml.get("instances")
        .and_then(|v| v.as_sequence())
        .expect("Instance file should have instances section");

    println!("Loaded {} country instances", instances_array.len());

    // Validate each instance against the ISO3166Entity class
    let mut valid_count = 0;
    let mut invalid_count = 0;

    let engine = ValidationEngine::new(&country_schema)?;

    for (idx, instance) in instances_array.iter().enumerate() {
        // Convert YAML to JSON for validation
        let instance_json: serde_json::Value = serde_yaml::from_value(instance.clone())?;

        let validation_result = engine
            .validate_as_class(&instance_json, "ISO3166Entity", None)
            .await?;

        if validation_result.valid {
            valid_count += 1;
            if idx < 3 {
                // Show first few valid instances
                println!(
                    "  ✓ Valid: {} - {}",
                    instance.get("id").and_then(|v| v.as_str()).unwrap_or("?"),
                    instance.get("label").and_then(|v| v.as_str()).unwrap_or("?")
                );
            }
        } else {
            invalid_count += 1;
            let errors: Vec<_> = validation_result.errors().collect();
            println!("  ✗ Invalid instance at index {}: {} errors", idx, errors.len());
            for error in errors.iter().take(3) {
                println!("      - {}", error.message);
            }
        }
    }

    println!("\nValidation Summary:");
    println!("  - Valid instances: {}", valid_count);
    println!("  - Invalid instances: {}", invalid_count);
    println!();

    // ========================================================================
    // Part 2: Validate CountryCodeAlpha2Identifier with instance-based permissible values
    // ========================================================================

    println!("Part 2: Validating CountryCodeAlpha2Identifier");
    println!("{}", "=".repeat(50));

    // Load the identifier schema
    let identifier_schema = loader.load_file(&identifier_schema_path).await?;

    // Get the CountryCodeAlpha2Identifier class and slot
    let class = identifier_schema.classes.get("CountryCodeAlpha2Identifier")
        .expect("CountryCodeAlpha2Identifier should exist");
    let slot = class.slot_usage.get("identifier")
        .expect("identifier slot should exist");

    println!("Schema configuration:");
    println!("  range: {:?}", slot.range);
    println!("  range_type: {:?}", slot.range_type);
    println!("  range_properties: {:?}", slot.range_properties);

    // Get valid IDs from instance resolver
    if let Some(valid_ids) = resolver.get_valid_ids_for_slot(slot, &identifier_schema).await? {
        println!("\n✓ Loaded {} permissible country codes from instances", valid_ids.len());
        println!("  Examples: {:?}", &valid_ids[..5.min(valid_ids.len())]);
    }

    println!();

    // Test cases for CountryCodeAlpha2Identifier validation
    let test_cases = vec![
        ("US", true, "Valid ISO 3166-1 alpha-2 code"),
        ("GB", true, "Valid ISO 3166-1 alpha-2 code"),
        ("DE", true, "Valid ISO 3166-1 alpha-2 code"),
        ("XX", false, "Invalid - not in ISO 3166-1"),
        ("ZZ", false, "Invalid - not in ISO 3166-1"),
        ("usa", false, "Invalid - must be uppercase"),
        ("U", false, "Invalid - must be exactly 2 characters"),
        ("USA", false, "Invalid - alpha-3 code, not alpha-2"),
    ];

    println!("Testing CountryCodeAlpha2Identifier validation:");

    for (code, expected_valid, description) in test_cases {
        // Test with instance resolver
        let is_valid = resolver.validate_instance_value(code, slot, &identifier_schema).await?;
        let status = if is_valid == expected_valid { "✓" } else { "✗ UNEXPECTED" };

        println!(
            "  {} Code '{}': {} - {}",
            status,
            code,
            if is_valid { "Valid  " } else { "Invalid" },
            description
        );
    }

    println!();

    // ========================================================================
    // Part 3: Demonstrate pattern validation
    // ========================================================================

    println!("Part 3: Pattern Validation");
    println!("{}", "=".repeat(50));

    // The country_code_alpha2_identifier_pattern: [A-Z]{2}
    let pattern_test = "GB";
    println!("Testing pattern matching for: '{}'", pattern_test);

    // Test with instance resolver
    let is_valid = resolver.validate_instance_value(pattern_test, slot, &identifier_schema).await?;

    if is_valid {
        println!("  ✓ Pattern matches and value is in instance data");
        println!("  - Value '{}' is a valid ISO 3166-1 alpha-2 code", pattern_test);
    } else {
        println!("  ✗ Pattern validation failed");
    }

    println!();

    // ========================================================================
    // Part 4: Summary
    // ========================================================================

    println!("Part 4: Summary");
    println!("{}", "=".repeat(50));

    println!("This example demonstrates:");
    println!("  1. Loading instance files (iso_3166_entity.yaml)");
    println!("  2. Validating instances against their schema");
    println!("  3. Using instance-based validation with range_type: instance");
    println!("  4. Validating values against permissible instance IDs");
    println!("  5. Pattern validation combined with instance validation");
    println!("\nKey Conventions:");
    println!("  - Instance imports: txp:place/polity/country/iso_3166_entity/instance");
    println!("  - range_type: instance triggers instance-based validation");
    println!("  - range_properties: [id] specifies field to validate against");
    println!("  - Values must match actual instance data");

    println!();
    println!("=== Example Complete ===");

    Ok(())
}

