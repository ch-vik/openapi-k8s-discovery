mod error;

use chrono::Utc;
use futures::StreamExt;
use k8s_openapi::api::core::v1::{ConfigMap, Service};
use kube::{
    Client, ResourceExt,
    api::{Api, Patch, PatchParams},
    runtime::{controller::{Action, Controller}, watcher::Config},
};
use std::{collections::BTreeMap, env, sync::Arc, time::Duration};
use tracing::{error, info, warn};

use error::AppError;
use openapi_common::{
    ApiDocEntry, DiscoveryConfig,
    API_DOC_ENABLED_ANNOTATION, API_DOC_PATH_ANNOTATION, API_DOC_NAME_ANNOTATION, API_DOC_DESCRIPTION_ANNOTATION,
    DEFAULT_API_DOC_PATH, WATCH_NAMESPACES_ENV, DISCOVERY_NAMESPACE_ENV, DISCOVERY_CONFIGMAP_ENV,
    spec_utils, namespace_utils
};

#[derive(Clone)]
struct ContextData {
    discovery: Api<ConfigMap>,
    http_client: reqwest::Client,
    watch_namespaces: Vec<String>,
    discovery_namespace: String,
    discovery_configmap: String,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting OpenAPI K8s Operator");

    let client = Client::try_default().await.map_err(|e| {
        error!("Failed to create Kubernetes client: {}", e);
        e
    })?;

    let watch_namespaces = match namespace_utils::parse_watch_namespaces() {
        Some(namespaces) => {
            if namespaces.contains(&"current".to_string()) {
                // Watch current namespace only
                let current_namespace = env::var("POD_NAMESPACE").unwrap_or_else(|_| "default".to_string());
                info!("Watching current namespace: {}", current_namespace);
                vec![current_namespace]
            } else {
                info!("Watching specified namespaces: {:?}", namespaces);
                namespaces
            }
        }
        None => {
            info!("WATCH_NAMESPACES=all, watching all namespaces");
            vec!["all".to_string()]
        }
    };
    
    let discovery_namespace =
        env::var(DISCOVERY_NAMESPACE_ENV).unwrap_or_else(|_| "default".to_string());
    let discovery_configmap =
        env::var(DISCOVERY_CONFIGMAP_ENV).unwrap_or_else(|_| "openapi-discovery".to_string());

    // Validate discovery namespace and configmap names
    if discovery_namespace.is_empty() {
        error!("DISCOVERY_NAMESPACE cannot be empty");
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "DISCOVERY_NAMESPACE cannot be empty",
        )));
    }
    
    if discovery_configmap.is_empty() {
        error!("DISCOVERY_CONFIGMAP cannot be empty");
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "DISCOVERY_CONFIGMAP cannot be empty",
        )));
    }

    // Validate discovery namespace name follows Kubernetes naming rules
    if !discovery_namespace.chars().all(|c| c.is_alphanumeric() || c == '-') {
        error!("Invalid DISCOVERY_NAMESPACE: '{}' contains invalid characters", discovery_namespace);
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid discovery namespace name: {}", discovery_namespace),
        )));
    }
    
    if discovery_namespace.len() > 63 {
        error!("Invalid DISCOVERY_NAMESPACE: '{}' exceeds 63 characters", discovery_namespace);
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Discovery namespace name too long: {}", discovery_namespace),
        )));
    }

    // Validate discovery configmap name follows Kubernetes naming rules
    if !discovery_configmap.chars().all(|c| c.is_alphanumeric() || c == '-') {
        error!("Invalid DISCOVERY_CONFIGMAP: '{}' contains invalid characters", discovery_configmap);
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid discovery configmap name: {}", discovery_configmap),
        )));
    }
    
    if discovery_configmap.len() > 63 {
        error!("Invalid DISCOVERY_CONFIGMAP: '{}' exceeds 63 characters", discovery_configmap);
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Discovery configmap name too long: {}", discovery_configmap),
        )));
    }

    info!("Watching namespaces: {:?}", watch_namespaces);
    info!("Discovery namespace: {}", discovery_namespace);
    info!("Discovery ConfigMap: {}", discovery_configmap);

    let services = if watch_namespaces.is_empty() {
        let current_namespace =
            env::var("POD_NAMESPACE").unwrap_or_else(|_| "default".to_string());
        info!("Watching current namespace: {}", current_namespace);
        Api::namespaced(client.clone(), &current_namespace)
    } else if watch_namespaces.len() == 1 && watch_namespaces[0] == "all" {
        info!("Watching all namespaces");
        Api::all(client.clone())
    } else if watch_namespaces.len() == 1 {
        let namespace = &watch_namespaces[0];
        info!("Watching single namespace: {}", namespace);
        Api::namespaced(client.clone(), namespace)
    } else {
        info!("Watching multiple namespaces: {:?}", watch_namespaces);
        Api::all(client.clone())
    };

    let discovery: Api<ConfigMap> =
        Api::namespaced(client.clone(), &discovery_namespace);

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let context = Arc::new(ContextData {
        discovery,
        http_client,
        watch_namespaces,
        discovery_namespace,
        discovery_configmap,
    });

    // Initialize the ConfigMap if it doesn't exist
    if let Err(e) = initialize_discovery_configmap(&context).await {
        error!("Failed to initialize discovery ConfigMap: {}", e);
        return Err(e);
    }

    let controller = Controller::new(services, Config::default().any_semantic())
        .run(reconcile, error_policy, context)
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("Reconciled service: {:?}", o),
                Err(e) => error!("Reconcile failed: {:?}", e),
            }
        });

    info!("Controller started, watching for services with API documentation annotations");
    controller.await;

    Ok(())
}

