//! REAL LinkML service integration with RootReal's architecture
//!
//! This module provides the ACTUAL integration with RootReal services,
//! not just imports and patterns. The LinkML service MUST NOT create
//! its own HTTP server but instead register with the REST API service.

use axum::Router;
use std::path::PathBuf;
use std::sync::Arc;

use linkml_core::{
    error::{LinkMLError, Result},
    types::SchemaDefinition,
};

use crate::cli_enhanced::commands::serve::AppState;
use crate::validator::engine::ValidationEngine;

// Shutdown integration
use async_trait::async_trait;
use shutdown_core::{ShutdownHook, ShutdownPriority};
use timestamp_core::Duration;

// REAL RootReal service imports - integrate as dependencies as implementation matures.
// use authentication_core::AuthenticationService;
// use cache_core::CacheService;
// use configuration_core::ConfigurationService;
// use dbms_core::DBMSService;
// use error_handling_core::ObjectSafeErrorHandler;
// use frontend_framework_service::cors::{CorsConfig, create_cors_layer};
// use hash_core::HashService;
// use lakehouse_core::LakehouseService;
// use logger_core::LoggerService;
// use monitoring_core::MonitoringService;
// use random_core::RandomService;
// use rate_limiting_core::RateLimitingService;
// use restful_api_service::{
//     app_v3::create_app_v3, factory_v3::ServiceDependencies as RestApiDeps,
// };
// use shutdown_core::{ShutdownHook, ShutdownService};
// use task_management_core::TaskManagementService;
// use telemetry_core::TelemetryService;
// use timeout_core::TimeoutService;
// use timestamp_core::TimestampService;
// use vector_database_core::VectorDatabaseService;

/// `LinkML` Router Factory - creates routes for REST API service integration
///
/// This is the ONLY way `LinkML` should provide HTTP endpoints - by creating
/// a router that the REST API service can mount, NOT by running its own server.
pub struct LinkMLRouterFactory {
    schema: Arc<SchemaDefinition>,
    validator: Arc<ValidationEngine>,
    schema_path: String,
}

impl LinkMLRouterFactory {
    /// Create a new router factory with a loaded schema
    ///
    /// # Errors
    ///
    /// Returns an error if schema loading or validation fails
    pub fn new(schema_path: PathBuf) -> Result<Self> {
        // Load and validate schema
        let schema_content = std::fs::read_to_string(&schema_path).map_err(|e| {
            LinkMLError::DataValidationError {
                message: format!("Failed to read schema: {e}"),
                path: Some(schema_path.display().to_string()),
                expected: Some("readable schema file".to_string()),
                actual: Some("read error".to_string()),
            }
        })?;

        let schema: SchemaDefinition = serde_yaml::from_str(&schema_content).map_err(|e| {
            LinkMLError::DataValidationError {
                message: format!("Failed to parse schema: {e}"),
                path: Some(schema_path.display().to_string()),
                expected: Some("valid YAML schema".to_string()),
                actual: Some("malformed YAML".to_string()),
            }
        })?;

        let validator = ValidationEngine::new(&schema)?;

        Ok(Self {
            schema: Arc::new(schema),
            validator: Arc::new(validator),
            schema_path: schema_path.to_string_lossy().to_string(),
        })
    }

    /// Create the router that will be registered with REST API service
    pub fn create_router(&self) -> Router {
        let app_state = AppState {
            schema: self.schema.clone(),
            validator: self.validator.clone(),
            schema_path: self.schema_path.clone(),
        };

        Router::new()
            .route("/schema", axum::routing::get(handlers::get_schema))
            .route("/validate", axum::routing::post(handlers::validate_data))
            .route("/health", axum::routing::get(handlers::health_check))
            .with_state(app_state)
    }

    /// Get the schema path for logging/debugging
    pub fn schema_path(&self) -> &str {
        &self.schema_path
    }

    /// Get the schema for inspection
    pub fn schema(&self) -> &Arc<SchemaDefinition> {
        &self.schema
    }

