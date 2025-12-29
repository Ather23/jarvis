use crate::error::Error;

#[derive(Clone, Debug)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub model: String,
    pub site_url: Option<String>,
    pub app_name: Option<String>,
    pub temperature: Option<f32>,
    pub preamble: Option<String>,
    pub base_url: String,
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENROUTER_API_KEY").unwrap_or_default(),
            model: "google/gemini-2.5-flash".to_string(),
            site_url: std::env::var("OPENROUTER_SITE_URL").ok(),
            app_name: std::env::var("OPENROUTER_APP_NAME").ok().or(Some("jarvis-core".to_string())),
            temperature: None,
            preamble: None,
            base_url: "https://openrouter.ai/api/v1".to_string(),
        }
    }
}

impl OpenRouterConfig {
    pub fn builder() -> OpenRouterConfigBuilder {
        OpenRouterConfigBuilder::new()
    }

    pub fn from_env() -> Result<Self, Error> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| Error::configuration("OPENROUTER_API_KEY not set"))?;
        Ok(Self::builder()
            .api_key(api_key)
            .build())
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_preamble(mut self, preamble: impl Into<String>) -> Self {
        self.preamble = Some(preamble.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct OpenRouterConfigBuilder {
    config: OpenRouterConfig,
}

impl OpenRouterConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: OpenRouterConfig::default(),
        }
    }

    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.config.api_key = api_key.into();
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    pub fn site_url(mut self, site_url: impl Into<String>) -> Self {
        self.config.site_url = Some(site_url.into());
        self
    }

    pub fn app_name(mut self, app_name: impl Into<String>) -> Self {
        self.config.app_name = Some(app_name.into());
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = Some(temperature);
        self
    }

    pub fn preamble(mut self, preamble: impl Into<String>) -> Self {
        self.config.preamble = Some(preamble.into());
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.config.base_url = base_url.into();
        self
    }

    pub fn build(self) -> OpenRouterConfig {
        self.config
    }
}

impl Default for OpenRouterConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