fn parse_watch_namespaces() -> Result<Vec<String>, AppError> {
    let namespaces_str = env::var(WATCH_NAMESPACES_ENV).unwrap_or_default();

    if namespaces_str.is_empty() {
        info!("No WATCH_NAMESPACES specified, watching current namespace only");
        return Ok(Vec::new());
    }

    if namespaces_str == "all" {
        info!("WATCH_NAMESPACES=all, watching all namespaces");
        return Ok(vec!["all".to_string()]);
    }

    let namespaces: Vec<String> = namespaces_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if namespaces.is_empty() {
        info!("Empty WATCH_NAMESPACES, watching current namespace only");
        return Ok(Vec::new());
    }

    // Validate namespace names
    for namespace in &namespaces {
        if namespace.is_empty() {
            error!("Invalid WATCH_NAMESPACES: empty namespace specified");
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Empty namespace in WATCH_NAMESPACES",
            )));
        }
        
        // Basic namespace name validation (Kubernetes namespace naming rules)
        if !namespace.chars().all(|c| c.is_alphanumeric() || c == '-') {
            error!("Invalid WATCH_NAMESPACES: namespace '{}' contains invalid characters", namespace);
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid namespace name: {}", namespace),
            )));
        }
        
        if namespace.len() > 63 {
            error!("Invalid WATCH_NAMESPACES: namespace '{}' exceeds 63 characters", namespace);
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Namespace name too long: {}", namespace),
            )));
        }
    }

    info!("Parsed watch namespaces: {:?}", namespaces);
    Ok(namespaces)
}

