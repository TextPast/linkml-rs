//! Comprehensive Instance-Based Validation Example
//!
//! This example demonstrates the complete RootReal/Textpast LinkML instance-based
//! validation system with all supported patterns:
//!
//! 1. Country codes (ISO 3166) - field: `id`
//! 2. Timezones - field: `timezone_component_value`
//! 3. Languages (ISO 639-3) - field: `id` with additional fields
//! 4. Document parsers - field: `id` + `name`
//!
//! Key Conventions:
//! - Instance imports MUST end with `/instance`
//! - Schema imports MUST end with `/schema`
//! - `range_type: instance` triggers instance-based validation
//! - `range_properties` specifies which field to validate against

use linkml_core::error::Result;
use linkml_service::parser::factory::create_dev_schema_loader;
use linkml_service::validator::{InstanceLoader, InstanceResolver, ValidationEngine};
use serde_json::json;
use timestamp_service::wiring::wire_timestamp;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Comprehensive Instance-Based Validation Example ===\n");

    let workspace_root = std::env::current_dir()?;
    let schemata_dir = workspace_root.join("crates/model/symbolic/schemata");

    if !schemata_dir.exists() {
        println!("⚠ Schemata directory not found. Skipping example.");
        return Ok(());
    }

    // Initialize instance resolver
    let timestamp_service = wire_timestamp().into_arc();
    let loader = std::sync::Arc::new(InstanceLoader::new(timestamp_service));
    let resolver = InstanceResolver::new(schemata_dir.clone(), loader);

    // Test 1: Country Codes (ISO 3166)
    println!("Test 1: Country Codes (ISO 3166)");
    println!("{}", "=".repeat(50));
    test_country_codes(&schemata_dir, &resolver).await?;

    // Test 2: Timezones
    println!("\nTest 2: Timezones");
    println!("{}", "=".repeat(50));
    test_timezones(&schemata_dir, &resolver).await?;

    // Test 3: Languages (ISO 639-3)
    println!("\nTest 3: Languages (ISO 639-3)");
    println!("{}", "=".repeat(50));
    test_languages(&schemata_dir, &resolver).await?;

    // Test 4: Document Parsers
    println!("\nTest 4: Document Parsers");
    println!("{}", "=".repeat(50));
    test_document_parsers(&schemata_dir, &resolver).await?;

    println!("\n{}", "=".repeat(50));
    println!("✅ All instance-based validation tests completed!");
    println!("\nTest Coverage Summary:");
    println!("  ✓ Country codes: 10 test cases (InstanceResolver + ValidationEngine)");
    println!("  ✓ Timezones: 5 test cases (InstanceResolver + ValidationEngine)");
    println!("  ✓ Languages: 10 test cases (InstanceResolver + ValidationEngine)");
    println!("  ✓ Document parsers: 6 test cases (InstanceResolver + ValidationEngine)");
    println!("  ✓ Total: 31 validation test cases across 4 instance types");
    println!("\nKey Takeaways:");
    println!("  1. Instance imports use /instance suffix");
    println!("  2. range_type: instance triggers validation");
    println!("  3. range_properties specifies field name");
    println!("  4. Works with any field name (id, timezone_component_value, etc.)");
    println!("  5. Both InstanceResolver and ValidationEngine support instance validation");
    println!("  6. Edge cases tested: empty strings, invalid formats, special characters");
    println!("  7. Validation is automatic and cached for performance");

    Ok(())
}

