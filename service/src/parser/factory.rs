//! Factory functions for creating production-ready LinkML parsers and loaders
//!
//! This module provides convenient factory functions that wire up LinkML components
//! with the external API service for production-ready HTTP imports with rate limiting,
//! caching, retries, and comprehensive logging.

use external_api_core::HttpClient;
use std::sync::Arc;

use super::{ImportResolverV2, SchemaLoader};

/// Create a production-ready schema loader with external API HTTP client
///
/// This creates a schema loader that uses the external API service's HttpClient
/// for all HTTP imports, providing:
/// - Rate limiting to prevent overwhelming external servers
/// - Response caching to avoid redundant fetches
/// - Retry logic for transient failures
/// - Request logging for debugging and monitoring
/// - Authentication support for private repositories
/// - Connection pooling for better performance
///
/// # Arguments
///
/// * `http_client` - The external API HTTP client with all production features
///
/// # Returns
///
/// Returns a configured SchemaLoader ready for production use
///
/// # Example
///
/// ```rust,no_run
/// use linkml_service::parser::factory::create_production_schema_loader;
/// use external_api_service::http::client::StandardHttpClient;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create HTTP client with all dependencies
/// let http_client = StandardHttpClient::new(
///     config,
///     logger,
///     hash_service,
///     cache_service,
///     rate_limiting_service,
/// )?;
///
/// // Create production-ready schema loader
/// let loader = create_production_schema_loader(Arc::new(http_client));
///
/// // Load schemas with full production features
/// let schema = loader.load_file("schema.yaml").await?;
/// # Ok(())
/// # }
/// ```
pub fn create_production_schema_loader(
    http_client: Arc<dyn HttpClient>,
) -> SchemaLoader {
    let resolver = ImportResolverV2::with_http_client(http_client);
    SchemaLoader::with_resolver(resolver)
}

/// Create a schema loader with shared import resolver
///
/// This allows multiple loaders to share the same import resolver (and its cache),
/// which is useful when loading multiple schemas that may have common imports.
///
/// # Arguments
///
/// * `resolver` - Shared import resolver
///
/// # Returns
///
/// Returns a SchemaLoader that shares the resolver's cache
///
/// # Example
///
/// ```rust,no_run
/// use linkml_service::parser::factory::create_schema_loader_with_shared_resolver;
/// use linkml_service::parser::ImportResolverV2;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a shared resolver
/// let resolver = Arc::new(ImportResolverV2::with_http_client(http_client));
///
/// // Create multiple loaders that share the same cache
/// let loader1 = create_schema_loader_with_shared_resolver(resolver.clone());
/// let loader2 = create_schema_loader_with_shared_resolver(resolver.clone());
///
/// // Both loaders will benefit from shared cache
/// let schema1 = loader1.load_file("schema1.yaml").await?;
/// let schema2 = loader2.load_file("schema2.yaml").await?;
/// # Ok(())
/// # }
/// ```
pub fn create_schema_loader_with_shared_resolver(
    resolver: Arc<ImportResolverV2>,
) -> SchemaLoader {
    SchemaLoader::with_shared_resolver(resolver)
}

/// Create a development/test schema loader
///
/// This creates a simple schema loader without external API integration,
/// suitable for development and testing where you don't need production features.
///
/// # Returns
///
/// Returns a basic SchemaLoader using fallback HTTP client
///
/// # Example
///
/// ```rust
/// use linkml_service::parser::factory::create_dev_schema_loader;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create simple loader for tests
/// let loader = create_dev_schema_loader();
///
/// // Load schemas (uses fallback client)
/// let schema = loader.load_file("schema.yaml").await?;
/// # Ok(())
/// # }
/// ```
pub fn create_dev_schema_loader() -> SchemaLoader {
    SchemaLoader::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dev_loader() {
        let _loader = create_dev_schema_loader();
        // Just verify it compiles and creates successfully
    }
}

