//! Test script to parse and convert all LinkML schema files
//!
//! This example tests:
//! 1. Parsing all schema files in crates/model/symbolic/schemata
//! 2. Generating Rust code from each schema
//! 3. Generating RDF/Turtle from each schema
//! 4. Generating TypeQL from each schema

use linkml_core::error::Result;
use linkml_service::generator::{Generator, RustGenerator, RdfGenerator, TypeQLGenerator};
use linkml_service::parser::factory::create_dev_schema_loader;
use std::path::PathBuf;
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> Result<()> {
    println!("========================================");
    println!("Testing All LinkML Schema Files");
    println!("========================================\n");

    // Get the workspace root
    // When running from the service crate, we need to go up to the schemata directory
    let workspace_root = std::env::current_dir()?;

    // Try multiple possible paths
    let possible_paths = vec![
        workspace_root.join("crates/model/symbolic/schemata"),  // From workspace root
        workspace_root.join("../../schemata"),                   // From service crate
        workspace_root.join("../../../schemata"),                // Alternative
    ];

    let schemata_dir = possible_paths.iter()
        .find(|p| p.exists())
        .ok_or_else(|| linkml_core::error::LinkMLError::io_error(
            format!("Could not find schemata directory. Tried: {:?}", possible_paths)
        ))?
        .clone();

    println!("Looking for schemas in: {}", schemata_dir.display());

    // Find all schema.yaml files (excluding data directories)
    let schema_files = find_schema_files(&schemata_dir)?;
    
    println!("Found {} schema files:\n", schema_files.len());
    for file in &schema_files {
        println!("  - {}", file.display());
    }
    println!();

    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut failed_files = Vec::new();

    // Create schema loader
    let loader = create_dev_schema_loader();

    // Create generators
    let rust_gen = RustGenerator::new();
    let rdf_gen = RdfGenerator::new();
    let typeql_gen = TypeQLGenerator::new();

    // Test each schema file
    for schema_file in &schema_files {
        total += 1;
        println!("----------------------------------------");
        println!("Testing: {}", schema_file.display());
        println!("----------------------------------------");

        // Test parsing
        print!("1. Testing parsing... ");
        let schema = match loader.load_file(schema_file).await {
            Ok(s) => {
                println!("✓");
                s
            }
            Err(e) => {
                println!("✗ Failed: {}", e);
                failed += 1;
                failed_files.push(format!("{} (parsing)", schema_file.display()));
                continue;
            }
        };

        // Test Rust generator
        print!("2. Testing Rust generator... ");
        match Generator::generate(&rust_gen, &schema) {
            Ok(_) => println!("✓"),
            Err(e) => {
                println!("✗ Failed: {}", e);
                failed += 1;
                failed_files.push(format!("{} (rust)", schema_file.display()));
                continue;
            }
        }

        // Test RDF generator
        print!("3. Testing RDF generator... ");
        match Generator::generate(&rdf_gen, &schema) {
            Ok(_) => println!("✓"),
            Err(e) => {
                println!("✗ Failed: {}", e);
                failed += 1;
                failed_files.push(format!("{} (rdf)", schema_file.display()));
                continue;
            }
        }

        // Test TypeQL generator
        print!("4. Testing TypeDB generator... ");
        match Generator::generate(&typeql_gen, &schema) {
            Ok(_) => println!("✓"),
            Err(e) => {
                println!("✗ Failed: {}", e);
                failed += 1;
                failed_files.push(format!("{} (typedb)", schema_file.display()));
                continue;
            }
        }

        passed += 1;
        println!("✓ All tests passed for {}\n", schema_file.display());
    }

    // Print summary
    println!("========================================");
    println!("Test Summary");
    println!("========================================");
    println!("Total schemas tested: {}", total);
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!();

    if failed > 0 {
        println!("Failed files:");
        for file in &failed_files {
            println!("  ✗ {}", file);
        }
        println!();
        std::process::exit(1);
    } else {
        println!("All tests passed! ✓\n");
    }

    Ok(())
}

/// Find all schema.yaml files in the given directory (excluding data directories)
fn find_schema_files(base_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut schema_files = Vec::new();

    if !base_dir.exists() {
        eprintln!("Warning: Directory does not exist: {}", base_dir.display());
        return Ok(schema_files);
    }

    for entry in WalkDir::new(base_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip data directories
        if path.components().any(|c| c.as_os_str() == "data") {
            continue;
        }

        // Only include schema.yaml files
        if path.file_name() == Some(std::ffi::OsStr::new("schema.yaml")) {
            schema_files.push(path.to_path_buf());
        }
    }

    schema_files.sort();
    Ok(schema_files)
}