async fn reconcile(
    service: Arc<Service>,
    ctx: Arc<ContextData>,
) -> Result<Action, AppError> {
    let service_name = service.name_any();
    let namespace = service.namespace().unwrap_or_default();

    if !ctx.watch_namespaces.is_empty()
        && !ctx.watch_namespaces.contains(&"all".to_string())
        && !ctx.watch_namespaces.contains(&namespace)
    {
        info!(
            "Skipping service {} in namespace {} (not in watch list)",
            service_name, namespace
        );
        return Ok(Action::requeue(Duration::from_secs(300)));
    }

    info!(
        "Reconciling service: {} in namespace: {}",
        service_name, namespace
    );

    let annotations = service.annotations();
    let enabled = annotations
        .get(API_DOC_ENABLED_ANNOTATION)
        .map(|v| v == "true")
        .unwrap_or(false);

    if !enabled {
        info!(
            "Service {} does not have API documentation enabled, skipping",
            service_name
        );
        return Ok(Action::requeue(Duration::from_secs(300)));
    }

    let api_path = annotations
        .get(API_DOC_PATH_ANNOTATION)
        .cloned()
        .unwrap_or_else(|| DEFAULT_API_DOC_PATH.to_string());

    let api_name = annotations
        .get(API_DOC_NAME_ANNOTATION)
        .cloned()
        .unwrap_or_else(|| format!("{} API", service_name));

    let description = annotations.get(API_DOC_DESCRIPTION_ANNOTATION).cloned();

    let port = service
        .spec
        .as_ref()
        .and_then(|s| s.ports.as_ref())
        .and_then(|ports| ports.first())
        .map(|p| p.port)
        .unwrap_or(8080);

    let url = format!(
        "http://{}.{}.svc.cluster.local:{}{}",
        service_name, namespace, port, api_path
    );

    let available = check_api_availability(&ctx.http_client, &url).await;

    // Create a deterministic ID based on service name and namespace
    let entry_id = format!("{}-{}", namespace, service_name);
    
    // Fetch the actual OpenAPI spec
    let spec = if available {
        match fetch_openapi_spec(&url).await {
            Ok(spec) => {
                info!("Successfully fetched OpenAPI spec for service: {}", service_name);
                spec
            }
            Err(e) => {
                warn!("Failed to fetch OpenAPI spec for service {}: {}", service_name, e);
                spec_utils::create_default_spec(&api_name, "API documentation not available")
            }
        }
    } else {
        spec_utils::create_default_spec(&api_name, "API documentation not available")
    };

    let entry = ApiDocEntry {
        id: entry_id,
        name: api_name,
        namespace: namespace.clone(),
        service_name: service_name.clone(),
        url,
        description,
        last_updated: Utc::now(),
        available,
        spec,
    };

    update_discovery_configmap(ctx, entry).await?;

    info!(
        "Successfully reconciled service: {} (available: {})",
        service_name, available
    );

    Ok(Action::requeue(Duration::from_secs(300)))
}

