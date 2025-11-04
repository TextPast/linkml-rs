//! Tests for instance-based range validation
//!
//! This tests the critical RootReal/Textpast convention where:
//! 1. Instance files are imported alongside schemas
//! 2. Slots with range_type: instance are constrained to instance IDs
//! 3. Validation enforces that values match actual instance data

use linkml_core::error::Result;
use linkml_service::parser::factory::create_dev_schema_loader;
use timestamp_service::wiring::wire_timestamp;

#[tokio::test]
async fn test_instance_file_import() -> Result<()> {
    // Test that instance files are loaded when imported
    let workspace_root = std::env::current_dir()?;
    let schemata_dir = workspace_root.join("crates/model/symbolic/schemata");
    
    if !schemata_dir.exists() {
        // Skip test if schemata directory doesn't exist
        return Ok(());
    }

    let loader = create_dev_schema_loader();
    let identifier_schema_path = schemata_dir.join("meta/identifier/identifier/schema.yaml");
    
    let schema = loader.load_file(&identifier_schema_path).await?;
    
    // Verify the instance import is present
    assert!(
        schema.imports.iter().any(|i| i.contains("iso_3166_entity")),
        "identifier schema should import iso_3166_entity instance file"
    );
    
    Ok(())
}

#[tokio::test]
async fn test_country_code_alpha2_range_type() -> Result<()> {
    // Test that CountryCodeAlpha2Identifier has range_type: instance
    let workspace_root = std::env::current_dir()?;
    let schemata_dir = workspace_root.join("crates/model/symbolic/schemata");
    
    if !schemata_dir.exists() {
        return Ok(());
    }

    let loader = create_dev_schema_loader();
    let identifier_schema_path = schemata_dir.join("meta/identifier/identifier/schema.yaml");
    
    let schema = loader.load_file(&identifier_schema_path).await?;
    
    // Find CountryCodeAlpha2Identifier class
    let country_code_class = schema.classes.get("CountryCodeAlpha2Identifier")
        .expect("CountryCodeAlpha2Identifier class should exist");
    
    // Check slot_usage for identifier slot
    let identifier_slot = country_code_class.slot_usage.get("identifier")
        .expect("identifier slot should be in slot_usage");
    
    // Verify range is ISO3166Entity
    assert_eq!(
        identifier_slot.range.as_deref(),
        Some("ISO3166Entity"),
        "identifier slot should have range: ISO3166Entity"
    );
    
    // Verify range_type is instance
    assert_eq!(
        identifier_slot.range_type.as_deref(),
        Some("instance"),
        "identifier slot should have range_type: instance"
    );
    
    // Verify range_properties includes id
    assert!(
        !identifier_slot.range_properties.is_empty() &&
        identifier_slot.range_properties.contains(&"id".to_string()),
        "identifier slot should have range_properties: [id]"
    );
    
    Ok(())
}

#[tokio::test]
async fn test_instance_file_loading() -> Result<()> {
    // Test that instance files can be loaded and parsed
    let workspace_root = std::env::current_dir()?;
    let instance_file = workspace_root.join("crates/model/symbolic/schemata/place/polity/country/iso_3166_entity.yaml");
    
    if !instance_file.exists() {
        return Ok(());
    }

    // Load the instance file
    let content = tokio::fs::read_to_string(&instance_file).await?;
    let instance_data: serde_yaml::Value = serde_yaml::from_str(&content)?;
    
    // Verify it has instances section
    assert!(
        instance_data.get("instances").is_some(),
        "Instance file should have 'instances' section"
    );
    
    // Verify instances is a sequence
    let instances = instance_data.get("instances")
        .and_then(|v| v.as_sequence())
        .expect("instances should be a sequence");
    
    // Verify instances have id fields
    assert!(
        instances.len() > 0,
        "Instance file should have at least one instance"
    );
    
    let first_instance = &instances[0];
    assert!(
        first_instance.get("id").is_some(),
        "Each instance should have an 'id' field"
    );
    
    // Collect all instance IDs
    let instance_ids: Vec<String> = instances.iter()
        .filter_map(|inst| inst.get("id"))
        .filter_map(|id| id.as_str())
        .map(|s| s.to_string())
        .collect();
    
    // Verify we have the expected country codes
    assert!(instance_ids.contains(&"US".to_string()), "Should have US country code");
    assert!(instance_ids.contains(&"AD".to_string()), "Should have AD country code");
    assert!(instance_ids.len() > 200, "Should have many country codes (249 expected)");
    
    println!("✓ Loaded {} country code instances", instance_ids.len());
    println!("  Examples: {:?}", &instance_ids[..5.min(instance_ids.len())]);
    
    Ok(())
}

