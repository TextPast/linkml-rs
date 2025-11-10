//! Unit tests for LinkML parser
//!
//! This module provides comprehensive testing for the Pest-based LinkML parser,
//! ensuring it correctly parses all LinkML constructs according to the specification.

use linkml_core::error::Result;
use linkml_core::parser::LinkMLParser;
use pretty_assertions::assert_eq;

/// Test parsing a minimal schema with only required fields
#[test]
fn test_parse_minimal_schema() -> Result<()> {
    let input = r#"
id: https://example.org/test
name: test_schema
"#;

    let schema = LinkMLParser::parse_schema(input)?;
    
    assert_eq!(schema.id.as_ref().unwrap().value, "https://example.org/test");
    assert_eq!(schema.name.as_ref().unwrap().value, "test_schema");
    
    Ok(())
}

/// Test parsing a schema with metadata fields
#[test]
fn test_parse_schema_with_metadata() -> Result<()> {
    let input = r#"
id: https://example.org/test
name: test_schema
title: Test Schema
description: A test schema for parser validation
version: 1.0.0
license: CC-BY-4.0
"#;

    let schema = LinkMLParser::parse_schema(input)?;
    
    assert_eq!(schema.id.as_ref().unwrap().value, "https://example.org/test");
    assert_eq!(schema.name.as_ref().unwrap().value, "test_schema");
    assert_eq!(schema.title.as_ref().unwrap().value, "Test Schema");
    assert_eq!(schema.version.as_ref().unwrap().value, "1.0.0");
    assert_eq!(schema.license.as_ref().unwrap().value, "CC-BY-4.0");
    
    Ok(())
}

/// Test parsing a schema with prefixes
#[test]
fn test_parse_schema_with_prefixes() -> Result<()> {
    let input = r#"
id: https://example.org/test
name: test_schema
prefixes:
  linkml: https://w3id.org/linkml/
  skos: http://www.w3.org/2004/02/skos/core#
  rdf: http://www.w3.org/1999/02/22-rdf-syntax-ns#
default_prefix: linkml
"#;

    let schema = LinkMLParser::parse_schema(input)?;
    
    assert_eq!(schema.prefixes.len(), 3);
    assert_eq!(schema.prefixes.get("linkml").unwrap().value, "https://w3id.org/linkml/");
    assert_eq!(schema.prefixes.get("skos").unwrap().value, "http://www.w3.org/2004/02/skos/core#");
    assert_eq!(schema.default_prefix.as_ref().unwrap().value, "linkml");
    
    Ok(())
}

/// Test parsing a schema with imports
#[test]
fn test_parse_schema_with_imports() -> Result<()> {
    let input = r#"
id: https://example.org/test
name: test_schema
imports:
  - linkml:types
  - txp:meta/identifier/identifier/schema
"#;

    let schema = LinkMLParser::parse_schema(input)?;
    
    assert_eq!(schema.imports.len(), 2);
    assert_eq!(schema.imports[0].value, "linkml:types");
    assert_eq!(schema.imports[1].value, "txp:meta/identifier/identifier/schema");
    
    Ok(())
}

/// Test parsing a schema with a simple class definition
#[test]
fn test_parse_schema_with_simple_class() -> Result<()> {
    let input = r#"
id: https://example.org/test
name: test_schema

classes:
  Person:
    description: A person
    slots:
      - name
      - email
"#;

    let schema = LinkMLParser::parse_schema(input)?;
    
    assert_eq!(schema.classes.len(), 1);
    let person_class = schema.classes.get("Person").unwrap();
    assert_eq!(person_class.value.name, "Person");
    
    Ok(())
}

/// Test parsing a schema with class inheritance
#[test]
fn test_parse_schema_with_class_inheritance() -> Result<()> {
    let input = r#"
id: https://example.org/test
name: test_schema

classes:
  NamedEntity:
    description: An entity with a name
    slots:
      - name
  
  Person:
    is_a: NamedEntity
    description: A person
    slots:
      - email
"#;

    let schema = LinkMLParser::parse_schema(input)?;
    
    assert_eq!(schema.classes.len(), 2);
    let person_class = schema.classes.get("Person").unwrap();
    assert_eq!(person_class.value.is_a.as_ref().unwrap().value, "NamedEntity");
    
    Ok(())
}

/// Test parsing a schema with slot definitions
#[test]
fn test_parse_schema_with_slots() -> Result<()> {
    let input = r#"
id: https://example.org/test
name: test_schema

slots:
  name:
    description: A name
    range: string
    required: true
  
  email:
    description: An email address
    range: string
    pattern: "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
"#;

    let schema = LinkMLParser::parse_schema(input)?;
    
    assert_eq!(schema.slots.len(), 2);
    assert!(schema.slots.contains_key("name"));
    assert!(schema.slots.contains_key("email"));
    
    Ok(())
}

