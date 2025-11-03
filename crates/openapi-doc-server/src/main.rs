mod config;
mod frontend;
mod frontends;

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Json},
    routing::get,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use openapi_common::spec_utils;
use serde::{Deserialize, Serialize};

use frontend::{ApiInfo, DocFrontend};

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

/// Frontend manager that holds configured frontend instances
#[derive(Clone)]
struct FrontendManager {
    frontends: HashMap<String, Arc<dyn DocFrontend>>,
    default_frontend: Option<String>,
}

impl FrontendManager {
    fn from_config(config: &config::FrontendConfig) -> Self {
        use frontend::FrontendType;
        let mut frontends = HashMap::new();

        // Create frontend instances with their configurations
        for frontend_name in &config.enabled_frontends {
            if let Some(frontend_type) = FrontendType::from_str(frontend_name) {
                let options = config.get_options(frontend_name);
                if let Some(frontend) = frontend_type.create_frontend(options) {
                    frontends.insert(frontend_name.clone(), Arc::from(frontend));
                    tracing::info!("Enabled frontend: {} (with custom config)", frontend_name);
                } else {
                    tracing::warn!(
                        "Frontend '{}' is enabled in config but not compiled (missing feature?)",
                        frontend_name
                    );
                }
            }
        }

        // If no frontends were enabled, try to enable at least one compiled frontend
        if frontends.is_empty() {
            #[cfg(feature = "scalar")]
            {
                if let Some(frontend_type) = FrontendType::from_str("scalar") {
                    if let Some(frontend) = frontend_type.create_frontend(None) {
                        frontends.insert("scalar".to_string(), Arc::from(frontend));
                        tracing::info!("Auto-enabled scalar frontend (default)");
                    }
                }
            }
            #[cfg(all(not(feature = "scalar"), feature = "redoc"))]
            {
                if let Some(frontend_type) = FrontendType::from_str("redoc") {
                    if let Some(frontend) = frontend_type.create_frontend(None) {
                        frontends.insert("redoc".to_string(), Arc::from(frontend));
                        tracing::info!("Auto-enabled redoc frontend (default)");
                    }
                }
            }
        }

        // Determine default frontend
        let default = if let Some(ref default_name) = config.default_frontend {
            // Check if default is enabled
            if frontends.contains_key(default_name) {
                Some(default_name.clone())
            } else {
                // Fall back to first enabled frontend
                frontends.keys().next().cloned()
            }
        } else {
            // No explicit default, use first enabled frontend
            frontends.keys().next().cloned()
        };

        if let Some(ref default_name) = default {
            tracing::info!("Default frontend: {}", default_name);
        }

        Self {
            frontends,
            default_frontend: default,
        }
    }

    fn get_frontend(&self, name: &str) -> Option<Arc<dyn DocFrontend>> {
        self.frontends.get(name).cloned()
    }

    fn get_default_frontend(&self) -> Option<Arc<dyn DocFrontend>> {
        self.default_frontend
            .as_ref()
            .and_then(|name| self.get_frontend(name))
    }
}

#[derive(Clone)]
struct AppState {
    cache_dir: PathBuf,
    discovery_path: PathBuf,
    frontend_manager: FrontendManager,
}

// Default values for cache directory and discovery path
const DEFAULT_CACHE_DIR: &str = "/tmp/openapi-cache";
const DEFAULT_DISCOVERY_PATH: &str = "/etc/config/discovery.json";

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            _ => '_',
        })
        .collect()
}

fn get_spec_file_path(cache_dir: &StdPath, api_name: &str) -> PathBuf {
    let sanitized = sanitize_filename(api_name);
    cache_dir.join(format!("{}.json", sanitized))
}