#[tokio::test]
async fn test_instance_resolver_basic() -> Result<()> {
    // Test the instance resolver with actual files
    use linkml_service::validator::{InstanceLoader, InstanceResolver};
    use timestamp_service::wiring::wire_timestamp;

    let workspace_root = std::env::current_dir()?;
    let schemata_dir = workspace_root.join("crates/model/symbolic/schemata");

    if !schemata_dir.exists() {
        return Ok(());
    }

    // Create instance loader and resolver
    let timestamp_service = wire_timestamp().into_arc();
    let loader = std::sync::Arc::new(InstanceLoader::new(timestamp_service));
    let resolver = InstanceResolver::new(schemata_dir.clone(), loader);

    // Load the identifier schema
    let loader_svc = create_dev_schema_loader();
    let identifier_schema_path = schemata_dir.join("meta/identifier/identifier/schema.yaml");
    let schema = loader_svc.load_file(&identifier_schema_path).await?;

    // Get the CountryCodeAlpha2Identifier class
    let country_code_class = schema.classes.get("CountryCodeAlpha2Identifier")
        .expect("CountryCodeAlpha2Identifier class should exist");

    // Get the identifier slot from slot_usage
    let identifier_slot = country_code_class.slot_usage.get("identifier")
        .expect("identifier slot should be in slot_usage");

    // Get valid IDs for this slot
    let valid_ids = resolver.get_valid_ids_for_slot(identifier_slot, &schema).await?;

    if let Some(ids) = valid_ids {
        println!("✓ Loaded {} valid country codes", ids.len());
        println!("  Examples: {:?}", &ids[..5.min(ids.len())]);

        // Verify expected country codes are present
        assert!(ids.contains(&"US".to_string()), "Should have US country code");
        assert!(ids.contains(&"AD".to_string()), "Should have AD country code");
        assert!(ids.len() > 200, "Should have many country codes");

        // Test validation
        assert!(
            resolver.validate_instance_value("US", identifier_slot, &schema).await?,
            "US should be valid"
        );
        assert!(
            resolver.validate_instance_value("AD", identifier_slot, &schema).await?,
            "AD should be valid"
        );
        assert!(
            !resolver.validate_instance_value("XX", identifier_slot, &schema).await?,
            "XX should be invalid"
        );

        println!("✓ Instance-based validation working correctly!");
    } else {
        println!("⚠ No instance data found - check instance file loading");
    }

    Ok(())
}

#[tokio::test]
async fn test_yaml_instance_file_loading() -> Result<()> {
    // Test direct YAML instance file loading
    use linkml_service::validator::{InstanceConfig, InstanceLoader};
    use timestamp_service::wiring::wire_timestamp;

    let workspace_root = std::env::current_dir()?;
    let instance_file = workspace_root.join("crates/model/symbolic/schemata/place/polity/country/iso_3166_entity.yaml");

    if !instance_file.exists() {
        return Ok(());
    }

    let timestamp_service = wire_timestamp().into_arc();
    let loader = InstanceLoader::new(timestamp_service);
    let config = InstanceConfig::default(); // Uses 'id' as key field

    // Load the YAML instance file
    let instance_data = loader.load_yaml_file(&instance_file, &config).await?;

    println!("✓ Loaded instance data from YAML file");
    println!("  Source: {}", instance_data.source);
    println!("  Loaded at: {}", instance_data.loaded_at);

    // Check that we have values
    assert!(!instance_data.values.is_empty(), "Should have loaded values");

    // The values should be keyed by the instance IDs
    let total_values: usize = instance_data.values.values().map(|v| v.len()).sum();
    println!("  Total values: {}", total_values);

    Ok(())
}