/// Test parsing a schema with enum definitions
#[test]
fn test_parse_schema_with_enums() -> Result<()> {
    let input = r#"
id: https://example.org/test
name: test_schema

enums:
  StatusEnum:
    description: Status values
    permissible_values:
      active:
        description: Currently active
      inactive:
        description: No longer active
      pending:
        description: Awaiting activation
"#;

    let schema = LinkMLParser::parse_schema(input)?;
    
    assert_eq!(schema.enums.len(), 1);
    let status_enum = schema.enums.get("StatusEnum").unwrap();
    assert_eq!(status_enum.value.name, "StatusEnum");
    
    Ok(())
}

/// Test parsing a schema with type definitions
#[test]
fn test_parse_schema_with_types() -> Result<()> {
    let input = r#"
id: https://example.org/test
name: test_schema

types:
  EmailAddress:
    typeof: string
    description: An email address
    pattern: "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
"#;

    let schema = LinkMLParser::parse_schema(input)?;
    
    assert_eq!(schema.types.len(), 1);
    let email_type = schema.types.get("EmailAddress").unwrap();
    assert_eq!(email_type.value.name, "EmailAddress");
    
    Ok(())
}

/// Test parsing the actual LinkML meta schema
#[test]
fn test_parse_linkml_meta_schema() -> Result<()> {
    let input = include_str!("../../schemas/meta.yaml");
    
    let schema = LinkMLParser::parse_schema(input)?;
    
    assert_eq!(schema.id.as_ref().unwrap().value, "https://w3id.org/linkml/meta");
    assert_eq!(schema.name.as_ref().unwrap().value, "meta");
    assert!(schema.classes.len() > 0);
    
    Ok(())
}

/// Test error handling for invalid YAML
#[test]
fn test_parse_invalid_yaml() {
    let input = r#"
id: https://example.org/test
name: [invalid: yaml: structure
"#;

    let result = LinkMLParser::parse_schema(input);
    assert!(result.is_err());
}

/// Test error handling for missing required fields
#[test]
fn test_parse_missing_required_fields() {
    let input = r#"
title: Test Schema Without Required Fields
"#;

    let result = LinkMLParser::parse_schema(input);
    // Parser should still succeed, validation is separate
    assert!(result.is_ok());
}

/// Test parsing schema with inline and block descriptions
#[test]
fn test_parse_descriptions() -> Result<()> {
    let input = r#"
id: https://example.org/test
name: test_schema

classes:
  Person:
    description: A simple inline description
  
  Organization:
    description: |
      A multi-line block description
      that spans multiple lines
      and preserves formatting
"#;

    let schema = LinkMLParser::parse_schema(input)?;
    
    assert_eq!(schema.classes.len(), 2);
    
    Ok(())
}

/// Test parsing schema with annotations
#[test]
fn test_parse_annotations() -> Result<()> {
    let input = r#"
id: https://example.org/test
name: test_schema

classes:
  Person:
    description: A person
    annotations:
      owl: Thing
      rdfs:label: Person
"#;

    let schema = LinkMLParser::parse_schema(input)?;
    
    assert_eq!(schema.classes.len(), 1);
    
    Ok(())
}

/// Test span information is correctly captured
#[test]
fn test_span_information() -> Result<()> {
    let input = r#"
id: https://example.org/test
name: test_schema
"#;

    let schema = LinkMLParser::parse_schema(input)?;
    
    // Verify that span information is captured
    assert!(schema.id.is_some());
    let id_span = &schema.id.as_ref().unwrap().span;
    assert!(id_span.line > 0);
    assert!(id_span.column > 0);
    
    Ok(())
}

/// Test parsing performance with a large schema
#[test]
#[ignore] // Run with --ignored flag for performance testing
fn test_parse_performance_large_schema() -> Result<()> {
    use std::time::Instant;
    
    // Generate a large schema with 100 classes
    let mut schema_parts = vec![
        "id: https://example.org/test".to_string(),
        "name: large_schema".to_string(),
        "classes:".to_string(),
    ];
    
    for i in 0..100 {
        schema_parts.push(format!("  Class{}:", i));
        schema_parts.push(format!("    description: Class number {}", i));
        schema_parts.push("    slots:".to_string());
        schema_parts.push(format!("      - field{}", i));
    }
    
    let input = schema_parts.join("\n");
    
    let start = Instant::now();
    let schema = LinkMLParser::parse_schema(&input)?;
    let duration = start.elapsed();
    
    println!("Parsed {} classes in {:?}", schema.classes.len(), duration);
    
    // Target: <1ms for 100-line schema (this is ~400 lines)
    // We'll be lenient here, but should be <10ms
    assert!(duration.as_millis() < 10, "Parser took too long: {:?}", duration);
    
    Ok(())
}
