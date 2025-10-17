use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Standard Kubernetes annotations for API documentation
pub const API_DOC_ENABLED_ANNOTATION: &str = "api-doc.io/enabled";
pub const API_DOC_PATH_ANNOTATION: &str = "api-doc.io/path";
pub const API_DOC_NAME_ANNOTATION: &str = "api-doc.io/name";
pub const API_DOC_DESCRIPTION_ANNOTATION: &str = "api-doc.io/description";

/// Default values
pub const DEFAULT_API_DOC_PATH: &str = "/swagger/openapi.yml";

/// Environment variables
pub const WATCH_NAMESPACES_ENV: &str = "WATCH_NAMESPACES";
pub const DISCOVERY_NAMESPACE_ENV: &str = "DISCOVERY_NAMESPACE";
pub const DISCOVERY_CONFIGMAP_ENV: &str = "DISCOVERY_CONFIGMAP";

/// Represents an API documentation entry
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiDocEntry {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub service_name: String,
    pub url: String,
    pub description: Option<String>,
    pub last_updated: DateTime<Utc>,
    pub available: bool,
    pub spec: String, // The actual OpenAPI spec content
}

/// Configuration for API discovery
#[derive(Serialize, Deserialize, Debug)]
pub struct DiscoveryConfig {
    pub apis: Vec<ApiDocEntry>,
    pub last_updated: DateTime<Utc>,
}

/// Utility functions for working with OpenAPI specs
pub mod spec_utils {
    use serde_json;

    /// Creates a default OpenAPI spec for unavailable APIs
    pub fn create_default_spec(name: &str, description: &str) -> String {
        serde_json::json!({
            "openapi": "3.0.0",
            "info": {
                "title": name,
                "version": "1.0.0",
                "description": description
            },
            "paths": {}
        }).to_string()
    }

    /// Parses OpenAPI spec content (JSON or YAML) and returns JSON
    pub fn parse_spec_to_json(spec_content: &str) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        if spec_content.trim().starts_with('{') {
            // Already JSON
            Ok(serde_json::from_str(spec_content)?)
        } else {
            // YAML content, parse it
            Ok(serde_yaml::from_str(spec_content)?)
        }
    }
}

/// Utility functions for namespace handling
pub mod namespace_utils {
    use std::env;

    /// Parses the WATCH_NAMESPACES environment variable
    /// Returns:
    /// - Some(namespaces) if specific namespaces are specified
    /// - None if "all" is specified (watch all namespaces)
    /// - Some(vec!["current"]) if empty or not set (watch current namespace)
    pub fn parse_watch_namespaces() -> Option<Vec<String>> {
        match env::var(super::WATCH_NAMESPACES_ENV) {
            Ok(value) if value.trim().is_empty() => Some(vec!["current".to_string()]),
            Ok(value) if value.trim().to_lowercase() == "all" => None,
            Ok(value) => Some(
                value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            ),
            Err(_) => Some(vec!["current".to_string()]),
        }
    }
}