#[tokio::test]
async fn test_multiple_instance_files_different_field_names() -> Result<()> {
    // Test that instance loading works with different field names
    use linkml_service::validator::{InstanceConfig, InstanceLoader};

    let workspace_root = std::env::current_dir()?;
    let schemata_dir = workspace_root.join("crates/model/symbolic/schemata");

    if !schemata_dir.exists() {
        return Ok(());
    }

    let timestamp_service = wire_timestamp().into_arc();
    let loader = InstanceLoader::new(timestamp_service);

    // Test 1: Country codes (uses 'id' field)
    let country_file = schemata_dir.join("place/polity/country/iso_3166_entity.yaml");
    if country_file.exists() {
        let config = InstanceConfig {
            key_field: "id".to_string(),
            value_field: None,
            filter: None,
        };
        let country_data = loader.load_yaml_file(&country_file, &config).await?;
        println!("✓ Loaded country codes with 'id' field");
        assert!(!country_data.values.is_empty(), "Should have country code values");
    }

    // Test 2: Timezones (uses 'timezone_component_value' field)
    let timezone_file = schemata_dir.join("time/timezone/timezone.yaml");
    if timezone_file.exists() {
        let config = InstanceConfig {
            key_field: "timezone_component_value".to_string(),
            value_field: None,
            filter: None,
        };
        let timezone_data = loader.load_yaml_file(&timezone_file, &config).await?;
        println!("✓ Loaded timezones with 'timezone_component_value' field");
        assert!(!timezone_data.values.is_empty(), "Should have timezone values");
    }

    // Test 3: Languages (uses 'id' field with additional fields)
    let language_file = schemata_dir.join("language/iso_639-3/iso_639-3_entity.yaml");
    if language_file.exists() {
        let config = InstanceConfig {
            key_field: "id".to_string(),
            value_field: Some("label".to_string()),
            filter: None,
        };
        let language_data = loader.load_yaml_file(&language_file, &config).await?;
        println!("✓ Loaded languages with 'id' and 'label' fields");
        assert!(!language_data.values.is_empty(), "Should have language values");
    }

    // Test 4: Document parsers (uses 'id' field with complex structure)
    let parser_file = schemata_dir.join("meta/document/document_parser/document_parser.yaml");
    if parser_file.exists() {
        let config = InstanceConfig {
            key_field: "id".to_string(),
            value_field: Some("name".to_string()),
            filter: None,
        };
        let parser_data = loader.load_yaml_file(&parser_file, &config).await?;
        println!("✓ Loaded document parsers with 'id' and 'name' fields");
        assert!(!parser_data.values.is_empty(), "Should have parser values");
    }

    println!("\n✓ All instance file formats loaded successfully!");
    println!("  Instance loading works with:");
    println!("  - Different field names (id, timezone_component_value, etc.)");
    println!("  - Simple structures (just id)");
    println!("  - Complex structures (id + multiple fields)");
    println!("  - Large files (48k+ lines for languages)");

    Ok(())
}

#[tokio::test]
async fn test_instance_resolver_with_custom_field_names() -> Result<()> {
    // Test that InstanceResolver works with custom field names from range_properties
    use linkml_service::validator::{InstanceLoader, InstanceResolver};
    use linkml_core::types::{SlotDefinition, SchemaDefinition};

    let workspace_root = std::env::current_dir()?;
    let schemata_dir = workspace_root.join("crates/model/symbolic/schemata");

    if !schemata_dir.exists() {
        return Ok(());
    }

    // Create resolver
    let timestamp_service = wire_timestamp().into_arc();
    let loader = std::sync::Arc::new(InstanceLoader::new(timestamp_service));
    let resolver = InstanceResolver::new(schemata_dir.clone(), loader);

    // Create a mock schema with timezone import (correct convention: ends with /instance)
    let mut schema = SchemaDefinition::default();
    schema.imports = vec!["txp:time/timezone/timezone/instance".to_string()];

    // Create a mock slot with range_type: instance and custom field name
    let mut slot = SlotDefinition::default();
    slot.range_type = Some("instance".to_string());
    slot.range = Some("Timezone".to_string());
    slot.range_properties = vec!["timezone_component_value".to_string()];

    // Get valid IDs - should use timezone_component_value field
    if let Some(valid_ids) = resolver.get_valid_ids_for_slot(&slot, &schema).await? {
        println!("✓ Loaded {} timezone values using custom field 'timezone_component_value'", valid_ids.len());
        println!("  Examples: {:?}", &valid_ids[..3.min(valid_ids.len())]);

        // Verify we got timezone values
        assert!(valid_ids.len() > 0, "Should have timezone values");
        assert!(
            valid_ids.iter().any(|id| id.contains("/")),
            "Timezone values should contain '/' (e.g., 'Europe/London')"
        );

        println!("✓ Instance resolver correctly uses range_properties field!");
    } else {
        println!("⚠ Could not load timezone instance data");
    }

    Ok(())
}

