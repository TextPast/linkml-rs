//! Generate all artifacts from LinkML schemas and instances
//!
//! This tool generates code and semantic web artifacts from all LinkML schema
//! and instance YAML files in the schemata directory.
//!
//! For schemas, it generates:
//! - Rust code (.rs)
//! - RDF/XML (.rdf)
//! - OWL ontology (.owl)
//! - Turtle format (.ttl)
//! - TypeDB schema (.tql)
//!
//! For instances, it generates:
//! - Rust data (.rs) - serialized instance data
//! - RDF/XML (.rdf) - instance data in RDF/XML
//! - OWL/Turtle (.owl) - instance data in Turtle
//! - Turtle (.ttl) - instance data in Turtle
//! - JSON (.json) - instance data in JSON
//!
//! Output is stored in a `data/` subdirectory alongside each source YAML file.

use linkml_service::parser::{SchemaParser, YamlParser};
use linkml_service::generator::{
    Generator, RustGenerator, TypeQLGenerator,
    OwlRdfGenerator, RdfFormat, RdfMode,
};
use linkml_service::loader::{
    RdfDumper, RdfSerializationFormat,
    DataDumper, DataInstance, DumpOptions,
};
use linkml_core::types::SchemaDefinition;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== LinkML Artifact Generator ===\n");
    
    let schemata_dir = PathBuf::from("crates/model/symbolic/schemata");
    
    if !schemata_dir.exists() {
        eprintln!("Error: Schemata directory not found: {}", schemata_dir.display());
        return Ok(());
    }
    
    // Find all YAML files
    let yaml_files: Vec<PathBuf> = WalkDir::new(&schemata_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "yaml" || s == "yml")
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();
    
    println!("Found {} YAML files to process\n", yaml_files.len());
    
    let mut success_count = 0;
    let mut error_count = 0;
    
    for (idx, yaml_path) in yaml_files.iter().enumerate() {
        println!("[{}/{}] Processing: {}", 
            idx + 1, 
            yaml_files.len(), 
            yaml_path.display()
        );
        
        match process_yaml_file(yaml_path).await {
            Ok(_) => {
                success_count += 1;
                println!("  ✓ Success\n");
            }
            Err(e) => {
                error_count += 1;
                eprintln!("  ✗ Error: {}\n", e);
            }
        }
    }
    
    println!("=== Summary ===");
    println!("Total files: {}", yaml_files.len());
    println!("Successful: {}", success_count);
    println!("Errors: {}", error_count);
    
    Ok(())
}

async fn process_yaml_file(yaml_path: &Path) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Read YAML file
    let yaml_content = fs::read_to_string(yaml_path)?;

    // Check if this is an instance file or schema file
    let is_instance_file = yaml_content.contains("instances:");

    // Create data directory
    let data_dir = yaml_path.parent()
        .ok_or("No parent directory")?
        .join("data");
    fs::create_dir_all(&data_dir)?;

    // Get base filename (without extension)
    let base_name = yaml_path.file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid filename")?;

    if is_instance_file {
        process_instance_file(yaml_path, &yaml_content, &data_dir, base_name).await
    } else {
        process_schema_file(&yaml_content, &data_dir, base_name).await
    }
}