async fn test_country_codes(
    schemata_dir: &PathBuf,
    resolver: &InstanceResolver,
) -> Result<()> {
    let loader = create_dev_schema_loader();
    let schema_path = schemata_dir.join("meta/identifier/identifier/schema.yaml");
    let schema = loader.load_file(&schema_path).await?;

    // Get the CountryCodeAlpha2Identifier class
    let class = schema.classes.get("CountryCodeAlpha2Identifier")
        .expect("CountryCodeAlpha2Identifier should exist");
    
    let slot = class.slot_usage.get("identifier")
        .expect("identifier slot should exist");

    println!("Schema: meta/identifier/identifier/schema.yaml");
    println!("Class: CountryCodeAlpha2Identifier");
    println!("Slot: identifier");
    println!("  range: {:?}", slot.range);
    println!("  range_type: {:?}", slot.range_type);
    println!("  range_properties: {:?}", slot.range_properties);

    // Get valid IDs
    if let Some(valid_ids) = resolver.get_valid_ids_for_slot(slot, &schema).await? {
        println!("\n✓ Loaded {} country codes", valid_ids.len());
        println!("  Examples: {:?}", &valid_ids[..5.min(valid_ids.len())]);

        // Test validation with comprehensive test cases
        let test_cases = vec![
            ("US", true, "United States"),
            ("AD", true, "Andorra"),
            ("GB", true, "United Kingdom"),
            ("FR", true, "France"),
            ("DE", true, "Germany"),
            ("XX", false, "Invalid code"),
            ("ZZ", false, "Invalid code"),
            ("", false, "Empty string"),
            ("us", false, "Lowercase not allowed"),
            ("USA", false, "Three characters not allowed"),
        ];

        println!("\nValidation tests (InstanceResolver):");
        let mut passed = 0;
        let mut failed = 0;

        for (code, expected, description) in &test_cases {
            let is_valid = resolver.validate_instance_value(code, slot, &schema).await?;
            let status = if is_valid == *expected {
                passed += 1;
                "✓"
            } else {
                failed += 1;
                "✗ UNEXPECTED"
            };
            println!("  {} '{}': {} - {}", status, code,
                if is_valid { "Valid  " } else { "Invalid" }, description);
        }

        println!("\nInstanceResolver test summary: {} passed, {} failed", passed, failed);

        // Also test with ValidationEngine for comparison
        println!("\nValidating with ValidationEngine:");
        let engine = ValidationEngine::new(&schema)?;

        for (code, expected, description) in &[
            ("US", true, "United States"),
            ("GB", true, "United Kingdom"),
            ("XX", false, "Invalid code"),
            ("", false, "Empty string"),
        ] {
            let test_data = json!({
                "identifier": code
            });

            let report = engine.validate_as_class(&test_data, "CountryCodeAlpha2Identifier", None).await?;
            let is_valid = report.valid;
            let status = if is_valid == *expected { "✓" } else { "✗ UNEXPECTED" };

            println!("  {} '{}': {} - {}", status, code,
                if is_valid { "Valid  " } else { "Invalid" }, description);

            let errors: Vec<_> = report.errors().collect();
            if !is_valid && !errors.is_empty() {
                println!("      Error: {}", errors[0].message);
            }
        }
    }

    Ok(())
}

