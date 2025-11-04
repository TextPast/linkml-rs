//! Load YAML instance data into TypeDB for testing
//!
//! This tool loads existing YAML instance files into TypeDB so we can test
//! the export pipeline (TypeDB → RDF/OWL/Turtle/JSON/YAML).
//!
//! Usage:
//!   cargo run --example load_yaml_to_typedb -- \
//!     --config batch_export_config.yaml \
//!     --database rootreal_test

use clap::Parser;
use linkml_core::types::SchemaDefinition;
use linkml_service::parser::YamlParser;
use linkml_service::loader::DataInstance;
use linkml_service::typedb_helper::{TypeDBHelper, instance_to_typeql};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "load_yaml_to_typedb")]
#[command(about = "Load YAML instance data into TypeDB", long_about = None)]
struct Args {
    /// Path to batch export configuration file
    #[arg(short, long, default_value = "batch_export_config.yaml")]
    config: PathBuf,

    /// TypeDB database name (will be created if doesn't exist)
    #[arg(short, long, default_value = "rootreal_test")]
    database: String,

    /// TypeDB server address
    #[arg(long, default_value = "localhost")]
    server: String,

    /// TypeDB server port
    #[arg(long, default_value = "1729")]
    port: u16,

    /// Dry run (don't actually load into TypeDB)
    #[arg(long)]
    dry_run: bool,

