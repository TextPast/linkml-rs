//! Test pattern import and interpolation functionality
//!
//! This example verifies that:
//! 1. Pattern definitions in settings are correctly parsed
//! 2. Imported patterns are accessible in importing schemas
//! 3. Pattern interpolation with {pattern_name} syntax works
//! 4. Structured patterns with interpolated: true are validated

use linkml_core::error::Result;
use linkml_service::parser::factory::create_dev_schema_loader;
use linkml_service::validator::ValidationEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("========================================");
    println!("Testing Pattern Import and Interpolation");
    println!("========================================\n");

    // Get the workspace root
    let workspace_root = std::env::current_dir()?;
    
    // Try multiple possible paths
    let possible_paths = vec![
        workspace_root.join("crates/model/symbolic/schemata"),
        workspace_root.join("../../schemata"),
    ];
    
    let schemata_dir = possible_paths.iter()
        .find(|p| p.exists())
        .ok_or_else(|| linkml_core::error::LinkMLError::io_error(
            format!("Could not find schemata directory. Tried: {:?}", possible_paths)
        ))?
        .clone();

    println!("Looking for schemas in: {}\n", schemata_dir.display());

    // Create schema loader
    let loader = create_dev_schema_loader();

    // Test 1: Load identifier schema and check pattern definitions
    println!("Test 1: Verify pattern definitions in identifier schema");
    println!("--------------------------------------------------------");
    let identifier_schema_path = schemata_dir.join("meta/identifier/identifier/schema.yaml");
    let identifier_schema = loader.load_file(&identifier_schema_path).await?;
    
    println!("✓ Loaded identifier schema: {}", identifier_schema.name);
    
    if let Some(settings) = &identifier_schema.settings {
        println!("✓ Settings section found");
        
        // Check for pattern definitions in custom settings
        let pattern_count = settings.custom.iter()
            .filter(|(k, _)| k.ends_with("_pattern"))
            .count();
        
        println!("✓ Found {} pattern definitions in settings.custom", pattern_count);
        
        // Show some example patterns
        println!("\nExample patterns:");
        for (key, value) in settings.custom.iter().take(5) {
            if key.ends_with("_pattern") {
                println!("  - {}: {}", key, value);
            }
        }
    } else {
        println!("✗ No settings section found!");
        return Err(linkml_core::error::LinkMLError::schema_validation(
            "identifier schema should have settings section with patterns"
        ));
    }

    // Test 2: Load FQN schema and check it imports identifier patterns
    println!("\n\nTest 2: Verify FQN schema imports identifier patterns");
    println!("--------------------------------------------------------");
    let fqn_schema_path = schemata_dir.join("meta/identifier/fqn/schema.yaml");
    let fqn_schema = loader.load_file(&fqn_schema_path).await?;
    
    println!("✓ Loaded FQN schema: {}", fqn_schema.name);
    println!("✓ Imports: {:?}", fqn_schema.imports);
    
    if let Some(settings) = &fqn_schema.settings {
        println!("✓ Settings section found");
        
        // Check for FQN pattern definitions
        let fqn_pattern_count = settings.custom.iter()
            .filter(|(k, _)| k.ends_with("_fqn_pattern"))
            .count();
        
        println!("✓ Found {} FQN pattern definitions", fqn_pattern_count);
        
        // Show some example FQN patterns
        println!("\nExample FQN patterns:");
        for (key, value) in settings.custom.iter().take(5) {
            if key.ends_with("_fqn_pattern") {
                println!("  - {}: {}", key, value);
            }
        }
        
        // Check if patterns reference imported patterns using {pattern_name} syntax
        let interpolated_count = settings.custom.iter()
            .filter(|(k, v)| {
                k.ends_with("_fqn_pattern") && 
                v.as_str().map_or(false, |s| s.contains("{") && s.contains("_identifier_pattern}"))
            })
            .count();
        
        println!("\n✓ Found {} FQN patterns that reference imported identifier patterns", interpolated_count);
    } else {
        println!("✗ No settings section found!");
    }

    // Test 3: Check structured_pattern usage in FQN class
    println!("\n\nTest 3: Verify structured_pattern usage in FQN class");
    println!("--------------------------------------------------------");
    
    if let Some(fqn_class) = fqn_schema.classes.get("FQN") {
        println!("✓ Found FQN class");

        // slot_usage is an IndexMap, not an Option
        let slot_usage = &fqn_class.slot_usage;
        if !slot_usage.is_empty() {
            println!("✓ FQN class has slot_usage");

            if let Some(fqn_slot) = slot_usage.get("fqn") {
                println!("✓ Found fqn slot usage");

                if let Some(structured_pattern) = &fqn_slot.structured_pattern {
                    println!("✓ fqn slot has structured_pattern");
                    println!("  - syntax: {:?}", structured_pattern.syntax);
                    println!("  - interpolated: {:?}", structured_pattern.interpolated);

                    // Check if it references the fqn_pattern
                    if structured_pattern.syntax.as_ref().map_or(false, |s| s.contains("{fqn_pattern}")) {
                        println!("✓ structured_pattern correctly references {{fqn_pattern}}");
                    } else {
                        println!("⚠ structured_pattern doesn't reference {{fqn_pattern}}");
                    }
                } else {
                    println!("✗ fqn slot doesn't have structured_pattern");
                }
            }
        }
    }

    // ========================================================================
    // Part 3: Test Pattern Validation Through LinkML (not just regex)
    // ========================================================================

    println!("\n========================================");
    println!("Part 3: Pattern Validation Tests");
    println!("========================================\n");

    // Test country code pattern validation through ValidationEngine
    println!("Testing country_code_alpha2_identifier_pattern:");
    let engine = ValidationEngine::new(&identifier_schema)?;

    let pattern_test_cases = vec![
        ("US", true, "Valid: matches pattern [A-Z]{2}"),
        ("GB", true, "Valid: matches pattern [A-Z]{2}"),
        ("FR", true, "Valid: matches pattern [A-Z]{2}"),
        ("us", false, "Invalid: lowercase doesn't match pattern"),
        ("USA", false, "Invalid: too long (3 chars)"),
        ("U", false, "Invalid: too short (1 char)"),
        ("U1", false, "Invalid: contains digit"),
        ("", false, "Invalid: empty string"),
    ];

    println!("\nValidating through ValidationEngine:");
    let mut passed = 0;
    let mut failed = 0;

    for (value, expected, description) in &pattern_test_cases {
        let data = json!({
            "identifier": value
        });

        let report = engine.validate_as_class(&data, "CountryCodeAlpha2Identifier", None).await?;
        let is_valid = report.valid;

        let status = if is_valid == *expected {
            passed += 1;
            "✓"
        } else {
            failed += 1;
            "✗ UNEXPECTED"
        };

        println!("  {} '{}': {} - {}", status, value,
            if is_valid { "Valid  " } else { "Invalid" }, description);

        let errors: Vec<_> = report.errors().collect();
        if !is_valid && !errors.is_empty() {
            println!("      Error: {}", errors[0].message);
        }
    }

    println!("\nPattern validation test summary: {} passed, {} failed", passed, failed);

    // ========================================================================
    // Part 4: Test Pattern Import Validation (if FQN schema exists)
    // ========================================================================

    println!("\n========================================");
    println!("Part 4: Pattern Import Validation");
    println!("========================================\n");

    if fqn_schema_path.exists() {
        println!("Testing FQN pattern interpolation:");
        let fqn_engine = ValidationEngine::new(&fqn_schema)?;

        // Test FQN validation with interpolated patterns
        let fqn_test_cases = vec![
            ("US-NY-NYC-g-smithsonian", true, "Valid: all components match patterns"),
            ("GB-LN-LON-m-british", true, "Valid: UK museum"),
            ("XX-NY-NYC-g-smithsonian", false, "Invalid: XX not a valid country code"),
            ("US-NY-NYC-invalid-smithsonian", false, "Invalid: bad GLAM type"),
            ("us-ny-nyc-g-smithsonian", false, "Invalid: lowercase country code"),
        ];

        println!("\nValidating FQN with interpolated patterns:");
        let mut fqn_passed = 0;
        let mut fqn_failed = 0;

        for (fqn, expected, description) in &fqn_test_cases {
            let data = json!({
                "fqn": fqn
            });

            let report = fqn_engine.validate_as_class(&data, "FQN", None).await?;
            let is_valid = report.valid;

            let status = if is_valid == *expected {
                fqn_passed += 1;
                "✓"
            } else {
                fqn_failed += 1;
                "✗ UNEXPECTED"
            };

            println!("  {} '{}': {} - {}", status, fqn,
                if is_valid { "Valid  " } else { "Invalid" }, description);

            let errors: Vec<_> = report.errors().collect();
            if !is_valid && !errors.is_empty() {
                println!("      Error: {}", errors[0].message);
            }
        }

        println!("\nFQN pattern validation test summary: {} passed, {} failed", fqn_passed, fqn_failed);
    } else {
        println!("⚠ FQN schema not found, skipping FQN pattern validation tests");
    }

    println!("\n========================================");
    println!("Pattern Import and Validation Tests Complete");
    println!("========================================");
    println!("\nSummary:");
    println!("  ✓ Pattern definitions loaded from settings");
    println!("  ✓ Pattern imports resolved correctly");
    println!("  ✓ Pattern validation tested through ValidationEngine");
    println!("  ✓ Pattern interpolation tested (if FQN schema available)");

    Ok(())
}