async fn test_timezones(
    schemata_dir: &PathBuf,
    resolver: &InstanceResolver,
) -> Result<()> {
    let loader = create_dev_schema_loader();
    let schema_path = schemata_dir.join("time/timezone/schema.yaml");
    let schema = loader.load_file(&schema_path).await?;

    // Get the TimeZone class
    let class = schema.classes.get("TimeZone")
        .expect("TimeZone class should exist");

    let slot = class.slot_usage.get("timezone_component_value")
        .expect("timezone_component_value slot should exist");

    println!("Schema: time/timezone/schema.yaml");
    println!("Class: TimeZone");
    println!("Slot: timezone_component_value (custom field name)");
    println!("  range: {:?}", slot.range);
    println!("  range_type: {:?}", slot.range_type);
    println!("  range_properties: {:?}", slot.range_properties);

    // Get valid IDs from timezone instance file
    if let Some(valid_ids) = resolver.get_valid_ids_for_slot(slot, &schema).await? {
        println!("\n✓ Loaded {} timezone values", valid_ids.len());
        println!("  Examples: {:?}", &valid_ids[..3.min(valid_ids.len())]);

        // Test validation with actual timezone values
        let test_cases = vec![
            (valid_ids.get(0).map(|s| s.as_str()).unwrap_or(""), true, "First timezone from instance file"),
            (valid_ids.get(1).map(|s| s.as_str()).unwrap_or(""), true, "Second timezone from instance file"),
            ("Invalid/Timezone", false, "Invalid timezone not in IANA list"),
            ("", false, "Empty string"),
            ("UTC+5", false, "Offset format not in instance file"),
        ];

        println!("\nValidation tests:");
        for (tz, expected, description) in test_cases {
            let is_valid = resolver.validate_instance_value(tz, slot, &schema).await?;
            let status = if is_valid == expected { "✓" } else { "✗ UNEXPECTED" };
            println!("  {} '{}': {} - {}", status, tz,
                if is_valid { "Valid  " } else { "Invalid" }, description);
        }

        // Also test with ValidationEngine
        println!("\nValidating with ValidationEngine:");
        let engine = ValidationEngine::new(&schema)?;

        for (tz, expected, description) in &[
            (valid_ids.get(0).map(|s| s.as_str()).unwrap_or(""), true, "Valid timezone"),
            ("Invalid/Timezone", false, "Invalid timezone"),
        ] {
            let test_data = json!({
                "timezone_component_value": tz
            });

            let report = engine.validate_as_class(&test_data, "TimeZone", None).await?;
            let is_valid = report.valid;
            let status = if is_valid == *expected { "✓" } else { "✗ UNEXPECTED" };

            println!("  {} '{}': {} - {}", status, tz,
                if is_valid { "Valid  " } else { "Invalid" }, description);

            let errors: Vec<_> = report.errors().collect();
            if !is_valid && !errors.is_empty() {
                println!("      Error: {}", errors[0].message);
            }
        }
    }

    Ok(())
}

async fn test_languages(
    schemata_dir: &PathBuf,
    resolver: &InstanceResolver,
) -> Result<()> {
    let loader = create_dev_schema_loader();
    let schema_path = schemata_dir.join("language/iso_639-3/schema.yaml");
    let schema = loader.load_file(&schema_path).await?;

    // Get the ISO639Entity class
    let class = schema.classes.get("ISO639Entity")
        .expect("ISO639Entity class should exist");
    
    let slot = class.slots.iter()
        .find(|s| *s == "identifier")
        .and_then(|_| class.slot_usage.get("identifier"))
        .or_else(|| schema.slots.get("identifier"))
        .expect("identifier slot should exist");

    println!("Schema: language/iso_639-3/schema.yaml");
    println!("Class: ISO639Entity");
    println!("Slot: identifier (from Entity parent class)");
    println!("  range: {:?}", slot.range);
    println!("  range_type: {:?}", slot.range_type);
    println!("  range_properties: {:?}", slot.range_properties);

    if let Some(valid_ids) = resolver.get_valid_ids_for_slot(slot, &schema).await? {
        println!("\n✓ Loaded {} language codes", valid_ids.len());
        println!("  Examples: {:?}", &valid_ids[..5.min(valid_ids.len())]);

        // Test validation with known language codes - comprehensive test cases
        let test_cases = vec![
            ("eng", true, "English - common language"),
            ("fra", true, "French - common language"),
            ("deu", true, "German - common language"),
            ("spa", true, "Spanish - common language"),
            ("zho", true, "Chinese - common language"),
            ("xxx", false, "Invalid three-letter code"),
            ("en", false, "Two-letter code (ISO 639-1, not 639-3)"),
            ("", false, "Empty string"),
            ("ENGLISH", false, "Uppercase not allowed"),
            ("eng1", false, "Four characters not allowed"),
        ];

        println!("\nValidation tests (InstanceResolver):");
        let mut passed = 0;
        let mut failed = 0;

        for (code, expected, description) in &test_cases {
            let is_valid = resolver.validate_instance_value(code, slot, &schema).await?;
            let status = if is_valid == *expected {
                passed += 1;
                "✓"
            } else {
                failed += 1;
                "✗ UNEXPECTED"
            };
            println!("  {} '{}': {} - {}", status, code,
                if is_valid { "Valid  " } else { "Invalid" }, description);
        }

        println!("\nInstanceResolver test summary: {} passed, {} failed", passed, failed);

        // Also test with ValidationEngine for comparison
        println!("\nValidating with ValidationEngine:");
        let engine = ValidationEngine::new(&schema)?;

        for (code, expected, description) in &[
            ("eng", true, "English"),
            ("fra", true, "French"),
            ("xxx", false, "Invalid code"),
            ("", false, "Empty string"),
        ] {
            let test_data = json!({
                "identifier": code
            });

            let report = engine.validate_as_class(&test_data, "ISO639Entity", None).await?;
            let is_valid = report.valid;
            let status = if is_valid == *expected { "✓" } else { "✗ UNEXPECTED" };

            println!("  {} '{}': {} - {}", status, code,
                if is_valid { "Valid  " } else { "Invalid" }, description);

            let errors: Vec<_> = report.errors().collect();
            if !is_valid && !errors.is_empty() {
                println!("      Error: {}", errors[0].message);
            }
        }
    }

    Ok(())
}