    /// Only load specific classes (comma-separated)
    #[arg(long)]
    only: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BatchExportConfig {
    database: String,
    output_base_dir: PathBuf,
    classes: Vec<ClassMapping>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ClassMapping {
    name: String,
    typedb_type: String,
    linkml_class: String,
    schema: PathBuf,
    output: PathBuf,
    enabled: bool,
}

#[derive(Debug)]
struct LoadResult {
    class_name: String,
    success: bool,
    instance_count: usize,
    duration_secs: f64,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    println!("=== YAML to TypeDB Loader ===\n");

    // Load configuration
    println!("Loading configuration from: {}", args.config.display());
    let config_content = fs::read_to_string(&args.config)?;
    let config: BatchExportConfig = serde_yaml::from_str(&config_content)?;
    println!("  ✓ Configuration loaded");
    println!("  Target database: {}", args.database);
    println!("  TypeDB server: {}:{}", args.server, args.port);
    println!();

    // Filter classes
    let classes_to_load = filter_classes(&config.classes, &args)?;
    println!("Classes to load: {}", classes_to_load.len());
    for class in &classes_to_load {
        println!("  - {} ({})", class.name, class.linkml_class);
    }
    println!();

    if args.dry_run {
        println!("DRY RUN MODE - Analyzing YAML files without loading to TypeDB\n");
    }

    // Check TypeDB connection
    if !args.dry_run {
        println!("Checking TypeDB connection...");
        println!("  Note: This requires TypeDB server running at {}:{}", args.server, args.port);
        println!("  Note: This example shows the structure - integrate with DBMS service for production");
        println!();
    }

    // Load each class
    let start_time = Instant::now();
    let mut results = Vec::new();

    for (idx, class) in classes_to_load.iter().enumerate() {
        let result = load_class_instances(
            &config,
            class,
            idx + 1,
            classes_to_load.len(),
            args.dry_run,
            &args.server,
            args.port,
            &args.database,
        ).await;
        results.push(result);
    }

    let total_duration = start_time.elapsed();

    // Print summary
    print_summary(&results, total_duration, args.dry_run);

    Ok(())
}

fn filter_classes(
    classes: &[ClassMapping],
    args: &Args,
) -> std::result::Result<Vec<ClassMapping>, Box<dyn std::error::Error>> {
    let mut filtered: Vec<ClassMapping> = classes.iter()
        .filter(|c| c.enabled)
        .cloned()
        .collect();

    if let Some(only) = &args.only {
        let only_set: std::collections::HashSet<String> = only
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        filtered.retain(|c| only_set.contains(&c.linkml_class) || only_set.contains(&c.name));
    }

    Ok(filtered)
}

async fn load_class_instances(
    config: &BatchExportConfig,
    class: &ClassMapping,
    idx: usize,
    total: usize,
    dry_run: bool,
    server: &str,
    port: u16,
    database: &str,
) -> LoadResult {
    let start_time = Instant::now();
    
    println!("[{}/{}] Loading: {} ({})", idx, total, class.name, class.linkml_class);

    // Find instance YAML file
    let instance_file = find_instance_file(&config.output_base_dir, &class.output);
    
    if instance_file.is_none() {
        println!("  ⚠ No instance file found");
        return LoadResult {
            class_name: class.name.clone(),
            success: false,
            instance_count: 0,
            duration_secs: start_time.elapsed().as_secs_f64(),
            error: Some("No instance file found".to_string()),
        };
    }

    let instance_path = instance_file.unwrap();
    println!("  Instance file: {}", instance_path.display());

    // Load schema
    let schema_path = config.output_base_dir.join(&class.schema);
    let schema = match load_schema(&schema_path).await {
        Ok(s) => s,
        Err(e) => {
            return LoadResult {
                class_name: class.name.clone(),
                success: false,
                instance_count: 0,
                duration_secs: start_time.elapsed().as_secs_f64(),
                error: Some(format!("Failed to load schema: {}", e)),
            };
        }
    };

    println!("  Schema: {}", schema.name);

    // Load instances from YAML
    let instances = match load_instances(&instance_path, &schema).await {
        Ok(i) => i,
        Err(e) => {
            return LoadResult {
                class_name: class.name.clone(),
                success: false,
                instance_count: 0,
                duration_secs: start_time.elapsed().as_secs_f64(),
                error: Some(format!("Failed to load instances: {}", e)),
            };
        }
    };

    println!("  Loaded {} instances from YAML", instances.len());

    let mut inserted_count = instances.len();

    if !dry_run {
        println!("  Loading into TypeDB...");

        // Connect to TypeDB
        let typedb_address = format!("{}:{}", server, port);
        let typedb = match TypeDBHelper::connect(&typedb_address).await {
            Ok(t) => t,
            Err(e) => {
                return LoadResult {
                    class_name: class.name.clone(),
                    success: false,
                    instance_count: 0,
                    duration_secs: start_time.elapsed().as_secs_f64(),
                    error: Some(format!("Failed to connect to TypeDB: {}", e)),
                };
            }
        };

        // Ensure database exists
        if let Err(e) = typedb.ensure_database(database).await {
            return LoadResult {
                class_name: class.name.clone(),
                success: false,
                instance_count: 0,
                duration_secs: start_time.elapsed().as_secs_f64(),
                error: Some(format!("Failed to create database: {}", e)),
            };
        }

        // Insert instances
        let mut inserted = 0;
        for (idx, instance) in instances.iter().enumerate() {
            let typeql = match instance_to_typeql(instance) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("  ⚠ Failed to convert instance to TypeQL: {}", e);
                    continue;
                }
            };

            match typedb.insert_instance(database, &typeql).await {
                Ok(_) => inserted += 1,
                Err(e) => {
                    eprintln!("  ⚠ Failed to insert instance: {}", e);
                    // Continue with next instance
                }
            }

            // Progress indicator every 100 instances
            if (idx + 1) % 100 == 0 {
                print!(".");
                let _ = std::io::Write::flush(&mut std::io::stdout());
            }
        }

        println!();
        println!("  ✓ Loaded {}/{} instances into TypeDB", inserted, instances.len());
        inserted_count = inserted;
    }

    println!("  ✓ Completed in {:.2}s", start_time.elapsed().as_secs_f64());

    LoadResult {
        class_name: class.name.clone(),
        success: true,
        instance_count: inserted_count,
        duration_secs: start_time.elapsed().as_secs_f64(),
        error: None,
    }
}