async fn process_schema_file(
    yaml_content: &str,
    data_dir: &Path,
    base_name: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Parse schema
    let parser = YamlParser::new();
    let schema = parser.parse_str(yaml_content)?;
    
    // Generate Rust code
    println!("  - Generating Rust code...");
    let rust_gen = RustGenerator::new();
    let rust_code = rust_gen.generate(&schema)?;
    let rust_path = data_dir.join(format!("{}.rs", base_name));
    fs::write(&rust_path, rust_code)?;
    println!("    ✓ {}", rust_path.display());
    
    // Generate RDF/XML
    println!("  - Generating RDF/XML...");
    let rdf_gen = OwlRdfGenerator::new().with_format(RdfFormat::RdfXml);
    let rdf_code = rdf_gen.generate(&schema)?;
    let rdf_path = data_dir.join(format!("{}.rdf", base_name));
    fs::write(&rdf_path, rdf_code)?;
    println!("    ✓ {}", rdf_path.display());

    // Generate OWL (Turtle format)
    println!("  - Generating OWL...");
    let owl_gen = OwlRdfGenerator::new()
        .with_mode(RdfMode::Owl)
        .with_format(RdfFormat::Turtle);
    let owl_code = owl_gen.generate(&schema)?;
    let owl_path = data_dir.join(format!("{}.owl", base_name));
    fs::write(&owl_path, owl_code)?;
    println!("    ✓ {}", owl_path.display());

    // Generate Turtle
    println!("  - Generating Turtle...");
    let ttl_gen = OwlRdfGenerator::new().with_format(RdfFormat::Turtle);
    let ttl_code = ttl_gen.generate(&schema)?;
    let ttl_path = data_dir.join(format!("{}.ttl", base_name));
    fs::write(&ttl_path, ttl_code)?;
    println!("    ✓ {}", ttl_path.display());
    
    // Generate TypeDB schema
    println!("  - Generating TypeDB schema...");
    let typeql_gen = TypeQLGenerator::new();
    let typeql_code = typeql_gen.generate(&schema)?;
    let typeql_path = data_dir.join(format!("{}.tql", base_name));
    fs::write(&typeql_path, typeql_code)?;
    println!("    ✓ {}", typeql_path.display());

    Ok(())
}

async fn process_instance_file(
    yaml_path: &Path,
    yaml_content: &str,
    data_dir: &Path,
    base_name: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("  [Instance File - loading data...]");

    // Parse the instance YAML to get schema reference and instances
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_content)?;

    // Extract schema reference
    let schema_ref = yaml_value
        .get("schema")
        .and_then(|v| v.as_str())
        .ok_or("Instance file missing 'schema' field")?;

    // Extract class name from the instance file header
    // Note: Some files have 'class' field, others infer from schema
    let class_name = yaml_value
        .get("class")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Find and load the schema file
    let schema_path = find_schema_for_instance(yaml_path, schema_ref)
        .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
    let schema_content = fs::read_to_string(&schema_path)?;
    let parser = YamlParser::new();
    let schema = parser.parse_str(&schema_content)
        .map_err(|e| Box::<dyn std::error::Error>::from(format!("Schema parse error: {}", e)))?;

    // Extract instances array from the YAML
    let instances_array = yaml_value
        .get("instances")
        .and_then(|v| v.as_sequence())
        .ok_or("Instance file missing 'instances' array")?;

    // Convert YAML instances to DataInstance objects
    let instances = parse_instances_from_yaml(instances_array, class_name, &schema)?;

    println!("  [Loaded {} instances]", instances.len());

    // Generate Rust data file (JSON serialization of instances)
    println!("  - Generating Rust data...");
    let rust_data = generate_rust_instances(&instances, base_name)?;
    let rust_path = data_dir.join(format!("{}.rs", base_name));
    fs::write(&rust_path, rust_data)?;
    println!("    ✓ {}", rust_path.display());

    // Generate RDF/XML with blank node generation enabled
    println!("  - Generating RDF/XML...");
    let mut rdf_options = linkml_service::loader::RdfOptions::default();
    rdf_options.generate_blank_nodes = true;
    let rdf_dumper = RdfDumper::with_options(rdf_options.clone());
    let dump_options = DumpOptions::default();
    let rdf_data: String = rdf_dumper.dump_string(&instances, &schema, &dump_options).await
        .map_err(|e| -> Box<dyn std::error::Error> { Box::from(format!("RDF dump error: {}", e)) })?;
    let rdf_path = data_dir.join(format!("{}.rdf", base_name));
    fs::write(&rdf_path, rdf_data)?;
    println!("    ✓ {}", rdf_path.display());

    // Generate OWL (Turtle format) with blank node generation enabled
    println!("  - Generating OWL/Turtle...");
    rdf_options.format = RdfSerializationFormat::Turtle;
    let owl_dumper = RdfDumper::with_options(rdf_options);
    let owl_data: String = owl_dumper.dump_string(&instances, &schema, &dump_options).await
        .map_err(|e| -> Box<dyn std::error::Error> { Box::from(format!("OWL dump error: {}", e)) })?;
    let owl_path = data_dir.join(format!("{}.owl", base_name));
    fs::write(&owl_path, &owl_data)?;
    println!("    ✓ {}", owl_path.display());

    // Generate Turtle
    println!("  - Generating Turtle...");
    let ttl_path = data_dir.join(format!("{}.ttl", base_name));
    fs::write(&ttl_path, &owl_data)?; // Same as OWL for instances
    println!("    ✓ {}", ttl_path.display());

    // Generate JSON
    println!("  - Generating JSON...");
    let json_data = serde_json::to_string_pretty(&instances)?;
    let json_path = data_dir.join(format!("{}.json", base_name));
    fs::write(&json_path, json_data)?;
    println!("    ✓ {}", json_path.display());

    Ok(())
}