async fn check_api_availability(client: &reqwest::Client, url: &str) -> bool {
    match client.get(url).send().await {
        Ok(response) => response.status().is_success(),
        Err(e) => {
            warn!("Failed to check API availability for {}: {}", url, e);
            false
        }
    }
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


async fn update_discovery_configmap(ctx: Arc<ContextData>, entry: ApiDocEntry) -> Result<(), AppError> {
    let configmap_name = &ctx.discovery_configmap;
    let configmap_namespace = &ctx.discovery_namespace;

    let discovery_api: Api<ConfigMap> =
        Api::namespaced(ctx.discovery.clone().into_client(), configmap_namespace);

    // Get existing ConfigMap or create new one
    let existing_configmap = discovery_api.get_opt(configmap_name).await.map_err(|e| {
        error!("Failed to get ConfigMap '{}' in namespace '{}': {}", configmap_name, configmap_namespace, e);
        AppError::Kube(e)
    })?;

    let apis = if let Some(configmap) = existing_configmap {
        if let Some(data) = configmap.data.as_ref() {
            if let Some(discovery_json) = data.get("discovery.json") {
                serde_json::from_str::<DiscoveryConfig>(discovery_json)
                    .map(|config| config.apis)
                    .unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Deduplicate APIs by service name and namespace
    let mut unique_apis: std::collections::HashMap<String, ApiDocEntry> = std::collections::HashMap::new();
    for api in apis {
        let key = format!("{}-{}", api.namespace, api.service_name);
        // Keep the most recent entry (highest last_updated timestamp)
        if let Some(existing) = unique_apis.get(&key) {
            if api.last_updated > existing.last_updated {
                unique_apis.insert(key, api);
            }
        } else {
            unique_apis.insert(key, api);
        }
    }

    // Add or update the current entry
    let key = format!("{}-{}", entry.namespace, entry.service_name);
    unique_apis.insert(key, entry);

    // Convert back to vector
    let apis: Vec<ApiDocEntry> = unique_apis.into_values().collect();

    let discovery_config = DiscoveryConfig {
        apis,
        last_updated: Utc::now(),
    };

    let discovery_json = serde_json::to_string_pretty(&discovery_config).map_err(|e| {
        error!("Failed to serialize discovery config to JSON: {}", e);
        AppError::Serde(e)
    })?;
    
    info!("Serialized discovery config with {} APIs", discovery_config.apis.len());
    

    let configmap = ConfigMap {
        metadata: kube::core::ObjectMeta {
            name: Some(configmap_name.to_string()),
            namespace: Some(configmap_namespace.to_string()),
            labels: Some(BTreeMap::from([
                (
                    "app.kubernetes.io/name".to_string(),
                    "openapi-discovery".to_string(),
                ),
                (
                    "app.kubernetes.io/component".to_string(),
                    "discovery".to_string(),
                ),
            ])),
            ..Default::default()
        },
        data: Some(BTreeMap::from([
            ("discovery.json".to_string(), discovery_json),
        ])),
        ..Default::default()
    };

    // Use apply to create or update the ConfigMap
    let patch_params = PatchParams::apply("openapi-k8s-operator");
    match discovery_api.patch(configmap_name, &patch_params, &Patch::Apply(configmap)).await {
        Ok(_) => {
            info!("Successfully updated ConfigMap '{}' in namespace '{}'", configmap_name, configmap_namespace);
        }
        Err(e) => {
            error!("Failed to update ConfigMap '{}' in namespace '{}': {}", configmap_name, configmap_namespace, e);
            return Err(AppError::Kube(e));
        }
    }

    info!(
        "Updated discovery ConfigMap with {} unique APIs",
        discovery_config.apis.len()
    );
    
    
    Ok(())
}

async fn initialize_discovery_configmap(ctx: &ContextData) -> Result<(), AppError> {
    let configmap_name = &ctx.discovery_configmap;
    let configmap_namespace = &ctx.discovery_namespace;

    let discovery_api: Api<ConfigMap> =
        Api::namespaced(ctx.discovery.clone().into_client(), configmap_namespace);

    // Check if ConfigMap already exists
    match discovery_api.get_opt(configmap_name).await {
        Ok(Some(_)) => {
            info!("Discovery ConfigMap '{}' already exists in namespace '{}'", configmap_name, configmap_namespace);
            return Ok(());
        }
        Ok(None) => {
            info!("Discovery ConfigMap '{}' does not exist, creating it", configmap_name);
        }
        Err(e) => {
            error!("Failed to check if ConfigMap '{}' exists in namespace '{}': {}", configmap_name, configmap_namespace, e);
            return Err(AppError::Kube(e));
        }
    }

    // Create empty discovery config
    let discovery_config = DiscoveryConfig {
        apis: Vec::new(),
        last_updated: Utc::now(),
    };

    let discovery_json = serde_json::to_string_pretty(&discovery_config).map_err(|e| {
        error!("Failed to serialize initial discovery config to JSON: {}", e);
        AppError::Serde(e)
    })?;

    let configmap = ConfigMap {
        metadata: kube::core::ObjectMeta {
            name: Some(configmap_name.to_string()),
            namespace: Some(configmap_namespace.to_string()),
            labels: Some(BTreeMap::from([
                (
                    "app.kubernetes.io/name".to_string(),
                    "openapi-discovery".to_string(),
                ),
                (
                    "app.kubernetes.io/component".to_string(),
                    "discovery".to_string(),
                ),
            ])),
            ..Default::default()
        },
        data: Some(BTreeMap::from([
            ("discovery.json".to_string(), discovery_json),
        ])),
        ..Default::default()
    };

    // Create the ConfigMap
    match discovery_api.create(&Default::default(), &configmap).await {
        Ok(_) => {
            info!("Successfully created initial discovery ConfigMap '{}' in namespace '{}'", configmap_name, configmap_namespace);
        }
        Err(e) => {
            error!("Failed to create discovery ConfigMap '{}' in namespace '{}': {}", configmap_name, configmap_namespace, e);
            return Err(AppError::Kube(e));
        }
    }

    Ok(())
}

fn error_policy(
    service: Arc<Service>,
    err: &AppError,
    _ctx: Arc<ContextData>,
) -> Action {
    error!(
        "Reconcile error for service {}: {}",
        service.name_any(),
        err
    );
    Action::requeue(Duration::from_secs(60))
}
