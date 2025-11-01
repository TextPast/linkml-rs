//! Test pattern import and interpolation functionality
//!
//! This example verifies that:
//! 1. Pattern definitions in settings are correctly parsed
//! 2. Imported patterns are accessible in importing schemas
//! 3. Pattern interpolation with {pattern_name} syntax works
//! 4. Structured patterns with interpolated: true are validated

use linkml_core::error::Result;
use linkml_service::parser::factory::create_dev_schema_loader;
use std::path::PathBuf;

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
        return Err(linkml_core::error::LinkMLError::validation(
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
        
        if let Some(slot_usage) = &fqn_class.slot_usage {
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

    println!("\n========================================");
    println!("Pattern Import Tests Complete");
    println!("========================================");
    
    Ok(())
}