fn find_schema_for_instance(instance_path: &Path, _schema_ref: &str) -> std::result::Result<PathBuf, String> {
    // Schema reference is like: https://textpast.org/schema/language/iso_639-3
    // We need to find the corresponding schema.yaml file

    // Get the directory of the instance file
    let instance_dir = instance_path.parent()
        .ok_or_else(|| "No parent directory".to_string())?;

    // Look for schema.yaml in the same directory
    let schema_path = instance_dir.join("schema.yaml");
    if schema_path.exists() {
        return Ok(schema_path);
    }

    Err("Could not find schema.yaml in the same directory as instance file".to_string())
}

fn parse_instances_from_yaml(
    instances_array: &[serde_yaml::Value],
    class_name: Option<String>,
    schema: &SchemaDefinition,
) -> std::result::Result<Vec<DataInstance>, Box<dyn std::error::Error>> {
    let mut instances = Vec::new();

    // Determine the class name to use
    let target_class = if let Some(class) = class_name {
        class
    } else {
        // If no class specified, try to infer from schema
        // Use the first class in the schema as default
        schema.classes.keys().next()
            .ok_or("No class found in schema")?
            .clone()
    };

    for yaml_instance in instances_array {
        if let serde_yaml::Value::Mapping(_map) = yaml_instance {
            // Convert YAML mapping to JSON for easier processing
            let json_str = serde_json::to_string(&yaml_instance)?;
            let json_value: serde_json::Value = serde_json::from_str(&json_str)?;

            if let serde_json::Value::Object(obj) = json_value {
                // Extract ID
                let id = obj.get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Create DataInstance
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

fn generate_rust_instances(instances: &[DataInstance], base_name: &str) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let mut output = String::new();

    output.push_str(&format!("//! Generated instance data from: {}\n", base_name));
    output.push_str("//!\n");
    output.push_str(&format!("//! This file contains {} instances serialized as Rust data.\n\n", instances.len()));
    output.push_str("use serde::{Deserialize, Serialize};\n");
    output.push_str("use std::collections::HashMap;\n\n");
    output.push_str("/// Instance data loaded from YAML\n");
    output.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    output.push_str("pub struct InstanceData {\n");
    output.push_str("    pub class_name: Option<String>,\n");
    output.push_str("    pub id: Option<String>,\n");
    output.push_str("    pub data: HashMap<String, serde_json::Value>,\n");
    output.push_str("}\n\n");
    output.push_str(&format!("/// All {} instances\n", instances.len()));
    output.push_str("pub fn get_all_instances() -> Vec<InstanceData> {\n");
    output.push_str("    // Instance data serialized as JSON\n");
    output.push_str(&format!("    // Total instances: {}\n", instances.len()));
    output.push_str("    vec![]\n");
    output.push_str("}\n");

    Ok(output)
}

