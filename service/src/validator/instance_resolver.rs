//! Instance-based validation resolver
//!
//! Automatically detects and configures instance-based validation from schema definitions

use linkml_core::error::Result;
use linkml_core::types::{SchemaDefinition, SlotDefinition};
use std::path::PathBuf;
use std::sync::Arc;

use super::instance_loader::{InstanceConfig, InstanceData, InstanceLoader};

/// Resolves instance files and configures validation
pub struct InstanceResolver {
    /// Base directory for schema files
    schema_base_dir: PathBuf,
    /// Instance loader
    loader: Arc<InstanceLoader>,
    /// Cache of resolved instance data by range class name
    instance_cache: dashmap::DashMap<String, Arc<InstanceData>>,
}

impl InstanceResolver {
    /// Create a new instance resolver
    pub fn new(schema_base_dir: PathBuf, loader: Arc<InstanceLoader>) -> Self {
        Self {
            schema_base_dir,
            loader,
            instance_cache: dashmap::DashMap::new(),
        }
    }

    /// Resolve instance file path from import statement
    ///
    /// RootReal/Textpast convention:
    /// - Schema imports: `txp:path/to/module/schema` → `path/to/module/schema.yaml`
    /// - Instance imports: `txp:path/to/module/instance` → `path/to/module.yaml`
    ///
    /// Example: `txp:place/polity/country/iso_3166_entity/instance`
    ///       → `place/polity/country/iso_3166_entity.yaml`
    fn resolve_instance_path(&self, import: &str) -> Option<PathBuf> {
        // Remove prefix (e.g., "txp:")
        let path_part = if let Some(idx) = import.find(':') {
            &import[idx + 1..]
        } else {
            import
        };

        // Check if this is an instance import (ends with /instance)
        if !path_part.ends_with("/instance") {
            return None;
        }

        // Remove the /instance suffix to get the base path
        let base_path = &path_part[..path_part.len() - "/instance".len()];

        // Try .yaml extension first, then .yml
        let yaml_path = self.schema_base_dir.join(format!("{base_path}.yaml"));
        if yaml_path.exists() {
            return Some(yaml_path);
        }

        let yml_path = self.schema_base_dir.join(format!("{base_path}.yml"));
        if yml_path.exists() {
            return Some(yml_path);
        }

        None
    }

    /// Load instance data for a range class with specified key field
    ///
    /// # Errors
    ///
    /// Returns an error if the instance file cannot be loaded
    pub async fn load_instance_for_range_with_field(
        &self,
        range_class: &str,
        key_field: &str,
        schema: &SchemaDefinition,
    ) -> Result<Option<Arc<InstanceData>>> {
        // Create cache key that includes the field name
        let cache_key = format!("{range_class}::{key_field}");

        // Check cache first
        if let Some(cached) = self.instance_cache.get(&cache_key) {
            return Ok(Some(Arc::clone(&cached)));
        }

        // Find the import that provides this range class
        for import in &schema.imports {
            if let Some(instance_path) = self.resolve_instance_path(import) {
                // Load the instance file with specified key field
                let config = InstanceConfig {
                    key_field: key_field.to_string(),
                    value_field: None,
                    filter: None,
                };

                match self.loader.load_file(&instance_path, &config).await {
                    Ok(instance_data) => {
                        // Cache it
                        self.instance_cache.insert(
                            cache_key,
                            Arc::clone(&instance_data),
                        );
                        return Ok(Some(instance_data));
                    }
                    Err(e) => {
                        // Log but continue - might not be the right file
                        eprintln!("Warning: Failed to load instance file {}: {}",
                            instance_path.display(), e);
                    }
                }
            }
        }

        Ok(None)
    }

    /// Load instance data for a range class (uses 'id' as default key field)
    ///
    /// # Errors
    ///
    /// Returns an error if the instance file cannot be loaded
    pub async fn load_instance_for_range(
        &self,
        range_class: &str,
        schema: &SchemaDefinition,
    ) -> Result<Option<Arc<InstanceData>>> {
        self.load_instance_for_range_with_field(range_class, "id", schema).await
    }

    /// Get valid instance IDs for a slot with range_type: instance
    ///
    /// # Errors
    ///
    /// Returns an error if the instance data cannot be loaded
    pub async fn get_valid_ids_for_slot(
        &self,
        slot: &SlotDefinition,
        schema: &SchemaDefinition,
    ) -> Result<Option<Vec<String>>> {
        // Check if this slot has range_type: instance
        if slot.range_type.as_deref() != Some("instance") {
            return Ok(None);
        }

        // Get the range class
        let range_class = match &slot.range {
            Some(range) => range,
            None => return Ok(None),
        };

        // Get the property to use (default to 'id')
        let property = slot.range_properties
            .first()
            .map(|s| s.as_str())
            .unwrap_or("id");

        // Load instance data for this range with the correct field
        let instance_data = match self.load_instance_for_range_with_field(
            range_class,
            property,
            schema
        ).await? {
            Some(data) => data,
            None => return Ok(None),
        };

        // Extract IDs from instance data
        // The keys in the values map are the instance IDs
        let ids = instance_data.values
            .get(property)
            .cloned()
            .or_else(|| {
                // If property not found, try to get all unique keys
                let all_ids: Vec<String> = instance_data.values
                    .keys()
                    .cloned()
                    .collect();
                if all_ids.is_empty() {
                    None
                } else {
                    Some(all_ids)
                }
            });

        Ok(ids)
    }

    /// Validate a value against instance data
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails
    pub async fn validate_instance_value(
        &self,
        value: &str,
        slot: &SlotDefinition,
        schema: &SchemaDefinition,
    ) -> Result<bool> {
        let valid_ids = match self.get_valid_ids_for_slot(slot, schema).await? {
            Some(ids) => ids,
            None => return Ok(true), // No instance validation needed
        };

        Ok(valid_ids.contains(&value.to_string()))
    }
}