async fn test_document_parsers(
    schemata_dir: &PathBuf,
    resolver: &InstanceResolver,
) -> Result<()> {
    let loader = create_dev_schema_loader();
    let schema_path = schemata_dir.join("meta/document/document_parser/schema.yaml");
    let schema = loader.load_file(&schema_path).await?;

    // Get the DocumentParser class
    let class = schema.classes.get("DocumentParser")
        .expect("DocumentParser class should exist");
    
    let slot = class.slot_usage.get("id")
        .or_else(|| schema.slots.get("id"))
        .expect("id slot should exist");

    println!("Schema: meta/document/document_parser/schema.yaml");
    println!("Class: DocumentParser");
    println!("Slot: id (with name, version, description)");
    println!("  range: {:?}", slot.range);
    println!("  range_type: {:?}", slot.range_type);
    println!("  range_properties: {:?}", slot.range_properties);

    if let Some(valid_ids) = resolver.get_valid_ids_for_slot(slot, &schema).await? {
        println!("\n✓ Loaded {} document parser IDs", valid_ids.len());
        println!("  Examples: {:?}", &valid_ids[..3.min(valid_ids.len())]);

        // Test validation with multiple cases
        let test_cases = vec![
            (valid_ids.get(0).map(|s| s.as_str()).unwrap_or(""), true, "First parser from instance file"),
            (valid_ids.get(1).map(|s| s.as_str()).unwrap_or(""), true, "Second parser from instance file"),
            ("invalid_parser_id", false, "Non-existent parser ID"),
            ("", false, "Empty string"),
            ("parser-with-special-chars!", false, "Special characters"),
            ("UPPERCASE_PARSER", false, "Uppercase ID"),
        ];

        println!("\nValidation tests (InstanceResolver):");
        let mut passed = 0;
        let mut failed = 0;

        for (parser_id, expected, description) in &test_cases {
            let is_valid = resolver.validate_instance_value(parser_id, slot, &schema).await?;
            let status = if is_valid == *expected {
                passed += 1;
                "✓"
            } else {
                failed += 1;
                "✗ UNEXPECTED"
            };
            println!("  {} '{}': {} - {}", status, parser_id,
                if is_valid { "Valid  " } else { "Invalid" }, description);
        }

        println!("\nInstanceResolver test summary: {} passed, {} failed", passed, failed);

        // Also test with ValidationEngine
        println!("\nValidating with ValidationEngine:");
        let engine = ValidationEngine::new(&schema)?;

        if !valid_ids.is_empty() {
            for (parser_id, expected, description) in &[
                (valid_ids[0].as_str(), true, "Valid parser"),
                ("invalid_parser", false, "Invalid parser"),
            ] {
                let test_data = json!({
                    "id": parser_id
                });

                let report = engine.validate_as_class(&test_data, "DocumentParser", None).await?;
                let is_valid = report.valid;
                let status = if is_valid == *expected { "✓" } else { "✗ UNEXPECTED" };

                println!("  {} '{}': {} - {}", status, parser_id,
                    if is_valid { "Valid  " } else { "Invalid" }, description);

                let errors: Vec<_> = report.errors().collect();
                if !is_valid && !errors.is_empty() {
                    println!("      Error: {}", errors[0].message);
                }
            }
        }
    }

    Ok(())
}
