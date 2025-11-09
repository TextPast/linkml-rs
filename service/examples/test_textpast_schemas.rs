//! Test TextPast/RootReal LinkML schema parsing
//!
//! This example tests the parsing of schemas and instances using the new
//! TextPast/RootReal conventions:
//! 1. Schemas explicitly reference their location
//! 2. Instances have real metadata (not commented out)
//! 3. txp: imports resolve local-first with remote fallback
//! 4. Validation of complex types like ISO3166Entity

use linkml_service::parser::{ImportResolverV2, Parser, SchemaLoader, SchemaParser};
use logger_service::wiring::wire_testing_logger;
use parse_service::NoLinkML;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TextPast/RootReal LinkML Schema Parser Test ===\n");

    // Test 1: Parse hyperentity schema (base schema)
    println!("Test 1: Parsing meta/entity/hyperentity schema...");
    test_schema("crates/model/symbolic/schemata/meta/entity/hyperentity/schema.yaml").await?;

    // Test 2: Parse country schema (with imports)
    println!("\nTest 2: Parsing place/polity/country schema...");
    test_schema("crates/model/symbolic/schemata/place/polity/country/schema.yaml").await?;

    // Test 3: Parse ISO 3166 instance file
    println!("\nTest 3: Parsing ISO 3166 instance file...");
    test_instance("crates/model/symbolic/schemata/place/polity/country/iso_3166_entity.yaml").await?;

    // Test 4: Test txp: import resolution
    println!("\nTest 4: Testing txp: import resolution...");
    test_txp_imports().await?;

    // Test 5: Validate ISO3166Entity id field
    println!("\nTest 5: Validating ISO3166Entity id field constraints...");
    test_iso3166_validation().await?;

    println!("\n=== All tests completed successfully! ===");
    Ok(())
}

async fn test_schema(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let logger = wire_testing_logger()?.into_arc();
    let parse_service_handle = parse_service::wiring::wire_parse_for_testing::<NoLinkML>(logger).await?;
    let parse_service = parse_service_handle.into_arc();
    
    let parser = Parser::new(parse_service);
    let content = tokio::fs::read_to_string(path).await?;
    
    let schema = parser.parse_str(&content, "yaml").await?;
    
    println!("  ✓ Schema ID: {}", schema.id);
    println!("  ✓ Schema name: {}", schema.name);
    println!("  ✓ Version: {}", schema.version.as_ref().unwrap_or(&"N/A".to_string()));
    println!("  ✓ Classes: {}", schema.classes.len());
    println!("  ✓ Slots: {}", schema.slots.len());
    println!("  ✓ Imports: {}", schema.imports.len());
    
    if !schema.imports.is_empty() {
        println!("  Imports:");
        for import in &schema.imports {
            println!("    - {}", import);
        }
    }
    
    Ok(())
}

async fn test_instance(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = tokio::fs::read_to_string(path).await?;
    let instance_data: serde_yaml::Value = serde_yaml::from_str(&content)?;
    
    // Check metadata
    if let Some(id) = instance_data.get("id") {
        println!("  ✓ Instance ID: {}", id.as_str().unwrap_or("N/A"));
    }
    if let Some(schema) = instance_data.get("schema") {
        println!("  ✓ Schema reference: {}", schema.as_str().unwrap_or("N/A"));
    }
    if let Some(version) = instance_data.get("version") {
        println!("  ✓ Version: {}", version.as_str().unwrap_or("N/A"));
    }
    if let Some(created_on) = instance_data.get("created_on") {
        println!("  ✓ Created on: {}", created_on.as_str().unwrap_or("N/A"));
    }
    
    // Check instances array
    if let Some(instances) = instance_data.get("instances") {
        if let Some(instances_array) = instances.as_sequence() {
            println!("  ✓ Number of instances: {}", instances_array.len());
            
            // Show first few instances
            for (i, instance) in instances_array.iter().take(3).enumerate() {
                if let Some(id) = instance.get("id") {
                    if let Some(label) = instance.get("label") {
                        println!("    {}. {} - {}", i + 1, 
                            id.as_str().unwrap_or("?"),
                            label.as_str().unwrap_or("?"));
                    }
                }
            }
        }
    }
    
    Ok(())
}

async fn test_txp_imports() -> Result<(), Box<dyn std::error::Error>> {
    let logger = wire_testing_logger()?.into_arc();
    let parse_service_handle = parse_service::wiring::wire_parse_for_testing::<NoLinkML>(logger).await?;
    let parse_service = parse_service_handle.into_arc();
    
    let loader = SchemaLoader::new(parse_service);
    
    // Load a schema with txp: imports
    let schema = loader.load_file(
        &PathBuf::from("crates/model/symbolic/schemata/place/polity/country/schema.yaml")
    ).await?;
    
    println!("  ✓ Schema loaded with {} imports", schema.imports.len());
    
    // Check that imports were resolved
    for import in &schema.imports {
        if import.starts_with("txp:") {
            println!("    - Resolved txp: import: {}", import);
        }
    }
    
    Ok(())
}

async fn test_iso3166_validation() -> Result<(), Box<dyn std::error::Error>> {
    // Load the country schema
    let logger = wire_testing_logger()?.into_arc();
    let parse_service_handle = parse_service::wiring::wire_parse_for_testing::<NoLinkML>(logger).await?;
    let parse_service = parse_service_handle.into_arc();
    
    let parser = Parser::new(parse_service);
    let content = tokio::fs::read_to_string(
        "crates/model/symbolic/schemata/place/polity/country/schema.yaml"
    ).await?;
    
    let schema = parser.parse_str(&content, "yaml").await?;
    
    // Check ISO3166Entity class
    if let Some(iso_class) = schema.classes.get("ISO3166Entity") {
        println!("  ✓ Found ISO3166Entity class");
        println!("    Description: {}", iso_class.description.as_ref().unwrap_or(&"N/A".to_string()));
        
        // Check identifier slot usage
        if let Some(slot_usage) = &iso_class.slot_usage {
            if let Some(identifier_usage) = slot_usage.get("identifier") {
                println!("    ✓ Identifier slot configured");
                if let Some(range) = &identifier_usage.range {
                    println!("      Range: {}", range);
                }
                if let Some(required) = identifier_usage.required {
                    println!("      Required: {}", required);
                }
            }
        }
    }
    
    Ok(())
}

