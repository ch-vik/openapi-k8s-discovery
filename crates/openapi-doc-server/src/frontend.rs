/// Trait for different API documentation frontends
pub trait DocFrontend: Send + Sync {
    /// Generate HTML for the documentation page with multiple APIs
    fn generate_html(&self, apis: &[ApiInfo]) -> String;
    
    /// Generate HTML for empty state (no APIs found)
    fn generate_empty_html(&self) -> String;
}

/// Information about an API for frontend rendering
#[derive(Debug, Clone)]
pub struct ApiInfo {
    pub name: String,
    pub slug: String,
    pub spec_url: String,
    #[allow(dead_code)] // May be used by frontends in the future
    pub description: Option<String>,
}

/// Available frontend types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontendType {
    Scalar,
    Redoc,
}

impl FrontendType {
    /// Create a frontend instance of this type with optional configuration
    pub fn create_frontend(
        &self,
        options: Option<&crate::config::FrontendOptions>,
    ) -> Option<Box<dyn DocFrontend>> {
        match self {
            FrontendType::Scalar => {
                #[cfg(feature = "scalar")]
                {
                    use crate::config::FrontendOptions;
                    let config = match options {
                        Some(FrontendOptions::Scalar(config)) => config.clone(),
                        _ => crate::config::ScalarConfig::default(),
                    };
                    Some(Box::new(crate::frontends::scalar::ScalarFrontend::new(config)))
                }
                #[cfg(not(feature = "scalar"))]
                {
                    None
                }
            }
            FrontendType::Redoc => {
                #[cfg(feature = "redoc")]
                {
                    use crate::config::FrontendOptions;
                    let config = match options {
                        Some(FrontendOptions::Redoc(config)) => config.clone(),
                        _ => crate::config::RedocConfig::default(),
                    };
                    Some(Box::new(crate::frontends::redoc::RedocFrontend::new(config)))
                }
                #[cfg(not(feature = "redoc"))]
                {
                    None
                }
            }
        }
    }

    /// Get string representation
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            FrontendType::Scalar => "scalar",
            FrontendType::Redoc => "redoc",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "scalar" => Some(FrontendType::Scalar),
            "redoc" => Some(FrontendType::Redoc),
            _ => None,
        }
    }
}

