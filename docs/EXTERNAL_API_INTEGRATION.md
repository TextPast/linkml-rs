# External API Service Integration

## Overview

The LinkML import resolver now integrates with RootReal's external API service to provide production-ready HTTP imports with rate limiting, caching, retries, and comprehensive logging.

## Features

### 1. Rate Limiting
Prevents overwhelming external schema servers by limiting the number of requests per time period.

### 2. Response Caching
Avoids redundant fetches of the same schema by caching responses. Subsequent imports of the same schema are served from cache.

### 3. Retry Logic
Handles transient network failures gracefully with exponential backoff and configurable retry attempts.

### 4. Request Logging
Tracks all external schema fetches for debugging and monitoring purposes.

### 5. Authentication Support
Supports various authentication methods for private schema repositories:
- API keys
- Bearer tokens
- Basic authentication
- Custom headers

### 6. Connection Pooling
Reuses HTTP connections for better performance when loading multiple schemas.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    LinkML Import Resolver                    │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  ImportResolverV2                                     │  │
│  │                                                        │  │
│  │  ┌──────────────────┐  ┌──────────────────────────┐ │  │
│  │  │ HttpClient       │  │ Fallback reqwest::Client │ │  │
│  │  │ (Optional)       │  │ (Always available)       │ │  │
│  │  │                  │  │                          │ │  │
│  │  │ - Rate limiting  │  │ - Simple HTTP GET        │ │  │
│  │  │ - Caching        │  │ - No extra features      │ │  │
│  │  │ - Retries        │  │ - Used for tests         │ │  │
│  │  │ - Logging        │  │                          │ │  │
│  │  └──────────────────┘  └──────────────────────────┘ │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Usage

### Basic Usage (Tests/Development)

```rust
use linkml_service::parser::{ImportResolverV2, SchemaLoader};

// Uses fallback reqwest client (no extra features)
let resolver = ImportResolverV2::new();
let loader = SchemaLoader::with_resolver(resolver);
```

### Production Usage (With External API Service)

```rust
use linkml_service::parser::{ImportResolverV2, SchemaLoader};
use external_api_service::http::client::StandardHttpClient;
use std::sync::Arc;

// Create HTTP client with all production features
let http_client = StandardHttpClient::new(
    config,
    logger,
    hash_service,
    cache_service,
    rate_limiting_service,
)?;

// Create import resolver with external API client
let resolver = ImportResolverV2::with_http_client(Arc::new(http_client));

// Create schema loader
let loader = SchemaLoader::with_resolver(resolver);

// Load schema with imports (uses production-ready HTTP client)
let schema = loader.load_file("schema.yaml").await?;
```

### With Custom Settings

```rust
use linkml_core::settings::ImportSettings;

let settings = ImportSettings {
    base_url: Some("https://schemas.example.com".to_string()),
    search_paths: vec![PathBuf::from("./schemas")],
    // ... other settings
};

let resolver = ImportResolverV2::with_settings_and_client(settings, http_client);
```

## Configuration

The HTTP client can be configured through the external API service configuration:

```rust
use external_api_service::http::client::HttpClientConfig;
use std::time::Duration;

let config = HttpClientConfig {
    timeout_settings: TimeoutConfig {
        connect_timeout: Duration::from_secs(5),
        request_timeout: Duration::from_secs(30),
        read_timeout: Duration::from_secs(10),
    },
    enable_caching: true,
    cache_ttl: Duration::from_secs(3600),  // 1 hour
    max_retries: 3,
    retry_delay: Duration::from_millis(100),
    user_agent: "LinkML-Rust/2.0.0".to_string(),
    custom_headers: HashMap::new(),
    enabled: true,
};
```

## Benefits

1. **Consistency** - All HTTP handling uses the same production-ready client across RootReal
2. **Reliability** - Automatic retries and error handling for transient failures
3. **Performance** - Response caching and connection pooling reduce latency
4. **Observability** - Comprehensive logging for debugging and monitoring
5. **Security** - Support for authentication and secure connections
6. **Compliance** - Follows RootReal architectural patterns (dependency injection)

## Examples

See `examples/import_with_external_api.rs` for a complete working example.

## Testing

All existing tests continue to work using the fallback reqwest client. No changes required to test code.

```bash
cargo test --package rootreal-model-symbolic-linkml
```

## Migration Guide

### For Existing Code

No changes required! The import resolver is backward compatible:

```rust
// This still works (uses fallback client)
let resolver = ImportResolverV2::new();
```

### To Enable Production Features

Simply provide an HttpClient:

```rust
// Add this to enable all production features
let resolver = ImportResolverV2::with_http_client(http_client);
```

## See Also

- [External API Service Documentation](../../../../hub/api/integration/external/service/README.md)
- [LinkML Service README](../README.md)
- [TextPast Conventions](TEXTPAST_CONVENTIONS.md)