fn get_metadata_file_path(cache_dir: &StdPath, api_name: &str) -> PathBuf {
    let sanitized = sanitize_filename(api_name);
    cache_dir.join(format!("{}.meta.json", sanitized))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Get cache directory from environment or use default
    let cache_dir = std::env::var("CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_CACHE_DIR));

    // Get discovery path from environment or use default
    let discovery_path = std::env::var("DISCOVERY_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_DISCOVERY_PATH));

    // Ensure cache directory exists
    fs::create_dir_all(&cache_dir)?;
    tracing::info!("Using cache directory: {:?}", cache_dir);
    tracing::info!("Using discovery path: {:?}", discovery_path);

    // Load frontend configuration
    let frontend_config = config::FrontendConfig::from_env();
    let frontend_manager = FrontendManager::from_config(&frontend_config);

    // Create application state
    let state = AppState {
        cache_dir: cache_dir.clone(),
        discovery_path: discovery_path.clone(),
        frontend_manager,
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

    // Build the application with routes
    let mut app = Router::new()
        .route("/", get(handle_default))
        .route("/api/{api_name}", get(handle_api_request))
        .route("/specs/{api_name}", get(handle_spec_request))
        .route("/health", get(handle_health));

    // Add frontend-specific routes
    if state.frontend_manager.get_frontend("scalar").is_some() {
        app = app.route("/scalar", get(handle_scalar));
    }
    
    if state.frontend_manager.get_frontend("redoc").is_some() {
        app = app.route("/redoc", get(handle_redoc));
    }

    let app = app
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

async fn handle_default(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    match state.frontend_manager.get_default_frontend() {
        Some(frontend) => {
            generate_frontend_html(frontend, &state.cache_dir).await
        }
        None => {
            tracing::error!("No default frontend configured");
            render_error_template().await
        }
    }
}

async fn render_error_template() -> Result<Html<String>, StatusCode> {
    use askama::Template;
    
    #[derive(askama::Template)]
    #[template(path = "error.html")]
    struct ErrorTemplate;
    
    let template = ErrorTemplate;
    template.render()
        .map(Html)
        .map_err(|e| {
            tracing::error!("Failed to render error template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn handle_scalar(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    match state.frontend_manager.get_frontend("scalar") {
        Some(frontend) => generate_frontend_html(frontend, &state.cache_dir).await,
        None => {
            tracing::warn!("Scalar frontend not available");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn handle_redoc(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    match state.frontend_manager.get_frontend("redoc") {
        Some(frontend) => generate_frontend_html(frontend, &state.cache_dir).await,
        None => {
            tracing::warn!("Redoc frontend not available");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn generate_frontend_html(
    frontend: Arc<dyn DocFrontend>,
    cache_dir: &PathBuf,
) -> Result<Html<String>, StatusCode> {
    // Load all API metadata from cache directory
    let apis = load_apis_from_cache(cache_dir).await;

    tracing::info!("Found {} APIs for frontend", apis.len());

    // Convert to ApiInfo for frontend
    let api_infos: Vec<ApiInfo> = apis
        .iter()
        .enumerate()
        .map(|(i, api)| ApiInfo {
            name: api.name.clone(),
            slug: format!("api-{}", i),
            spec_url: format!("/specs/{}", urlencoding::encode(&api.name)),
            description: api.description.clone(),
        })
        .collect();

    let html = frontend.generate_html(&api_infos);
    Ok(Html(html))
}

async fn handle_api_request(
    Path(api_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // URL decode the API name
    let decoded_name = urlencoding::decode(&api_name).unwrap_or_else(|_| api_name.clone().into());
    let decoded_name_str = decoded_name.as_ref();

    tracing::info!(
        "Looking for API: '{}' (decoded: '{}')",
        api_name,
        decoded_name_str
    );

    // Load spec from file cache
    let spec_path = get_spec_file_path(&state.cache_dir, decoded_name_str);
    
    match fs::read_to_string(&spec_path) {
        Ok(spec_content) => {
            tracing::info!("Serving cached OpenAPI spec for API: {}", decoded_name);
            match spec_utils::parse_spec_to_json(&spec_content) {
                Ok(spec) => Ok(Json(spec)),
                Err(e) => {
                    tracing::warn!("Failed to parse spec for {}: {}", decoded_name, e);
                    Ok(Json(serde_json::json!({
                        "error": "Failed to parse API spec"
                    })))
                }
            }
        }
        Err(e) => {
            tracing::warn!("API spec not found: {} (error: {})", decoded_name, e);
            Ok(Json(serde_json::json!({
                "error": "API not found"
            })))
        }
    }
}

async fn handle_spec_request(
    Path(api_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // This is the same as handle_api_request, but provides a cleaner endpoint for specs
    handle_api_request(Path(api_name), State(state)).await
}

async fn handle_health() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy"
    })))
}

async fn load_apis_from_cache(cache_dir: &StdPath) -> Vec<ServerApiDocEntry> {
    let mut apis = Vec::new();

    if let Ok(entries) = fs::read_dir(cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.ends_with(".meta.json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        match serde_json::from_str::<ServerApiDocEntry>(&content) {
                            Ok(api) => {
                                tracing::debug!("Loaded API from cache: {}", api.name);
                                apis.push(api);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse metadata file {:?}: {}", path, e);
                            }
                        }
                    } else {
                        tracing::warn!("Failed to read metadata file: {:?}", path);
                    }
                }
            }
        }
    } else {
        tracing::warn!("Failed to read cache directory: {:?}", cache_dir);
    }

    tracing::info!("Loaded {} APIs from cache directory", apis.len());
    apis
}

async fn refresh_api_cache(
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Read the discovery.json from the configured path
    match fs::read_to_string(&state.discovery_path) {
        Ok(discovery_json) => {
            let discovery_config: ServerDiscoveryConfig = serde_json::from_str(&discovery_json)?;

            // Clear old cache files (optional - you might want to keep them)
            // For now, we'll just update/add new ones

            for mut api in discovery_config.apis {
                // Fetch the actual OpenAPI spec from the service URL
                match fetch_openapi_spec(&api.url).await {
                    Ok(spec) => {
                        tracing::info!("Successfully fetched OpenAPI spec for API: {}", api.name);
                        
                        // Save spec to file
                        let spec_path = get_spec_file_path(&state.cache_dir, &api.name);
                        fs::write(&spec_path, &spec)?;
                        
                        // Update API metadata
                        api.available = true;
                        api.spec = spec; // Keep spec in metadata for reference, but it's also in the file
                        
                        // Save metadata to file
                        let metadata_path = get_metadata_file_path(&state.cache_dir, &api.name);
                        let api_json = serde_json::to_string(&api)?;
                        fs::write(&metadata_path, api_json)?;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to fetch OpenAPI spec for API {}: {}", api.name, e);
                        
                        // Store a dummy spec for failed APIs
                        let default_spec = serde_json::json!({
                            "openapi": "3.0.0",
                            "info": {
                                "title": api.name,
                                "version": "1.0.0",
                                "description": "API documentation not available"
                            },
                            "paths": {}
                        })
                        .to_string();
                        
                        // Save dummy spec to file
                        let spec_path = get_spec_file_path(&state.cache_dir, &api.name);
                        fs::write(&spec_path, &default_spec)?;
                        
                        api.available = false;
                        api.spec = default_spec;
                        
                        // Save metadata to file
                        let metadata_path = get_metadata_file_path(&state.cache_dir, &api.name);
                        let api_json = serde_json::to_string(&api)?;
                        fs::write(&metadata_path, api_json)?;
                    }
                }
            }

            // Count cached APIs
            let apis = load_apis_from_cache(&state.cache_dir).await;
            tracing::info!("Refreshed API cache with {} APIs", apis.len());
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