fn find_instance_file(base_dir: &PathBuf, output_dir: &PathBuf) -> Option<PathBuf> {
    let search_dir = base_dir.join(output_dir).parent()?.to_path_buf();

    // Look for YAML files that are not schema.yaml
    if let Ok(entries) = fs::read_dir(&search_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("yaml")
                && path.file_name().and_then(|s| s.to_str()) != Some("schema.yaml") {
                return Some(path);
            }
        }
    }

    None
}

async fn load_schema(
    schema_path: &PathBuf,
) -> std::result::Result<SchemaDefinition, Box<dyn std::error::Error>> {
    let schema_content = fs::read_to_string(schema_path)?;
    let parser = YamlParser::new();
    let schema = parser.parse(&schema_content)?;
    Ok(schema)
}

async fn load_instances(
    instance_path: &PathBuf,
    schema: &SchemaDefinition,
) -> std::result::Result<Vec<DataInstance>, Box<dyn std::error::Error>> {
    let yaml_content = fs::read_to_string(instance_path)?;

    // Parse YAML to extract instances array
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_content)?;

    // Extract class name from header
    let class_name = yaml_value
        .get("class")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Extract instances array
    let instances_array = yaml_value
        .get("instances")
        .and_then(|v| v.as_sequence())
        .ok_or("Instance file missing 'instances' array")?;

    // Convert to DataInstance objects
    let instances = parse_instances_from_yaml(instances_array, class_name, schema)?;

    Ok(instances)
}

fn parse_instances_from_yaml(
    instances_array: &[serde_yaml::Value],
    class_name: Option<String>,
    schema: &SchemaDefinition,
) -> std::result::Result<Vec<DataInstance>, Box<dyn std::error::Error>> {
    let mut instances = Vec::new();

    let target_class = if let Some(class) = class_name {
        class
    } else {
        schema.classes.keys().next()
            .ok_or("No class found in schema")?
            .clone()
    };

    for yaml_instance in instances_array {
        if let serde_yaml::Value::Mapping(_map) = yaml_instance {
            let json_str = serde_json::to_string(&yaml_instance)?;
            let json_value: serde_json::Value = serde_json::from_str(&json_str)?;

            if let serde_json::Value::Object(obj) = json_value {
                let id = obj.get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let instance = DataInstance {
                    class_name: target_class.clone(),
                    id,
                    data: obj.into_iter().collect(),
                    metadata: std::collections::HashMap::new(),
                };

                instances.push(instance);
            }
        }
    }

    Ok(instances)
}

fn print_summary(results: &[LoadResult], total_duration: std::time::Duration, dry_run: bool) {
    println!("\n=== Load Summary ===\n");

    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();
    let total_instances: usize = results.iter().map(|r| r.instance_count).sum();

    println!("Total classes: {}", results.len());
    println!("Successful: {}", successful);
    println!("Failed: {}", failed);
    println!("Total instances: {}", total_instances);
    println!("Total duration: {:.2}s", total_duration.as_secs_f64());

    if dry_run {
        println!("\nDRY RUN - No data was loaded into TypeDB");
    }

    if failed > 0 {
        println!("\nFailed loads:");
        for result in results.iter().filter(|r| !r.success) {
            println!("  - {}: {}", result.class_name, result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
        }
    }

    println!("\nSuccessful loads:");
    for result in results.iter().filter(|r| r.success) {
        println!("  ✓ {} - {} instances in {:.2}s",
                 result.class_name, result.instance_count, result.duration_secs);
    }

    if !dry_run {
        println!("\nNext steps:");
        println!("  1. Verify data in TypeDB: typedb console");
        println!("  2. Run export tool: cargo run --example batch_export_from_typedb");
        println!("  3. Compare exported artifacts with Phase 1 outputs");
    } else {
        println!("\nTo load into TypeDB:");
        println!("  1. Start TypeDB: typedb server");
        println!("  2. Run without --dry-run flag");
        println!("  3. Integrate with TypeDBIntegrationDumper for production");
    }
}

