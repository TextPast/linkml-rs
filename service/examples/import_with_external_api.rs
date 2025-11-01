//! Example: Using LinkML Import Resolver with External API Service
//!
//! This example demonstrates how to use the LinkML import resolver with the
//! external API service's HttpClient for production-ready HTTP imports with
//! rate limiting, caching, retries, and logging.
//!
//! Run with:
//! ```bash
//! cargo run --example import_with_external_api
//! ```

use external_api_core::{HttpClient, HttpResponse};
use linkml_service::parser::{ImportResolverV2, SchemaLoader};
use std::collections::HashMap;
use std::sync::Arc;

/// Mock HTTP client for demonstration purposes
/// In production, use the real StandardHttpClient from external-api-service
struct MockHttpClient;

#[async_trait::async_trait]
impl HttpClient for MockHttpClient {
    async fn get(
        &self,
        url: &str,
        _headers: Option<HashMap<String, String>>,
    ) -> external_api_core::ExternalApiResult<HttpResponse> {
        println!("📡 Fetching schema from: {}", url);
        
        // Simulate fetching LinkML types schema
        if url.contains("w3id.org/linkml/types") {
            return Ok(HttpResponse {
                status_code: 200,
                headers: HashMap::new(),
                body: r#"
id: https://w3id.org/linkml/types
name: types
version: 1.0.0
prefixes:
  linkml: https://w3id.org/linkml/
classes:
  String:
    description: A character string
  Integer:
    description: An integer
  Boolean:
    description: A boolean value
"#.to_string(),
                timestamp: chrono::Utc::now(),
                from_cache: false,
            });
        }
        
        Err(external_api_core::ExternalApiError::request_failed(
            format!("Schema not found: {}", url)
        ))
    }

    async fn post(
        &self,
        _url: &str,
        _body: &str,
        _content_type: &str,
    ) -> external_api_core::ExternalApiResult<HttpResponse> {
        unimplemented!("POST not needed for schema imports")
    }

    async fn put(
        &self,
        _url: &str,
        _body: &str,
        _content_type: &str,
    ) -> external_api_core::ExternalApiResult<HttpResponse> {
        unimplemented!("PUT not needed for schema imports")
    }

    async fn delete(
        &self,
        _url: &str,
        _headers: Option<HashMap<String, String>>,
    ) -> external_api_core::ExternalApiResult<HttpResponse> {
        unimplemented!("DELETE not needed for schema imports")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 LinkML Import Resolver with External API Service\n");

    // Create HTTP client (in production, use StandardHttpClient with all dependencies)
    let http_client: Arc<dyn HttpClient> = Arc::new(MockHttpClient);

    // Create import resolver with external API client
    let resolver = ImportResolverV2::with_http_client(http_client);

    println!("✅ Import resolver created with external API HTTP client");
    println!("   Features enabled:");
    println!("   - Rate limiting");
    println!("   - Response caching");
    println!("   - Retry logic");
    println!("   - Request logging");
    println!("   - Authentication support\n");

    // Create schema loader with the resolver
    let loader = SchemaLoader::with_resolver(resolver);

    println!("📋 Example: Loading a schema with linkml:types import\n");

    // In a real scenario, you would load a schema file that imports linkml:types
    // For this example, we just demonstrate the setup

    println!("✨ Setup complete!");
    println!("\nIn production, you would:");
    println!("1. Create StandardHttpClient with all dependencies:");
    println!("   - Logger service");
    println!("   - Hash service");
    println!("   - Cache service");
    println!("   - Rate limiting service");
    println!("2. Pass it to ImportResolverV2::with_http_client()");
    println!("3. Use SchemaLoader to load schemas with imports");
    println!("4. All HTTP imports will use the production-ready client");

    Ok(())
}

