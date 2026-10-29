use crate::config::ScalarConfig;
use crate::frontend::{ApiInfo, DocFrontend};
use scalar_api_reference::scalar_html;
use serde_json::json;

/// Scalar bundle pinned to the major version, so upstream breaking releases don't leak in
const SCALAR_JS_URL: &str = "https://cdn.jsdelivr.net/npm/@scalar/api-reference@1";

pub struct ScalarFrontend {
    config: ScalarConfig,
}

impl ScalarFrontend {
    pub fn new(config: ScalarConfig) -> Self {
        Self { config }
    }
}

impl DocFrontend for ScalarFrontend {
    fn generate_html(&self, apis: &[ApiInfo]) -> String {
        if apis.is_empty() {
            return self.generate_empty_html();
        }

        let mut configurations = Vec::new();

        for (i, api) in apis.iter().enumerate() {
            let config = json!({
                "title": api.name.clone(),
                "slug": api.slug.clone(),
                "url": api.spec_url.clone(),
                "theme": self.config.theme,
                "layout": self.config.layout,
                "darkMode": self.config.dark_mode,
                "showSidebar": self.config.show_sidebar,
                "hideDownloadButton": self.config.hide_download_button,
                "expandAllResponses": self.config.expand_all_responses,
                "expandAllModelSections": self.config.expand_all_model_sections,
                "default": i == 0
            });

            configurations.push(config);
        }

        scalar_html(&json!(configurations), Some(SCALAR_JS_URL))
    }

    fn generate_empty_html(&self) -> String {
        let default_spec = json!({
            "openapi": "3.0.0",
            "info": {
                "title": "No APIs Found",
                "version": "1.0.0",
                "description": "No APIs are currently available"
            },
            "paths": {}
        });

        let configuration = json!([{
            "title": "No APIs Found",
            "content": default_spec.to_string(),
            "theme": self.config.theme,
            "layout": self.config.layout,
            "darkMode": self.config.dark_mode,
            "showSidebar": self.config.show_sidebar,
            "hideDownloadButton": self.config.hide_download_button,
            "expandAllResponses": self.config.expand_all_responses,
            "expandAllModelSections": self.config.expand_all_model_sections
        }]);

        scalar_html(&json!(configuration), Some(SCALAR_JS_URL))
    }
}
