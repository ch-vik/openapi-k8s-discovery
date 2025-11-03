use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main frontend configuration
/// 
/// Configuration is loaded from environment variables:
/// - `ENABLED_FRONTENDS`: Comma-separated list of frontends to enable (e.g., "scalar,redoc")
/// - `DEFAULT_FRONTEND`: Default frontend to show at `/` (e.g., "scalar" or "redoc")
/// - `CACHE_DIR`: Directory for caching API specs (default: "/tmp/openapi-cache")
/// - `DISCOVERY_PATH`: Path to discovery.json file (default: "/etc/config/discovery.json")
/// 
/// Frontend-specific options use prefixes:
/// - Scalar: `SCALAR_*`
/// - Redoc: `REDOC_*`
#[derive(Debug, Clone)]
pub struct FrontendConfig {
    pub enabled_frontends: Vec<String>,
    pub default_frontend: Option<String>,
    pub frontend_options: HashMap<String, FrontendOptions>,
}

/// Options for specific frontends
#[derive(Debug, Clone)]
pub enum FrontendOptions {
    #[cfg(feature = "scalar")]
    Scalar(ScalarConfig),
    #[cfg(feature = "redoc")]
    Redoc(RedocConfig),
}

/// Configuration for Scalar frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "scalar")]
pub struct ScalarConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_layout")]
    pub layout: String,
    #[serde(default = "default_false")]
    pub dark_mode: bool,
    #[serde(default = "default_true")]
    pub show_sidebar: bool,
    #[serde(default = "default_true")]
    pub expand_all_responses: bool,
    #[serde(default = "default_false")]
    pub expand_all_model_sections: bool,
    #[serde(default = "default_false")]
    pub hide_download_button: bool,
}

#[cfg(feature = "scalar")]
impl Default for ScalarConfig {
    fn default() -> Self {
        Self {
            theme: "purple".to_string(),
            layout: "modern".to_string(),
            dark_mode: false,
            show_sidebar: true,
            expand_all_responses: true,
            expand_all_model_sections: false,
            hide_download_button: false,
        }
    }
}

/// Configuration for Redoc frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "redoc")]
pub struct RedocConfig {
    #[serde(default = "default_expand_responses")]
    pub expand_responses: String,
    #[serde(default = "default_true")]
    pub required_props_first: bool,
    #[serde(default = "default_api_selector")]
    pub show_api_selector: bool,
}

#[cfg(feature = "redoc")]
impl Default for RedocConfig {
    fn default() -> Self {
        Self {
            expand_responses: "200,201,400,401,403,404".to_string(),
            required_props_first: true,
            show_api_selector: true,
        }
    }
}

// Default value helpers
fn default_theme() -> String {
    "purple".to_string()
}

fn default_layout() -> String {
    "modern".to_string()
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

#[allow(dead_code)]
fn default_expand_responses() -> String {
    "200,201,400,401,403,404".to_string()
}

#[allow(dead_code)]
fn default_api_selector() -> bool {
    true
}

impl FrontendConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        use std::env;

        // Parse enabled frontends
        let enabled_list = env::var("ENABLED_FRONTENDS")
            .unwrap_or_else(|_| "scalar".to_string())
            .to_lowercase();

        let enabled_frontends: Vec<String> = enabled_list
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Get default frontend
        let default_frontend = env::var("DEFAULT_FRONTEND")
            .ok()
            .map(|s| s.to_lowercase());

        // Build frontend options map
        let mut frontend_options = HashMap::new();

        // Load Scalar config
        #[cfg(feature = "scalar")]
        if enabled_frontends.contains(&"scalar".to_string()) {
            let scalar_config = ScalarConfig::from_env();
            frontend_options.insert("scalar".to_string(), FrontendOptions::Scalar(scalar_config));
        }

        // Load Redoc config
        #[cfg(feature = "redoc")]
        if enabled_frontends.contains(&"redoc".to_string()) {
            let redoc_config = RedocConfig::from_env();
            frontend_options.insert("redoc".to_string(), FrontendOptions::Redoc(redoc_config));
        }

        Self {
            enabled_frontends,
            default_frontend,
            frontend_options,
        }
    }

    /// Get options for a specific frontend
    pub fn get_options(&self, frontend_name: &str) -> Option<&FrontendOptions> {
        self.frontend_options.get(frontend_name)
    }
}

#[cfg(feature = "scalar")]
impl ScalarConfig {
    pub fn from_env() -> Self {
        use std::env;

        let mut config = Self::default();

        if let Ok(theme) = env::var("SCALAR_THEME") {
            config.theme = theme;
        }
        if let Ok(layout) = env::var("SCALAR_LAYOUT") {
            config.layout = layout;
        }
        if let Ok(dark_mode) = env::var("SCALAR_DARK_MODE") {
            config.dark_mode = dark_mode.parse().unwrap_or(false);
        }
        if let Ok(show_sidebar) = env::var("SCALAR_SHOW_SIDEBAR") {
            config.show_sidebar = show_sidebar.parse().unwrap_or(true);
        }
        if let Ok(expand_responses) = env::var("SCALAR_EXPAND_ALL_RESPONSES") {
            config.expand_all_responses = expand_responses.parse().unwrap_or(true);
        }
        if let Ok(expand_models) = env::var("SCALAR_EXPAND_ALL_MODEL_SECTIONS") {
            config.expand_all_model_sections = expand_models.parse().unwrap_or(false);
        }
        if let Ok(hide_download) = env::var("SCALAR_HIDE_DOWNLOAD_BUTTON") {
            config.hide_download_button = hide_download.parse().unwrap_or(false);
        }

        config
    }
}

#[cfg(feature = "redoc")]
impl RedocConfig {
    pub fn from_env() -> Self {
        use std::env;

        let mut config = Self::default();

        if let Ok(expand_responses) = env::var("REDOC_EXPAND_RESPONSES") {
            config.expand_responses = expand_responses;
        }
        if let Ok(required_props) = env::var("REDOC_REQUIRED_PROPS_FIRST") {
            config.required_props_first = required_props.parse().unwrap_or(true);
        }
        if let Ok(show_selector) = env::var("REDOC_SHOW_API_SELECTOR") {
            config.show_api_selector = show_selector.parse().unwrap_or(true);
        }

        config
    }
}

