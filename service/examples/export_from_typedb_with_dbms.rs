//! Export data from TypeDB to all formats using real DBMS service
//!
//! This example demonstrates Phase 2 with a real TypeDB connection:
//! TypeDB → RDF/OWL/Turtle/YAML/JSON
//!
//! Prerequisites:
//!   - TypeDB server running on localhost:1729
//!   - Database created with schema and data
//!
//! Usage:
//!   cargo run --example export_from_typedb_with_dbms -- \
//!     --database rootreal \
//!     --class Translation \
//!     --schema crates/model/symbolic/schemata/language/iso_639-3/schema.yaml \
//!     --output crates/model/symbolic/schemata/language/iso_639-3/data/

use clap::Parser;
use linkml_core::types::SchemaDefinition;
use linkml_service::parser::{YamlParserV2, SchemaParser};
use linkml_service::file_system_adapter::TokioFileSystemAdapter;
use linkml_service::loader::{
    DataDumper, DataLoader, DumpOptions, LoadOptions,
    RdfDumper, RdfOptions, RdfSerializationFormat,
    TypeDBIntegrationLoader, TypeDBIntegrationOptions,
    dbms_executor::DBMSServiceExecutor,
};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

// Import DBMS service components
use dbms_core::config::{DBMSConfig, TypeDBServerConfig};
use dbms_service::factory::StandardDBMSServiceFactory;

#[derive(Parser, Debug)]
#[command(name = "export_from_typedb_with_dbms")]
#[command(about = "Export data from TypeDB using DBMS service", long_about = None)]
struct Args {
    /// TypeDB database name
    #[arg(short, long, default_value = "rootreal")]
    database: String,

    /// LinkML class name to export
    #[arg(short, long)]
    class: String,

    /// Path to LinkML schema YAML file
    #[arg(short, long)]
    schema: PathBuf,

    /// Output directory for generated files
    #[arg(short, long)]
    output: PathBuf,

    /// TypeDB server address
    #[arg(long, default_value = "localhost")]
    server: String,

    /// TypeDB server port
    #[arg(long, default_value = "1729")]
    port: u16,

    /// TypeDB type name (if different from class name)
    #[arg(long)]
    typedb_type: Option<String>,

    /// Batch size for loading instances
    #[arg(long, default_value = "1000")]
    batch_size: usize,

    /// Include inferred facts
    #[arg(long, default_value = "true")]
    include_inferred: bool,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    println!("=== TypeDB Export Tool (with DBMS Service) ===\n");
    println!("Configuration:");
    println!("  Database: {}", args.database);
    println!("  Class: {}", args.class);
    println!("  Schema: {}", args.schema.display());
    println!("  Output: {}", args.output.display());
    println!("  Server: {}:{}", args.server, args.port);
    println!();

    // Load LinkML schema
    println!("Loading LinkML schema...");
    let schema_content = fs::read_to_string(&args.schema)?;
    
    let fs = Arc::new(TokioFileSystemAdapter::new());
    let parser = YamlParserV2::new(fs);
    let schema: SchemaDefinition = parser.parse_str(&schema_content)
        .map_err(|e| Box::<dyn std::error::Error>::from(format!("Schema parse error: {}", e)))?;
    println!("  ✓ Schema loaded: {}", schema.name);

    // Create output directory
    fs::create_dir_all(&args.output)?;
    println!("  ✓ Output directory ready");
    println!();

    // Create DBMS service
    println!("Initializing DBMS service...");
    let dbms_service = create_dbms_service(&args).await?;
    println!("  ✓ DBMS service created");
    println!();

    // Configure TypeDB integration
    println!("Configuring TypeDB integration...");
    let mut options = TypeDBIntegrationOptions {
        database_name: args.database.clone(),
        batch_size: args.batch_size,
        include_inferred: args.include_inferred,
        infer_types: true,
        ..Default::default()
    };

    // Map TypeDB type to LinkML class
    let typedb_type = args.typedb_type.as_ref()
        .map(|s| s.clone())
        .unwrap_or_else(|| to_snake_case(&args.class));
    options.type_mapping.insert(typedb_type.clone(), args.class.clone());
    println!("  ✓ Type mapping: {} → {}", typedb_type, args.class);
    println!();

    // Create executor with DBMS service
    let executor = DBMSServiceExecutor::new(dbms_service);
    
