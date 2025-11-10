//! Schema loader for loading schemas from files and URLs

use crate::file_system_adapter::TokioFileSystemAdapter;
use linkml_core::{
    error::{LinkMLError, Result},
    settings::ImportSettings,
    types::SchemaDefinition,
};
use reqwest;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

use super::{AsyncSchemaParser, ImportResolverV2, YamlParserV2};

/// Loader for `LinkML` schemas from various sources
pub struct SchemaLoader {
    yaml_parser: YamlParserV2<TokioFileSystemAdapter>,
    fs_adapter: Arc<TokioFileSystemAdapter>,
    http_client: reqwest::Client,
    /// Optional import resolver with custom HTTP client
    import_resolver: Option<Arc<ImportResolverV2>>,
}

impl SchemaLoader {
    /// Create a new schema loader
    #[must_use]
    pub fn new() -> Self {
        let fs_adapter = Arc::new(TokioFileSystemAdapter::new());
        
        Self {
            yaml_parser: YamlParserV2::new(Arc::clone(&fs_adapter)),
            fs_adapter,
            http_client: reqwest::Client::new(),
            import_resolver: None,
        }
    }

    /// Create a schema loader with a custom import resolver
    ///
    /// This allows using a production-ready HTTP client with rate limiting,
    /// caching, retries, and logging for schema imports.
    #[must_use]
    pub fn with_resolver(resolver: ImportResolverV2) -> Self {
        let fs_adapter = Arc::new(TokioFileSystemAdapter::new());
        
        Self {
            yaml_parser: YamlParserV2::new(Arc::clone(&fs_adapter)),
            fs_adapter,
            http_client: reqwest::Client::new(),
            import_resolver: Some(Arc::new(resolver)),
        }
    }

    /// Create a schema loader with a shared import resolver
    ///
    /// This allows sharing the same resolver (and its cache) across multiple loaders.
    #[must_use]
    pub fn with_shared_resolver(resolver: Arc<ImportResolverV2>) -> Self {
        let fs_adapter = Arc::new(TokioFileSystemAdapter::new());
        
        Self {
            yaml_parser: YamlParserV2::new(Arc::clone(&fs_adapter)),
            fs_adapter,
            http_client: reqwest::Client::new(),
            import_resolver: Some(resolver),
        }
    }

    /// Parse JSON LinkML schema content directly
    /// 
    /// JSON LinkML schemas are rare (most are YAML), so we use direct serde_json
    /// deserialization instead of the full ParseService. This keeps the SchemaLoader
    /// lightweight and avoids complex dependency injection for an edge case.
    fn parse_json_schema(&self, content: &str) -> Result<SchemaDefinition> {
        serde_json::from_str(content).map_err(|e| {
            LinkMLError::parse_at(
                format!("JSON deserialization error: {e}"),
                format!("line {}, column {}", e.line(), e.column()),
            )
        })
    }

    /// Load a schema from a file path
    /// Returns an error if the operation fails
    ///
    /// # Errors
    ///
    pub async fn load_file(&self, path: impl AsRef<Path>) -> Result<SchemaDefinition> {
        let path = path.as_ref();

        // Read file content
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| LinkMLError::service(format!("Failed to read file: {e}")))?;

