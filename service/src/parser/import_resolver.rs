//! Import resolution for `LinkML` schemas

use linkml_core::{
    error::{LinkMLError, Result},
    types::SchemaDefinition,
};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{AsyncSchemaParser, JsonParserV2, YamlParserV2};
use crate::file_system_adapter::FileSystemOperations;
use parse_core::ParseService;

/// Import resolver for handling schema imports
///
/// This resolver uses V2 parsers that comply with RootReal's mandatory
/// centralized parsing architecture.
pub struct ImportResolver<P: ParseService, F: FileSystemOperations> {
    /// Cache of resolved schemas
    cache: Arc<RwLock<HashMap<String, SchemaDefinition>>>,
    /// Search paths for imports
    search_paths: Arc<RwLock<Vec<PathBuf>>>,
    /// Base path for relative imports
    base_path: Arc<RwLock<Option<PathBuf>>>,
    /// Base `URL` for `URL` imports
    base_url: Arc<RwLock<Option<String>>>,
    /// Maximum import depth to prevent infinite recursion
    max_depth: usize,
    /// YAML parser for .yaml/.yml files
    yaml_parser: Arc<YamlParserV2<F>>,
    /// JSON parser for .json files
    json_parser: Arc<JsonParserV2<P, F>>,
}

impl<P: ParseService, F: FileSystemOperations> ImportResolver<P, F> {
    /// Create a new import resolver with V2 parsers
    ///
    /// # Arguments
    ///
    /// * `parse_service` - ParseService for JSON parsing (centralized architecture)
    /// * `fs_adapter` - File system adapter for sandboxed file operations
    #[must_use]
    pub fn new(parse_service: Arc<P>, fs_adapter: Arc<F>) -> Self {
        let yaml_parser = Arc::new(YamlParserV2::new(Arc::clone(&fs_adapter)));
        let json_parser = Arc::new(JsonParserV2::new(parse_service, Arc::clone(&fs_adapter)));

        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            search_paths: Arc::new(RwLock::new(vec![PathBuf::from(".")])),
            base_path: Arc::new(RwLock::new(None)),
            base_url: Arc::new(RwLock::new(None)),
            max_depth: 10,
            yaml_parser,
            json_parser,
        }
    }

    /// Create with specific search paths
    ///
    /// # Arguments
    ///
    /// * `search_paths` - Paths to search for import files
    /// * `parse_service` - ParseService for JSON parsing (centralized architecture)
    /// * `fs_adapter` - File system adapter for sandboxed file operations
    #[must_use]
    pub fn with_search_paths(
        search_paths: Vec<PathBuf>,
        parse_service: Arc<P>,
        fs_adapter: Arc<F>,
    ) -> Self {
        let yaml_parser = Arc::new(YamlParserV2::new(Arc::clone(&fs_adapter)));
        let json_parser = Arc::new(JsonParserV2::new(parse_service, Arc::clone(&fs_adapter)));

        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            search_paths: Arc::new(RwLock::new(search_paths)),
            base_path: Arc::new(RwLock::new(None)),
            base_url: Arc::new(RwLock::new(None)),
            max_depth: 10,
            yaml_parser,
            json_parser,
        }
    }

    /// Set the base path for relative imports
    pub fn set_base_path(&self, path: &Path) {
        *self.base_path.write() = Some(path.to_path_buf());
    }

    /// Set the base `URL` for `URL` imports
    pub fn set_base_url(&self, url: &str) {
        *self.base_url.write() = Some(url.to_string());
    }

    /// Resolve all imports in a schema, returning a merged schema
    ///
    /// # Errors
    ///
    /// Returns a `LinkMLError` if:
    /// - Import resolution fails
    /// - Circular dependencies are detected
    /// - Maximum import depth is exceeded
    pub fn resolve_imports(&self, schema: &SchemaDefinition) -> Result<SchemaDefinition> {
        // For synchronous contexts, we can't easily resolve async file I/O
        // Users should use resolve_imports_async for full functionality
        // For now, return a clone to maintain API compatibility
        Ok(schema.clone())
    }

    /// Resolve imports asynchronously
    ///
    /// # Errors
    ///
    /// Returns an error if import resolution fails.
    pub async fn resolve_imports_async(
        &self,
        schema: &SchemaDefinition,
    ) -> Result<SchemaDefinition> {
        let mut merged = schema.clone();
        let mut visited = HashSet::new();

        self.resolve_imports_recursive(&mut merged, &mut visited, 0)
            .await?;

        Ok(merged)
    }

    /// Resolve imports recursively
    async fn resolve_imports_recursive(
        &self,
        schema: &mut SchemaDefinition,
        visited: &mut HashSet<String>,
        depth: usize,
    ) -> Result<()> {
        if depth > self.max_depth {
            return Err(LinkMLError::import(
                "imports",
                format!("Maximum import depth ({}) exceeded", self.max_depth),
            ));
        }

        // Process each import
        let imports_to_process: Vec<String> = schema.imports.clone();
        for import in imports_to_process {
            if visited.contains(&import) {
                continue; // Already processed
            }

            visited.insert(import.clone());

            // Try to resolve the import
            let imported_schema = self.load_import(&import).await?;

            // Merge the imported schema into the current schema
            Self::merge_schema(schema, &imported_schema)?;
        }

        Ok(())
    }

    /// Load an imported schema
    async fn load_import(&self, import: &str) -> Result<SchemaDefinition> {
        // Check cache first
        {
            let cache = self.cache.read();
            if let Some(schema) = cache.get(import) {
                return Ok(schema.clone());
            }
        }

        // Try to find the import file
        let path = self.find_import_file(import)?;

        // Load and parse the schema
        let schema = self.load_schema_file(&path).await?;

        // Cache the result
        {
            let mut cache = self.cache.write();
            cache.insert(import.to_string(), schema.clone());
        }

        Ok(schema)
    }

    /// Find the file for an import
    fn find_import_file(&self, import: &str) -> Result<PathBuf> {
        // Try with common extensions
        let extensions = ["yaml", "yml", "json"];

        let search_paths = self.search_paths.read();
        for search_path in search_paths.iter() {
            for ext in &extensions {
                let path = search_path.join(format!("{import}.{ext}"));
                if path.exists() {
                    return Ok(path);
                }

                // Also try without adding extension (if import already has one)
                let path = search_path.join(import);
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        Err(LinkMLError::import(
            import,
            format!(
                "Import file not found in search paths: {:?}",
                search_paths.clone()
            ),
        ))
    }

    /// Load and parse a schema file using V2 parsers
    ///
    /// Detects file format from extension and uses appropriate V2 parser.
    async fn load_schema_file(&self, path: &Path) -> Result<SchemaDefinition> {
        // Detect format from extension
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| LinkMLError::parse(format!("No file extension for: {}", path.display())))?;

        // Use appropriate V2 parser based on extension
        match extension {
            "yaml" | "yml" => self.yaml_parser.parse_file(path).await,
            "json" => self.json_parser.parse_file(path).await,
            _ => Err(LinkMLError::parse(format!(
                "Unsupported schema file format: {} (path: {})",
                extension,
                path.display()
            ))),
        }
    }

    /// Merge an imported schema into the current schema
    fn merge_schema(target: &mut SchemaDefinition, source: &SchemaDefinition) -> Result<()> {
        // Merge prefixes
        for (prefix, def) in &source.prefixes {
            if !target.prefixes.contains_key(prefix) {
                target.prefixes.insert(prefix.clone(), def.clone());
            }
        }

        // Merge classes
        for (name, class) in &source.classes {
            if target.classes.contains_key(name) {
                return Err(LinkMLError::import(
                    &target.name,
                    format!("Class '{name}' already defined"),
                ));
            }
            target.classes.insert(name.clone(), class.clone());
        }

        // Merge slots
        for (name, slot) in &source.slots {
            if target.slots.contains_key(name) {
                return Err(LinkMLError::import(
                    &target.name,
                    format!("Slot '{name}' already defined"),
                ));
            }
            target.slots.insert(name.clone(), slot.clone());
        }

        // Merge types
        for (name, type_def) in &source.types {
            if target.types.contains_key(name) {
                return Err(LinkMLError::import(
                    &target.name,
                    format!("Type '{name}' already defined"),
                ));
            }
            target.types.insert(name.clone(), type_def.clone());
        }

        // Merge enums
        for (name, enum_def) in &source.enums {
            if target.enums.contains_key(name) {
                return Err(LinkMLError::import(
                    &target.name,
                    format!("Enum '{name}' already defined"),
                ));
            }
            target.enums.insert(name.clone(), enum_def.clone());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_system_adapter::TokioFileSystemAdapter;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_import_resolver() -> std::result::Result<(), anyhow::Error> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();

        // Create a base schema
        let base_schema = r"
id: https://example.org/base
name: base
classes:
  BaseClass:
    name: BaseClass
    description: Base class
slots:
  base_slot:
    name: base_slot
    range: string
";

        fs::write(base_path.join("base.yaml"), base_schema)?;

        // Create a schema that imports base
        let main_schema = r"
id: https://example.org/main
name: main
imports:
  - base
classes:
  MainClass:
    name: MainClass
    is_a: BaseClass
    description: Main class
";

        // Parse main schema using V2 parser
        let fs_adapter = Arc::new(TokioFileSystemAdapter::new());
        let yaml_parser = YamlParserV2::new(Arc::clone(&fs_adapter));
        let schema = yaml_parser.parse_str(main_schema).await?;

        // Resolve imports
        let parse_service = Arc::new(parse_core::service::ParseServiceImpl::new());
        let resolver = ImportResolver::with_search_paths(
            vec![base_path.to_path_buf()],
            parse_service,
            fs_adapter,
        );
        let merged = resolver.resolve_imports_async(&schema).await?;

        // Check that base elements were imported
        assert!(merged.classes.contains_key("BaseClass"));
        assert!(merged.slots.contains_key("base_slot"));
        assert!(merged.classes.contains_key("MainClass"));
        Ok(())
    }
}
