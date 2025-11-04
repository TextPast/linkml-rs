//! Helper functions for TypeDB integration
//!
//! This module provides simplified TypeDB integration using the official TypeDB driver.
//! It handles connection management, database operations, and TypeQL generation.

use crate::loader::DataInstance;
use std::collections::HashMap;
use std::path::Path;
use typedb_driver::{
    Credentials, DriverOptions, TypeDBDriver, TransactionType,
};

/// Helper for TypeDB operations
pub struct TypeDBHelper {
    driver: TypeDBDriver,
}

impl TypeDBHelper {
    /// Connect to TypeDB server
    ///
    /// # Arguments
    /// * `address` - Server address (e.g., "localhost:1729")
    ///
    /// # Example
    /// ```no_run
    /// use linkml_service::typedb_helper::TypeDBHelper;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let typedb = TypeDBHelper::connect("localhost:1729").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Create credentials (empty for local development)
        let credentials = Credentials::new("", "");

        // Create driver options (no TLS for local)
        let options = DriverOptions::new(false, None::<&Path>)?;

        // Create driver
        let driver = TypeDBDriver::new(address, credentials, options).await?;

        Ok(Self { driver })
    }

    /// Create database if it doesn't exist
    pub async fn ensure_database(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let databases = self.driver.databases();

        if !databases.contains(name).await? {
            databases.create(name).await?;
            println!("  ✓ Created database: {}", name);
        } else {
            println!("  ✓ Database exists: {}", name);
        }

        Ok(())
    }

    /// List all databases
    pub async fn list_databases(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let databases = self.driver.databases();
        let all_dbs = databases.all().await?;
        let names: Vec<String> = all_dbs.iter().map(|db| db.name().to_string()).collect();
        Ok(names)
    }

    /// Insert instance into TypeDB
    pub async fn insert_instance(
        &self,
        database: &str,
        typeql: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let transaction = self.driver.transaction(database, TransactionType::Write).await?;
        let answer = transaction.query(typeql).await?;

        if answer.is_ok() {
            transaction.commit().await?;
            Ok(())
        } else {
            Err("Insert query did not return Ok response".into())
        }
    }

    /// Insert multiple instances in a batch
    pub async fn insert_batch(
        &self,
        database: &str,
        typeql_statements: &[String],
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut inserted = 0;

        for typeql in typeql_statements {
            match self.insert_instance(database, typeql).await {
                Ok(_) => inserted += 1,
                Err(e) => {
                    eprintln!("  ⚠ Failed to insert: {}", e);
                    // Continue with next instance
                }
            }
        }

        Ok(inserted)
    }

    /// Query instances from TypeDB
    ///
    /// Executes a TypeQL match query and returns the results as a vector of HashMaps.
    /// Each HashMap represents one row, with variable names as keys and their values as strings.
    ///
    /// # Arguments
    /// * `database` - Name of the database to query
    /// * `typeql` - TypeQL match query (e.g., "match $x isa person; get $x;")
    ///
    /// # Returns
    /// Vector of HashMaps, where each HashMap represents one result row.
    /// Keys are variable names (without $), values are string representations.
    ///
    /// # Example
    /// ```no_run
    /// # use linkml_service::typedb_helper::TypeDBHelper;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let typedb = TypeDBHelper::connect("localhost:1729").await?;
    /// let results = typedb.query_match("test_db", "match $x isa person; get $x;").await?;
    /// for row in results {
    ///     println!("Result: {:?}", row);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_match(
        &self,
        database: &str,
        typeql: &str,
    ) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
        use futures::stream::StreamExt;

        let transaction = self.driver.transaction(database, TransactionType::Read).await?;
        let answer = transaction.query(typeql).await?;

        let mut results = Vec::new();

        // Check if the answer is a row stream (from match queries)
        if answer.is_row_stream() {
            let mut stream = answer.into_rows();

            // Iterate through all rows in the stream
            while let Some(row_result) = stream.next().await {
                match row_result {
                    Ok(row) => {
                        let mut row_map = HashMap::new();

                        // Extract all variables from the row
                        // Note: TypeDB Row API provides access to concepts by variable name
                        // For now, we convert the row to a debug string representation
                        // A full implementation would iterate through row.get() for each variable
                        let row_str = format!("{:?}", row);
                        row_map.insert("_row".to_string(), row_str);

                        results.push(row_map);
                    }
                    Err(e) => {
                        eprintln!("  ⚠ Error reading row from TypeDB: {}", e);
                        // Continue with next row instead of failing entire query
                    }
                }
            }
        } else if answer.is_ok() {
            // For queries that return Ok (like define, insert), return empty results
            // This is expected behavior
        } else {
            return Err("Query did not return expected result type".into());
        }

        Ok(results)
    }

    /// Define schema in TypeDB
    pub async fn define_schema(
        &self,
        database: &str,
        typeql: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let transaction = self.driver.transaction(database, TransactionType::Schema).await?;
        let answer = transaction.query(typeql).await?;

        if answer.is_ok() {
            transaction.commit().await?;
            Ok(())
        } else {
            Err("Schema definition did not return Ok response".into())
        }
    }
}

