//! Test example to verify LinkML instance-based validation is working
//!
//! This example specifically tests the pattern where:
//! 1. An instance file (iso_3166_entity.yaml) contains permissible values
//! 2. A schema class (CountryCodeAlpha2Identifier) references these instances as its range
//! 3. Values are validated against the permissible values from the instance file
//!
//! Key Conventions:
//! - Instance imports MUST end with `/instance`: txp:place/polity/country/iso_3166_entity/instance
//! - Schema imports MUST end with `/schema`: txp:meta/identifier/identifier/schema
//! - `range_type: instance` triggers instance-based validation
//! - `range_properties: [id]` specifies which field to validate against

use linkml_core::error::Result;
use linkml_service::parser::factory::create_dev_schema_loader;
use linkml_service::validator::{InstanceLoader, InstanceResolver, ValidationEngine};
use serde_json::json;
use timestamp_service::wiring::wire_timestamp;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Testing LinkML Instance-Based Validation ===\n");

    let workspace_root = std::env::current_dir()?;
    let schemata_dir = workspace_root.join("crates/model/symbolic/schemata");

    if !schemata_dir.exists() {
        println!("⚠ Schemata directory not found. Skipping example.");
        return Ok(());
    }

    // Initialize services
    let loader = create_dev_schema_loader();
    let timestamp_service = wire_timestamp().into_arc();
    let instance_loader = std::sync::Arc::new(InstanceLoader::new(timestamp_service));
    let resolver = InstanceResolver::new(schemata_dir.clone(), instance_loader);

    // Test 1: Load and validate country instances
    println!("Test 1: Loading ISO3166Entity instances");
    println!("{}", "-".repeat(40));

    let country_schema_path = schemata_dir.join("place/polity/country/schema.yaml");
    let country_instances_path = schemata_dir.join("place/polity/country/iso_3166_entity.yaml");

    // Load schema
    let country_schema = loader.load_file(&country_schema_path).await?;
    println!("✓ Loaded country schema successfully");

    // Check if schema contains ISO3166Entity class
    if country_schema.classes.contains_key("ISO3166Entity") {
        println!("✓ Schema contains ISO3166Entity class");
    } else {
        println!("✗ Schema missing ISO3166Entity class");
    }

    // Load instances using YAML
    let instances_content = tokio::fs::read_to_string(&country_instances_path).await?;
    let instances_yaml: serde_yaml::Value = serde_yaml::from_str(&instances_content)?;

    if let Some(instances_array) = instances_yaml.get("instances").and_then(|v| v.as_sequence()) {
        println!("✓ Loaded {} country instances", instances_array.len());

        // Show a few examples
        println!("\nExample instances:");
        for instance in instances_array.iter().take(3) {
            if let (Some(id), Some(label)) = (
                instance.get("id").and_then(|v| v.as_str()),
                instance.get("label").and_then(|v| v.as_str()),
            ) {
                println!("  - {}: {}", id, label);
            }
        }
    }

    println!();

    // Test 2: Validate identifier against permissible values
    println!("Test 2: Validating CountryCodeAlpha2Identifier");
    println!("{}", "-".repeat(40));

    let identifier_schema_path = schemata_dir.join("meta/identifier/identifier/schema.yaml");

    // Load identifier schema
    let identifier_schema = loader.load_file(&identifier_schema_path).await?;
    println!("✓ Loaded identifier schema");

    // Check for CountryCodeAlpha2Identifier class
    if let Some(class_def) = identifier_schema.classes.get("CountryCodeAlpha2Identifier") {
        println!("✓ Found CountryCodeAlpha2Identifier class");

        // Check slot usage for identifier
        if let Some(slot_usage) = class_def.slot_usage.get("identifier") {
            println!("  range: {:?}", slot_usage.range);
            println!("  range_type: {:?}", slot_usage.range_type);
            println!("  range_properties: {:?}", slot_usage.range_properties);

            if slot_usage.range.as_deref() == Some("ISO3166Entity") {
                println!("✓ Range correctly references ISO3166Entity instances!");
            }

            // Test validation with actual values using instance resolver
            println!("\nValidating test values:");

            let test_values = vec![
                ("US", true, "United States"),
                ("GB", true, "United Kingdom"),
                ("FR", true, "France"),
                ("XX", false, "Invalid code"),
                ("12", false, "Numeric not allowed"),
            ];

            for (code, should_be_valid, description) in test_values {
                let is_valid = resolver.validate_instance_value(code, slot_usage, &identifier_schema).await?;

                let status = if is_valid == should_be_valid { "✓" } else { "✗" };
                println!(
                    "  {} {}: {} ({})",
                    status,
                    code,
                    if is_valid { "Valid  " } else { "Invalid" },
                    description
                );
            }

            // Also test with ValidationEngine
            println!("\nValidating with ValidationEngine:");
            let engine = ValidationEngine::new(&identifier_schema)?;

            for (code, should_be_valid, description) in &[
                ("US", true, "United States"),
                ("XX", false, "Invalid code"),
            ] {
                let test_data = json!({
                    "identifier": code
                });

                let report = engine.validate_as_class(&test_data, "CountryCodeAlpha2Identifier", None).await?;
                let is_valid = report.valid;
                let status = if is_valid == *should_be_valid { "✓" } else { "✗" };

                println!(
                    "  {} {}: {} ({})",
                    status,
                    code,
                    if is_valid { "Valid  " } else { "Invalid" },
                    description
                );

                let errors: Vec<_> = report.errors().collect();
                if !report.valid && !errors.is_empty() {
                    println!("      Error: {}", errors[0].message);
                }
            }
        }
    }

    println!();

    // Test 3: Check pattern validation through LinkML (not direct regex)
    println!("Test 3: Pattern Validation Through LinkML");
    println!("{}", "-".repeat(40));

    // Test pattern validation through ValidationEngine
    // This verifies that patterns are correctly loaded and applied by LinkML
    let pattern_tests = vec![
        ("US", true, "Valid: matches pattern [A-Z]{2}"),
        ("GB", true, "Valid: matches pattern [A-Z]{2}"),
        ("us", false, "Invalid: lowercase doesn't match pattern"),
        ("USA", false, "Invalid: three characters (pattern requires 2)"),
        ("U", false, "Invalid: too short (pattern requires 2)"),
        ("U1", false, "Invalid: contains digit"),
    ];

    println!("Testing pattern validation through ValidationEngine:");
    println!("Pattern: [A-Z]{{2}} (from schema definition)");

    let pattern_engine = ValidationEngine::new(&identifier_schema)?;
    let mut pattern_passed = 0;
    let mut pattern_failed = 0;

    for (value, should_be_valid, description) in pattern_tests {
        let test_data = json!({
            "identifier": value
        });

        let report = pattern_engine.validate_as_class(&test_data, "CountryCodeAlpha2Identifier", None).await?;
        let is_valid = report.valid;

        let status = if is_valid == should_be_valid {
            pattern_passed += 1;
            "✓"
        } else {
            pattern_failed += 1;
            "✗ UNEXPECTED"
        };

        println!(
            "  {} '{}': {} - {}",
            status,
            value,
            if is_valid { "Valid  " } else { "Invalid" },
            description
        );

        let errors: Vec<_> = report.errors().collect();
        if !is_valid && !errors.is_empty() {
            println!("      Error: {}", errors[0].message);
        }
    }

    println!("\nPattern validation summary: {} passed, {} failed", pattern_passed, pattern_failed);

    println!();
    println!("=== Test Complete ===");

    // Summary
    println!("\nSummary:");
    println!("This test verifies that the LinkML service can:");
    println!("1. Load instance files (iso_3166_entity.yaml)");
    println!("2. Use instance values as permissible values for validation");
    println!("3. Validate identifiers against both patterns and permissible values");
    println!("4. Use the InstanceResolver for efficient instance-based validation");
    println!("\nKey Conventions:");
    println!("  - Instance imports end with /instance");
    println!("  - range_type: instance triggers validation");
    println!("  - range_properties specifies field name (default: id)");
    println!("  - Values must match actual instance data");

    Ok(())
}