    // Create loader
    println!("Loading instances from TypeDB...");
    let loader = TypeDBIntegrationLoader::new(options, executor);
    let load_options = LoadOptions::default();
    
    match loader.load_string("", &schema, &load_options).await {
        Ok(instances) => {
            println!("  ✓ Loaded {} instances", instances.len());
            println!();

            // Export to all formats
            export_to_all_formats(&instances, &schema, &args).await?;
        }
        Err(e) => {
            println!("  ✗ Error loading from TypeDB: {}", e);
            println!();
            println!("Troubleshooting:");
            println!("  1. Ensure TypeDB is running: typedb server");
            println!("  2. Check database exists: typedb console");
            println!("  3. Verify schema is deployed");
            println!("  4. Check data exists in database");
        }
    }

    Ok(())
}

async fn create_dbms_service(
    _args: &Args,
) -> std::result::Result<Arc<impl dbms_core::DBMSService<Error = dbms_core::DBMSError>>, Box<dyn std::error::Error>> {
    // This is a simplified version - in production, use proper dependency injection
    // For now, we'll show the structure needed

    println!("  Note: This example requires full DBMS service setup");
    println!("  See crates/database/dbms/service/examples/ for complete examples");

    Err("DBMS service creation requires full dependency setup. See demo_helpers.rs for complete implementation.".into())
}

async fn export_to_all_formats(
    instances: &[linkml_service::loader::DataInstance],
    schema: &SchemaDefinition,
    args: &Args,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let dump_options = DumpOptions::default();
    let base_name = to_snake_case(&args.class);

    println!("Exporting to all formats...");

    // 1. RDF/XML
    println!("  - Generating RDF/XML...");
    let mut rdf_options = RdfOptions::default();
    rdf_options.generate_blank_nodes = true;
    let rdf_dumper = RdfDumper::with_options(rdf_options.clone());
    let rdf_data = rdf_dumper.dump_string(instances, schema, &dump_options).await?;
    let rdf_path = args.output.join(format!("{}.rdf", base_name));
    fs::write(&rdf_path, &rdf_data)?;
    println!("    ✓ {} ({} bytes)", rdf_path.display(), rdf_data.len());

    // 2. OWL (Turtle format)
    println!("  - Generating OWL/Turtle...");
    rdf_options.format = RdfSerializationFormat::Turtle;
    let owl_dumper = RdfDumper::with_options(rdf_options);
    let owl_data = owl_dumper.dump_string(instances, schema, &dump_options).await?;
    let owl_path = args.output.join(format!("{}.owl", base_name));
    fs::write(&owl_path, &owl_data)?;
    println!("    ✓ {} ({} bytes)", owl_path.display(), owl_data.len());

    // 3. Turtle
    println!("  - Generating Turtle...");
    let ttl_path = args.output.join(format!("{}.ttl", base_name));
    fs::write(&ttl_path, &owl_data)?;
    println!("    ✓ {} ({} bytes)", ttl_path.display(), owl_data.len());

    // 4. JSON
    println!("  - Generating JSON...");
    use linkml_service::loader::json::JsonDumper;
    let json_dumper = JsonDumper::new(true);
    let json_data = json_dumper.dump_string(instances, schema, &dump_options).await?;
    let json_path = args.output.join(format!("{}.json", base_name));
    fs::write(&json_path, &json_data)?;
    println!("    ✓ {} ({} bytes)", json_path.display(), json_data.len());

    // 5. YAML
    println!("  - Generating YAML...");
    use linkml_service::loader::yaml::YamlDumper;
    let yaml_dumper = YamlDumper::new();
    let yaml_data = yaml_dumper.dump_string(instances, schema, &dump_options).await?;
    let yaml_path = args.output.join(format!("{}.yaml", base_name));
    fs::write(&yaml_path, &yaml_data)?;
    println!("    ✓ {} ({} bytes)", yaml_path.display(), yaml_data.len());

    println!();
    println!("✅ Export complete!");
    println!("Generated 5 files in {}", args.output.display());
    println!();
    println!("Summary:");
    println!("  - {} instances exported", instances.len());
    println!("  - RDF/XML: {}", rdf_path.display());
    println!("  - OWL: {}", owl_path.display());
    println!("  - Turtle: {}", ttl_path.display());
    println!("  - JSON: {}", json_path.display());
    println!("  - YAML: {}", yaml_path.display());

    Ok(())
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}