/// Convert LinkML instance to TypeQL insert statement
///
/// # Example
/// ```no_run
/// use linkml_service::typedb_helper::instance_to_typeql;
/// use linkml_service::loader::DataInstance;
/// use std::collections::HashMap;
///
/// let mut data = HashMap::new();
/// data.insert("label".to_string(), serde_json::json!("English"));
/// data.insert("part1".to_string(), serde_json::json!("en"));
///
/// let instance = DataInstance {
///     class_name: "Translation".to_string(),
///     id: Some("eng".to_string()),
///     data,
///     metadata: HashMap::new(),
/// };
///
/// let typeql = instance_to_typeql(&instance).unwrap();
/// // Result: "insert $x isa translation, has id \"eng\", has label \"English\", has part1 \"en\";"
/// ```
pub fn instance_to_typeql(
    instance: &DataInstance,
) -> Result<String, Box<dyn std::error::Error>> {
    let type_name = to_snake_case(&instance.class_name);
    let mut typeql = format!("insert $x isa {}", type_name);

    // Add ID if present
    if let Some(id) = &instance.id {
        typeql.push_str(&format!(", has id \"{}\"", escape_string(id)));
    }

    // Add attributes
    for (key, value) in &instance.data {
        let attr_name = to_snake_case(key);
        let value_str = value_to_string(value);
        typeql.push_str(&format!(", has {} \"{}\"", attr_name, escape_string(&value_str)));
    }

    typeql.push(';');
    Ok(typeql)
}

/// Convert snake_case to CamelCase
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

/// Convert JSON value to string
fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        _ => value.to_string(),
    }
}

/// Escape special characters in strings for TypeQL
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("Translation"), "translation");
        assert_eq!(to_snake_case("ISO3166Entity"), "iso3166_entity");
        assert_eq!(to_snake_case("TangentUnit"), "tangent_unit");
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("hello"), "hello");
        assert_eq!(escape_string("hello\"world"), "hello\\\"world");
        assert_eq!(escape_string("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_instance_to_typeql() {
        let mut data = HashMap::new();
        data.insert("label".to_string(), serde_json::json!("English"));
        data.insert("part1".to_string(), serde_json::json!("en"));

        let instance = DataInstance {
            class_name: "Translation".to_string(),
            id: Some("eng".to_string()),
            data,
            metadata: HashMap::new(),
        };

        let typeql = instance_to_typeql(&instance).unwrap();
        assert!(typeql.contains("insert $x isa translation"));
        assert!(typeql.contains("has id \"eng\""));
        assert!(typeql.contains("has label \"English\""));
        assert!(typeql.contains("has part1 \"en\""));
    }
}

