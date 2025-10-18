//! Event handler for LinkML validation requests
//!
//! This module implements the LinkML service's subscription to validation request events
//! from the GraphQL service, allowing proper validation without circular dependencies.

use async_trait::async_trait;
use event_bus_core::{
    Event, EventBusService, EventHandler, Topic,
    LinkMLValidationRequest, LinkMLValidationResult,
};
use std::sync::Arc;
use chrono::Utc;

/// Handler for LinkML validation requests from the event bus
pub struct LinkMLValidationRequestHandler {
    /// Event bus for publishing validation results
    event_bus: Arc<dyn EventBusService<Error = event_bus_core::EventBusError>>,
    /// Reference to the LinkML validation service
    linkml_service: Arc<dyn LinkMLValidationService>,
}

/// Trait for LinkML validation services
#[async_trait]
pub trait LinkMLValidationService: Send + Sync {
    /// Validate an ontology's LinkML schema
    async fn validate_linkml_schema(
        &self,
        ontology_id: String,
    ) -> Result<(bool, Vec<String>), String>;
}

impl LinkMLValidationRequestHandler {
    /// Create a new validation request handler
    pub fn new(
        event_bus: Arc<dyn EventBusService<Error = event_bus_core::EventBusError>>,
        linkml_service: Arc<dyn LinkMLValidationService>,
    ) -> Self {
        Self {
            event_bus,
            linkml_service,
        }
    }

    /// Register this handler with the event bus
    pub async fn register(
        &self,
    ) -> Result<(), event_bus_core::EventBusError> {
        let handler: Arc<dyn EventHandler> = Arc::new(LinkMLValidationRequestHandlerImpl {
            event_bus: self.event_bus.clone(),
            linkml_service: self.linkml_service.clone(),
        });

        self.event_bus
            .subscribe("ontology.linkml.validation.request".to_string(), handler)
            .await?;

        Ok(())
    }
}

/// Internal handler implementation
struct LinkMLValidationRequestHandlerImpl {
    /// Event bus for publishing validation results
    event_bus: Arc<dyn EventBusService<Error = event_bus_core::EventBusError>>,
    /// Reference to the LinkML validation service
    linkml_service: Arc<dyn LinkMLValidationService>,
}

#[async_trait]
impl EventHandler for LinkMLValidationRequestHandlerImpl {
    async fn handle(&self, event: Arc<dyn Event>) -> Result<(), event_bus_core::EventBusError> {
        // Downcast to LinkMLValidationRequest
        if let Some(request) = event.as_any().downcast_ref::<LinkMLValidationRequest>() {
            let ontology_id = request.ontology.id.clone();
            let correlation_id = request.correlation_id.clone();

            // Perform validation
            let (schema_valid, errors) = match self.linkml_service.validate_linkml_schema(ontology_id.clone()).await {
                Ok((valid, errs)) => (valid, errs),
                Err(e) => (false, vec![format!("Validation error: {}", e)]),
            };

            // Create and publish validation result
            let result = if schema_valid && errors.is_empty() {
                LinkMLValidationResult::success(
                    ontology_id.clone(),
                    correlation_id.clone(),
                    "linkml-service".to_string(),
                    Utc::now(),
                )
            } else {
                LinkMLValidationResult::failure(
                    ontology_id.clone(),
                    errors,
                    correlation_id.clone(),
                    "linkml-service".to_string(),
                    Utc::now(),
                )
            };

            // Publish the result back to the event bus
            let topic = Topic::new("ontology.linkml.validation.result");
            let _ = self.event_bus.publish(topic, Box::new(result)).await;
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "linkml-validation-request-handler"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use event_bus_core::{EventId, SubscriptionId};

    struct MockLinkMLService;

    #[async_trait]
    impl LinkMLValidationService for MockLinkMLService {
        async fn validate_linkml_schema(
            &self,
            _ontology_id: String,
        ) -> Result<(bool, Vec<String>), String> {
            Ok((true, vec![]))
        }
    }

    #[test]
    fn test_handler_creation() {
        // This test verifies the handler can be created without panicking
        let _handler = LinkMLValidationRequestHandler::new(
            Arc::new(MockEventBus),
            Arc::new(MockLinkMLService),
        );
    }

    struct MockEventBus;

    #[async_trait]
    impl EventBusService for MockEventBus {
        type Error = event_bus_core::EventBusError;

        async fn publish(&self, _topic: Topic, _event: Box<dyn Event>) -> Result<EventId, Self::Error> {
            Ok(EventId::new())
        }

        async fn subscribe(
            &self,
            _topic_pattern: String,
            _handler: Arc<dyn EventHandler>,
        ) -> Result<SubscriptionId, Self::Error> {
            Ok(SubscriptionId::new())
        }

        async fn unsubscribe(&self, _subscription_id: SubscriptionId) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn subscription_count(&self) -> Result<usize, Self::Error> {
            Ok(0)
        }

        async fn queued_event_count(&self) -> Result<usize, Self::Error> {
            Ok(0)
        }

        async fn clear_subscriptions(&self) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn health_check(&self) -> Result<bool, Self::Error> {
            Ok(true)
        }
    }
}
