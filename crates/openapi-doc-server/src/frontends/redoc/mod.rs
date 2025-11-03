use crate::config::RedocConfig;
use crate::frontend::{ApiInfo, DocFrontend};
use askama::Template;

/// Template for Redoc main page with API selector
#[derive(Template)]
#[template(path = "redoc/main.html")]
struct RedocMainTemplate {
    apis: Vec<RedocApiInfo>,
    has_multiple_apis: bool,
    show_api_selector: bool,
    expand_responses: String,
    required_props_first: bool,
}

/// Template for Redoc empty state
#[derive(Template)]
#[template(path = "redoc/empty.html")]
struct RedocEmptyTemplate;

/// API info for Redoc template
pub struct RedocApiInfo {
    pub name: String,
    pub slug: String,
    pub spec_url: String,
}

impl From<&ApiInfo> for RedocApiInfo {
    fn from(api: &ApiInfo) -> Self {
        RedocApiInfo {
            name: api.name.clone(),
            slug: api.slug.clone(),
            spec_url: api.spec_url.clone(),
        }
    }
}

pub struct RedocFrontend {
    config: RedocConfig,
}

impl RedocFrontend {
    pub fn new(config: RedocConfig) -> Self {
        Self { config }
    }
}

impl DocFrontend for RedocFrontend {
    fn generate_html(&self, apis: &[ApiInfo]) -> String {
        if apis.is_empty() {
            return self.generate_empty_html();
        }

        let redoc_apis: Vec<RedocApiInfo> = apis.iter().map(RedocApiInfo::from).collect();
        let template = RedocMainTemplate {
            apis: redoc_apis,
            has_multiple_apis: apis.len() > 1,
            show_api_selector: self.config.show_api_selector && apis.len() > 1,
            expand_responses: self.config.expand_responses.clone(),
            required_props_first: self.config.required_props_first,
        };

        template.render().unwrap_or_else(|e| {
            tracing::error!("Failed to render Redoc template: {}", e);
            format!("<html><body><h1>Template Error</h1><p>{e}</p></body></html>",)
        })
    }

    fn generate_empty_html(&self) -> String {
        let template = RedocEmptyTemplate;
        template.render().unwrap_or_else(|e| {
            tracing::error!("Failed to render Redoc empty template: {}", e);
            format!("<html><body><h1>Template Error</h1><p>{e}</p></body></html>",)
        })
    }
}

impl Default for RedocFrontend {
    fn default() -> Self {
        Self::new(RedocConfig::default())
    }
}