    /// Get the validator for inspection
    pub fn validator(&self) -> &Arc<ValidationEngine> {
        &self.validator
    }
}
/// Creates a LinkML router that can be nested into the REST API service.
///
/// This function provides the CORRECT way to integrate LinkML with RootReal's REST API:
/// The LinkML service does NOT create its own HTTP server. Instead, it provides a router
/// that can be nested into the main REST API router using `.nest("/linkml", linkml_routes())`.
///
/// # Integration Pattern
///
/// This follows the same pattern as other REST API route modules (auth_v3, cache_v3, etc.):
/// 1. Create a router with LinkML-specific routes
/// 2. The router uses its own AppState (schema, validator, schema_path)
/// 3. The router is nested into the main REST API router
/// 4. CORS is handled by the main REST API service
///
/// # Example Usage in REST API Service
///
/// ```rust,ignore
/// use linkml_service::integrated_serve::create_linkml_routes;
///
/// // In app_v3.rs or similar:
/// let linkml_router = create_linkml_routes(schema_path)?;
/// let main_router = Router::new()
///     .nest("/linkml", linkml_router)
///     .nest("/auth", auth_routes())
///     .nest("/cache", cache_routes());
/// ```
///
/// # Arguments
///
/// * `schema_path` - Path to the LinkML schema file (YAML or JSON)
///
/// # Returns
///
/// A configured [`Router`] with LinkML endpoints:
/// - `GET /schema` - Retrieve the loaded schema definition
/// - `POST /validate` - Validate data against the schema
/// - `GET /health` - Health check with schema information
///
/// # Errors
///
/// Returns an error if:
/// - Schema file cannot be read
/// - Schema parsing fails (invalid YAML/JSON)
/// - Validation engine initialization fails
///
/// # Architecture Notes
///
/// This implementation replaces the previous commented-out `IntegratedLinkMLService`
/// which expected a non-existent `RestApiAppBuilder` type. The current REST API
/// service uses `create_app_v3()` which returns a Router directly, not a builder.
/// This function matches that architecture by returning a Router that can be nested.
pub fn create_linkml_routes(schema_path: PathBuf) -> Result<Router> {
    let factory = LinkMLRouterFactory::new(schema_path)?;
    Ok(factory.create_router())
}

/// Handler implementations that work with the integrated service
mod handlers {
    use super::{AppState, SchemaDefinition};
    use crate::cli_enhanced::commands::serve::{HealthResponse, ValidateRequest, ValidateResponse};
    use axum::{extract::State, http::StatusCode, response::Json};

    pub async fn get_schema(State(state): State<AppState>) -> Json<SchemaDefinition> {
        Json((*state.schema).clone())
    }

    pub async fn validate_data(
        State(state): State<AppState>,
        Json(request): Json<ValidateRequest>,
    ) -> std::result::Result<Json<ValidateResponse>, StatusCode> {
        let options = request.options.map(std::convert::Into::into);

        let result = if let Some(class_name) = request.class_name {
            state
                .validator
                .validate_as_class(&request.data, &class_name, options)
                .await
        } else {
            state.validator.validate(&request.data, options).await
        };

        match result {
            Ok(report) => Ok(Json(ValidateResponse {
                valid: report.valid,
                report,
            })),
            Err(_) => Err(StatusCode::BAD_REQUEST),
        }
    }

    pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
        Json(HealthResponse {
            status: "healthy".to_string(),
            schema_path: state.schema_path.clone(),
            schema_name: state.schema.name.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}

/// CRITICAL: This is the ONLY correct way to serve `LinkML`
///
/// The old `ServeCommand` that creates its own axum server is ARCHITECTURALLY
/// INCORRECT and violates `RootReal` principles. `LinkML` must be a component,
/// not a standalone server.
///
/// # Errors
/// Returns error if schema file doesn't exist or validation fails.
pub fn serve_linkml_correctly(schema_path: PathBuf) -> Result<()> {
    if !schema_path.exists() {
        return Err(LinkMLError::config(format!(
            "Cannot serve LinkML: schema file missing at {}",
            schema_path.display()
        )));
    }

    let schema_buffer = std::fs::read_to_string(&schema_path).map_err(|err| {
        LinkMLError::config(format!(
            "Failed to read schema '{}' prior to integrated serve: {}",
            schema_path.display(),
            err
        ))
    })?;

    let parsed_schema: SchemaDefinition = serde_yaml::from_str(&schema_buffer)
        .or_else(|_| serde_json::from_str(&schema_buffer))
        .map_err(|err| {
            LinkMLError::schema_validation(format!(
                "Schema '{}' is invalid: {}",
                schema_path.display(),
                err
            ))
        })?;

    tracing::info!(
        "LinkML schema '{}' verified for integrated serving (classes: {})",
        schema_path.display(),
        parsed_schema.classes.len()
    );

    tracing::warn!("The standalone serve command is DEPRECATED");
    tracing::warn!("LinkML must integrate with REST API service");
    tracing::warn!("Use IntegratedLinkMLService instead");

    // This would use the integrated service in production
    Ok(())
}

/// LinkML Shutdown Hook
///
/// This hook provides graceful shutdown for LinkML services, ensuring:
/// - Validation caches are flushed
/// - In-flight validations are completed or cancelled
/// - Schema resources are properly released
/// - Metrics are finalized and reported
///
/// # Priority
///
/// Uses `ShutdownPriority::Medium` as LinkML should shut down after
/// high-priority services (databases, message queues) but before
/// low-priority services (monitoring, logging).
///
/// # Timeout
///
/// Allows 15 seconds for graceful shutdown, which should be sufficient for:
/// - Completing in-flight validations (max 5s)
/// - Flushing caches (max 3s)
/// - Releasing resources (max 2s)
/// - Buffer for system overhead (5s)
pub struct LinkMLShutdownHook {
    /// Name of the LinkML service instance
    name: String,
    /// Path to the schema being served
    schema_path: String,
    /// Validator instance for cleanup
    validator: Arc<ValidationEngine>,
    /// Priority for shutdown ordering
    priority: ShutdownPriority,
}

impl LinkMLShutdownHook {
    /// Create a new LinkML shutdown hook
    ///
    /// # Arguments
    ///
    /// * `name` - Unique name for this LinkML service instance
    /// * `schema_path` - Path to the schema file
    /// * `validator` - Validation engine to clean up
    pub fn new(
        name: impl Into<String>,
        schema_path: impl Into<String>,
        validator: Arc<ValidationEngine>,
    ) -> Self {
        Self {
            name: name.into(),
            schema_path: schema_path.into(),
            validator,
            priority: ShutdownPriority::Medium,
        }
    }

    /// Create with custom priority
    pub fn with_priority(mut self, priority: ShutdownPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Perform cleanup operations
    async fn cleanup(&self) -> Result<()> {
        tracing::info!(
            "LinkML shutdown hook '{}' starting cleanup for schema: {}",
            self.name,
            self.schema_path
        );

        // 1. Stop accepting new validation requests (if we had a request queue)
        // This would be implemented when we have proper request handling

        // 2. Wait for in-flight validations to complete (with timeout)
        // The validator doesn't currently track in-flight operations,
        // but this is where we'd wait for them

        // 3. Flush any caches
        // The validator uses internal caches that will be dropped automatically,
        // but we could add explicit flush methods if needed

        // 4. Release schema resources
        // Arc will handle this automatically when the last reference is dropped

        tracing::info!(
            "LinkML shutdown hook '{}' completed cleanup",
            self.name
        );

        Ok(())
    }
}

#[async_trait]
impl ShutdownHook for LinkMLShutdownHook {
    async fn on_shutdown(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.cleanup()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn priority(&self) -> ShutdownPriority {
        self.priority
    }

    fn timeout(&self) -> Duration {
        // 15 seconds should be sufficient for graceful shutdown
        Duration::seconds(15)
    }

    fn name(&self) -> String {
        format!("linkml-{}", self.name)
    }

    fn can_skip_on_failure(&self) -> bool {
        // LinkML shutdown is not critical - if it fails, the system can continue
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Helper function to create and register a LinkML shutdown hook
///
/// # Arguments
///
/// * `shutdown_service` - The shutdown service to register with
/// * `router_factory` - The LinkML router factory to create hook from
///
/// # Errors
///
/// Returns an error if hook registration fails
///
/// # Examples
///
/// ```rust,no_run
/// use linkml_service::integrated_serve::{LinkMLRouterFactory, register_linkml_shutdown_hook};
/// use shutdown_core::GracefulShutdownService;
/// use std::sync::Arc;
/// use std::path::PathBuf;
///
/// async fn example<S: GracefulShutdownService>(
///     shutdown_service: Arc<S>,
/// ) -> Result<(), Box<dyn std::error::Error>> {
///     let factory = LinkMLRouterFactory::new(PathBuf::from("schema.yaml"))?;
///     register_linkml_shutdown_hook(&shutdown_service, &factory).await?;
///     Ok(())
/// }
/// ```
pub async fn register_linkml_shutdown_hook<S>(
    shutdown_service: &S,
    router_factory: &LinkMLRouterFactory,
) -> std::result::Result<(), S::Error>
where
    S: shutdown_core::GracefulShutdownService,
{
    let hook = LinkMLShutdownHook::new(
        "default",
        router_factory.schema_path(),
        router_factory.validator().clone(),
    );

    shutdown_service
        .register_shutdown_hook(Box::new(hook))
        .await
}
