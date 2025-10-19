use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Json},
    routing::get,
};
use scalar_api_reference::scalar_html_default;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use openapi_common::spec_utils;
use serde::{Deserialize, Serialize};

// Server-specific types that handle string serialization for last_updated
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ServerApiDocEntry {
    id: String,
    name: String,
    namespace: String,
    service_name: String,
    url: String,
    description: Option<String>,
    last_updated: String, // String version for server compatibility
    available: bool,
    spec: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ServerDiscoveryConfig {
    apis: Vec<ServerApiDocEntry>,
    last_updated: String, // String version for server compatibility
}

#[derive(Clone)]
struct AppState {
    api_cache: Arc<RwLock<HashMap<String, String>>>, // API name -> OpenAPI spec
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Create application state
    let state = AppState {
        api_cache: Arc::new(RwLock::new(HashMap::new())),
    };

    // Start background task to refresh API cache
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = refresh_api_cache(&state_clone).await {
                tracing::error!("Failed to refresh API cache: {}", e);
            }
        }
    });

    // Build the application
    let app = Router::new()
        .route("/", get(handle_index))
        .route("/api/:api_name", get(handle_api_request))
        .route("/health", get(handle_health))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Starting OpenAPI documentation server on port 8080");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_index(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    tracing::info!("Received request for index page");

    let cache = state.api_cache.read().await;
    let apis: Vec<ServerApiDocEntry> = cache
        .values()
        .filter_map(|json| serde_json::from_str::<ServerApiDocEntry>(json).ok())
        .collect();

    tracing::info!("Found {} APIs for index page", apis.len());

    let html = generate_index_html(&apis);
    Ok(Html(html))
}

async fn handle_api_request(
    Path(api_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let cache = state.api_cache.read().await;

    // URL decode the API name
    let decoded_name = urlencoding::decode(&api_name).unwrap_or_else(|_| api_name.clone().into());
    let decoded_name_str = decoded_name.as_ref();

    tracing::info!(
        "Looking for API: '{}' (decoded: '{}')",
        api_name,
        decoded_name_str
    );
    tracing::info!(
        "Available APIs in cache: {:?}",
        cache.keys().collect::<Vec<_>>()
    );

    // Find the API by name and return its cached OpenAPI spec
    if let Some(api_json) = cache.get(decoded_name_str) {
        tracing::info!("Serving cached OpenAPI spec for API: {}", decoded_name);
        if let Ok(api_entry) = serde_json::from_str::<ServerApiDocEntry>(api_json) {
            let spec =
                spec_utils::parse_spec_to_json(&api_entry.spec).unwrap_or(serde_json::Value::Null);
            Ok(Json(spec))
        } else {
            tracing::warn!("Failed to parse API entry for: {}", decoded_name);
            Ok(Json(serde_json::json!({
                "error": "Failed to parse API entry"
            })))
        }
    } else {
        tracing::warn!("API not found: {}", api_name);
        Ok(Json(serde_json::json!({
            "error": "API not found"
        })))
    }
}

async fn handle_health() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy"
    })))
}

fn generate_index_html(apis: &[ServerApiDocEntry]) -> String {
    // If we have no APIs, show a default message
    if apis.is_empty() {
        let default_spec = serde_json::json!({
            "openapi": "3.0.0",
            "info": {
                "title": "No APIs Found",
                "version": "1.0.0",
                "description": "No APIs are currently available"
            },
            "paths": {}
        });

        let configuration = serde_json::json!([{
            "title": "No APIs Found",
            "content": default_spec.to_string(),
            "theme": "purple",
            "layout": "modern",
            "darkMode": false,
            "showSidebar": true,
            "hideDownloadButton": false
        }]);

        return scalar_html_default(&configuration);
    }

    // Create multiple configurations for Scalar - one for each API
    let mut configurations = Vec::new();

    for (i, api) in apis.iter().enumerate() {
        // Parse the API spec using shared utility
        let api_spec: serde_json::Value =
            spec_utils::parse_spec_to_json(&api.spec).unwrap_or_else(|_| {
                serde_json::from_str(&spec_utils::create_default_spec(
                    &api.name,
                    "API documentation not available",
                ))
                .unwrap_or(serde_json::Value::Null)
            });

        let config = serde_json::json!({
            "title": api.name.clone(),
            "slug": format!("api-{}", i),
            "content": api_spec.to_string(),
            "theme": "purple",
            "layout": "modern",
            "darkMode": false,
            "showSidebar": true,
            "hideDownloadButton": false,
            "default": i == 0  // Make the first API the default
        });

        configurations.push(config);
    }

    // Use Scalar's Rust crate to generate the HTML with multiple configurations
    scalar_html_default(&serde_json::Value::Array(configurations))
}

async fn refresh_api_cache(
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Read the discovery.json from the mounted ConfigMap
    let discovery_path = "/etc/config/discovery.json";

    match fs::read_to_string(discovery_path) {
        Ok(discovery_json) => {
            let discovery_config: ServerDiscoveryConfig = serde_json::from_str(&discovery_json)?;

            let mut cache = state.api_cache.write().await;
            cache.clear();

            for mut api in discovery_config.apis {
                // Fetch the actual OpenAPI spec from the service URL
                match fetch_openapi_spec(&api.url).await {
                    Ok(spec) => {
                        tracing::info!("Successfully fetched OpenAPI spec for API: {}", api.name);
                        // Store the API metadata with the spec
                        api.available = true;
                        api.spec = spec;
                        let api_json = serde_json::to_string(&api)?;
                        cache.insert(api.name.clone(), api_json);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to fetch OpenAPI spec for API {}: {}", api.name, e);
                        // Store a dummy spec for failed APIs
                        api.available = false;
                        api.spec = serde_json::json!({
                            "openapi": "3.0.0",
                            "info": {
                                "title": api.name,
                                "version": "1.0.0",
                                "description": "API documentation not available"
                            },
                            "paths": {}
                        })
                        .to_string();
                        let api_json = serde_json::to_string(&api)?;
                        cache.insert(api.name.clone(), api_json);
                    }
                }
            }

            tracing::info!("Refreshed API cache with {} APIs", cache.len());
        }
        Err(e) => {
            tracing::error!("Failed to read discovery ConfigMap: {}", e);
        }
    }

    Ok(())
}

async fn fetch_openapi_spec(url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    if response.status().is_success() {
        Ok(response.text().await?)
    } else {
        Err(format!("HTTP error: {}", response.status()).into())
    }
}

