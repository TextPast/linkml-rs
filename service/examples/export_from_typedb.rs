//! Export data from TypeDB to all formats (RDF/OWL/Turtle/YAML/JSON)
//!
//! This example demonstrates Phase 2 of the bidirectional conversion pipeline:
//! TypeDB → RDF/OWL/Turtle/YAML/JSON
//!
//! Usage:
//!   cargo run --example export_from_typedb -- \
//!     --database rootreal \
//!     --class Translation \
//!     --schema crates/model/symbolic/schemata/language/iso_639-3/schema.yaml \
//!     --output crates/model/symbolic/schemata/language/iso_639-3/data/

use clap::Parser;
use linkml_core::types::SchemaDefinition;
use linkml_service::parser::{Parser as LinkmlParser, SchemaLoader};
use linkml_service::loader::{
    DataDumper, DumpOptions,
    RdfDumper, RdfSerializationFormat, RdfOptions,
    TypeDBIntegrationLoader, TypeDBIntegrationOptions,
    DataLoader, LoadOptions,
};
use logger_service::wiring::wire_testing_logger;
use parse_service::NoLinkML;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "export_from_typedb")]
#[command(about = "Export data from TypeDB to all formats", long_about = None)]
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
    #[arg(long, default_value = "localhost:1729")]
    server: String,

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

    println!("=== TypeDB Export Tool ===\n");
    println!("Configuration:");
    println!("  Database: {}", args.database);
    println!("  Class: {}", args.class);
    println!("  Schema: {}", args.schema.display());
    println!("  Output: {}", args.output.display());
    println!("  Server: {}", args.server);
    println!();

    // Wire ParseService
    println!("Wiring ParseService...");
    let logger = wire_testing_logger()?.into_arc();
    let parse_service_handle = parse_service::wiring::wire_parse_for_testing::<NoLinkML>(logger.clone())
        .await
        .map_err(|e| Box::<dyn std::error::Error>::from(format!("Parse service wiring error: {}", e)))?;
    let parse_service: Arc<dyn parse_core::ParseService<Error = parse_core::ParseError>> = 
        parse_service_handle.into_arc();
    println!("  ✓ ParseService wired");

    // Load LinkML schema
    println!("Loading LinkML schema...");
    let loader = SchemaLoader::new(parse_service);
    let schema: SchemaDefinition = loader.load_file(&args.schema)
        .await
        .map_err(|e| Box::<dyn std::error::Error>::from(format!("Schema load error: {}", e)))?;
    println!("  ✓ Schema loaded: {}", schema.name);

    // Create output directory
    fs::create_dir_all(&args.output)?;
    println!("  ✓ Output directory ready");
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
    let typedb_type = args.typedb_type.as_ref().map(|s| s.clone()).unwrap_or_else(|| to_snake_case(&args.class));
    options.type_mapping.insert(typedb_type.clone(), args.class.clone());
    println!("  ✓ Type mapping: {} → {}", typedb_type, args.class);

    // Create mock executor for demonstration
    // In production, use DBMSServiceExecutor with real DBMS service
    let executor = MockTypeDBExecutor::new();
    println!("  ⚠ Using mock executor (replace with DBMSServiceExecutor in production)");
    println!();

    // Create loader
    println!("Loading instances from TypeDB...");
    let loader = TypeDBIntegrationLoader::new(options, executor);
    let load_options = LoadOptions::default();
    
    // Note: This will fail with mock executor - replace with real DBMS service
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
            println!("To use this tool with a real TypeDB instance:");
            println!("  1. Replace MockTypeDBExecutor with DBMSServiceExecutor");
            println!("  2. Provide a StandardDBMSService instance");
            println!("  3. Ensure TypeDB is running and accessible");
            println!();
            println!("Example with real DBMS service:");
            println!("  let dbms_service = get_dbms_service(); // From DI container");
            println!("  let executor = DBMSServiceExecutor::new(dbms_service);");
            println!("  let loader = TypeDBIntegrationLoader::new(options, executor);");
        }
    }

    Ok(())
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
    fs::write(&ttl_path, &owl_data)?; // Same as OWL for instances
    println!("    ✓ {} ({} bytes)", ttl_path.display(), owl_data.len());

    // 4. JSON
    println!("  - Generating JSON...");
    use linkml_service::loader::json::JsonDumper;
    let json_dumper = JsonDumper::new(true); // pretty print
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

// Mock executor for demonstration
struct MockTypeDBExecutor;

impl MockTypeDBExecutor {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl linkml_service::loader::typedb_integration::TypeDBQueryExecutor for MockTypeDBExecutor {
    async fn execute_query(
        &self,
        _query: &str,
        _database: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error>> {
        Err("Mock executor - replace with DBMSServiceExecutor".into())
    }

    async fn execute_define(
        &self,
        _query: &str,
        _database: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Err("Mock executor - replace with DBMSServiceExecutor".into())
    }

    async fn execute_insert(
        &self,
        _query: &str,
        _database: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Err("Mock executor - replace with DBMSServiceExecutor".into())
    }
}