        // Determine format from extension
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| LinkMLError::parse("No file extension found"))?;

        // Parse the schema using V2 parsers
        let schema = match extension {
            "json" => self.parse_json_schema(&content)?,
            "yaml" | "yml" => self.yaml_parser.parse_str(&content).await?,
            _ => return Err(LinkMLError::parse(format!("Unsupported file extension: {extension}"))),
        };

        // Set up import settings with the file's parent directory as search path
        let mut settings = ImportSettings::default();
        if let Some(parent) = path.parent() {
            settings
                .search_paths
                .push(parent.to_string_lossy().to_string());
        }

        // Use schema settings if available
        if let Some(schema_settings) = &schema.settings
            && let Some(import_settings) = &schema_settings.imports
        {
            settings = import_settings.clone();

            // Resolve relative search paths from schema settings
            if let Some(parent) = path.parent() {
                // Make relative paths absolute based on schema location
                settings.search_paths = settings
                    .search_paths
                    .iter()
                    .map(|p| {
                        let path_buf = PathBuf::from(p);
                        if path_buf.is_relative() {
                            parent.join(path_buf).to_string_lossy().to_string()
                        } else {
                            p.clone()
                        }
                    })
                    .collect();

                // Also add the parent directory if not already present
                let parent_str = parent.to_string_lossy().to_string();
                if !settings.search_paths.contains(&parent_str) {
                    settings.search_paths.push(parent_str);
                }
            }
        }

        // Resolve imports using custom resolver if available, otherwise create one
        if let Some(ref resolver) = self.import_resolver {
            // Use the provided resolver (may have production HTTP client)
            resolver.resolve_imports(&schema).await
        } else {
            // Create a new resolver with settings
            let import_resolver = ImportResolverV2::with_settings(settings);
            import_resolver.resolve_imports(&schema).await
        }
    }

    /// Load a schema from a `URL`
    /// Returns an error if the operation fails
    ///
    /// # Errors
    ///
    pub async fn load_url(&self, url: &str) -> Result<SchemaDefinition> {
        // Fetch content from URL
        let response = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| LinkMLError::service(format!("Failed to fetch URL: {e}")))?;

        if !response.status().is_success() {
            return Err(LinkMLError::service(format!(
                "HTTP error {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let content = response
            .text()
            .await
            .map_err(|e| LinkMLError::service(format!("Failed to read response: {e}")))?;

        // Determine format from URL extension or content type (case-insensitive)
        let url_lower = url.to_lowercase();
        let is_json = std::path::Path::new(&url_lower)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"));
        
        let is_yaml = std::path::Path::new(&url_lower)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"));

        // Parse the schema using V2 parsers (default to YAML as it's more common for LinkML)
        let schema = if is_json {
            self.parse_json_schema(&content)?
        } else if is_yaml {
            self.yaml_parser.parse_str(&content).await?
        } else {
            // Default to YAML
            self.yaml_parser.parse_str(&content).await?
        };

        // Set up import settings with URL base
        let mut settings = ImportSettings::default();

        // Add base URL path for relative URL imports
        if let Ok(parsed_url) = url::Url::parse(url)
            && let Ok(base) = parsed_url.join("./")
        {
            settings.base_url = Some(base.to_string());
        }

        // Use schema settings if available
        if let Some(schema_settings) = &schema.settings
            && let Some(import_settings) = &schema_settings.imports
        {
            settings = import_settings.clone();
            // Still set base URL if not already set
            if settings.base_url.is_none()
                && let Ok(parsed_url) = url::Url::parse(url)
                && let Ok(base) = parsed_url.join("./")
            {
                settings.base_url = Some(base.to_string());
            }
        }

        // Resolve imports using custom resolver if available, otherwise create one
        if let Some(ref resolver) = self.import_resolver {
            resolver.resolve_imports(&schema).await
        } else {
            let import_resolver = ImportResolverV2::with_settings(settings);
            import_resolver.resolve_imports(&schema).await
        }
    }

    /// Load a schema from a string with specified format
    /// Returns an error if the operation fails
    ///
    /// # Errors
    ///
    pub async fn load_string(&self, content: &str, format: &str) -> Result<SchemaDefinition> {
        let schema = match format {
            "json" => self.parse_json_schema(content)?,
            "yaml" | "yml" => self.yaml_parser.parse_str(content).await?,
            _ => return Err(LinkMLError::parse(format!("Unsupported format: {format}"))),
        };

        // Use schema settings if available, otherwise defaults
        let settings = if let Some(schema_settings) = &schema.settings {
            schema_settings.imports.clone().unwrap_or_default()
        } else {
            ImportSettings::default()
        };

        // Resolve imports using custom resolver if available, otherwise create one
        if let Some(ref resolver) = self.import_resolver {
            resolver.resolve_imports(&schema).await
        } else {
            let import_resolver = ImportResolverV2::with_settings(settings);
            import_resolver.resolve_imports(&schema).await
        }
    }
}


