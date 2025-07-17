use anyhow::anyhow;
use futures::lock::Mutex;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LanguagePolicy {
    Deterministic,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize)]
pub struct Language {
    policy: LanguagePolicy,
    code: String,
}

impl Language {
    pub fn new(policy: LanguagePolicy, code: impl Into<String>) -> Self {
        Self {
            policy,
            code: code.into(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum ParameterType {
    Text,
}

#[derive(Debug, Serialize, Clone)]
pub struct TextParameter {
    r#type: ParameterType,
    text: String,
}

impl TextParameter {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            r#type: ParameterType::Text,
            text: text.into(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ComponentType {
    Body,
    Button,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateType {
    Authentication,
}

impl fmt::Display for TemplateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_str = match self {
            TemplateType::Authentication => "authentication",
        };
        write!(f, "{}", display_str)
    }
}

#[derive(Debug, Serialize)]
pub struct Component {
    r#type: ComponentType,
    parameters: Vec<TextParameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sub_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<u8>,
}

impl Component {
    pub fn body(parameters: Vec<TextParameter>) -> Self {
        Self {
            r#type: ComponentType::Body,
            parameters,
            sub_type: None,
            index: None,
        }
    }

    pub fn button(parameters: Vec<TextParameter>, sub_type: impl Into<String>, index: u8) -> Self {
        Self {
            r#type: ComponentType::Button,
            parameters,
            sub_type: Some(sub_type.into()),
            index: Some(index),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TemplateData {
    name: TemplateType,
    language: Language,
    components: Vec<Component>,
}

impl TemplateData {
    pub fn builder(name: TemplateType, language: Language) -> TemplateBuilder {
        TemplateBuilder {
            name,
            language,
            components: Vec::new(),
        }
    }
}

pub struct TemplateBuilder {
    name: TemplateType,
    language: Language,
    components: Vec<Component>,
}

impl TemplateBuilder {
    pub fn with_component(mut self, component: Component) -> Self {
        self.components.push(component);
        self
    }

    pub fn build(self) -> TemplateData {
        TemplateData {
            name: self.name,
            language: self.language,
            components: self.components,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Template,
}

#[derive(Debug, Serialize)]
pub struct SinchTextMessagePayload {
    recipient_type: String,
    to: String,
    r#type: MessageType,
    template: TemplateData,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
}

impl SinchTextMessagePayload {
    pub fn new(to: impl Into<String>, template: TemplateData) -> Self {
        Self {
            recipient_type: "individual".into(),
            to: to.into(),
            r#type: MessageType::Template,
            template,
            metadata: None,
        }
    }
}

#[derive(Debug)]
pub struct WhatsAppClient {
    base_url: String,
    auth_url: String,
    username: String,
    password: String,
    http_client: Client,
    access_token: Arc<Mutex<Option<String>>>,
}

impl WhatsAppClient {
    pub fn new(
        base_url: String,
        auth_url: String,
        username: String,
        password: String,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            base_url,
            auth_url,
            username,
            password,
            http_client: Client::builder().timeout(timeout).build().unwrap(),
            access_token: Arc::new(Mutex::new(None)),
        }
    }

    async fn authenticate(&self) -> Result<String, anyhow::Error> {
        let form = [
            ("grant_type", "password"),
            ("client_id", "conv-api"),
            ("username", &self.username),
            ("password", &self.password),
        ];
        let url = format!(
            "{}/realms/prepaid/protocol/ openid-connect/token ",
            self.auth_url
        );
        let res = self
            .http_client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&form)
            .send()
            .await?;

        if res.status().is_success() {
            let token: TokenResponse = res.json().await?;
            let mut guard = self.access_token.lock().await;
            *guard = Some(token.access_token.clone());
            Ok(token.access_token)
        } else {
            Err(anyhow!("Authentication failed: {}", res.status()))
        }
    }

    async fn get_token(&self) -> Result<String, anyhow::Error> {
        let mut token_guard = self.access_token.lock().await;
        match &*token_guard {
            Some(token) => Ok(token.clone()),
            None => {
                let token = self.authenticate().await?;
                *token_guard = Some(token.clone());
                Ok(token)
            }
        }
    }

    pub async fn send_text(
        &self,
        template_name: TemplateType,
        recipient_phone: &str,
        par_list: Vec<&str>,
        is_auth: bool,
    ) -> Result<String, anyhow::Error> {
        let parameters: Vec<TextParameter> = par_list.into_iter().map(TextParameter::new).collect();

        let mut builder = TemplateData::builder(
            template_name,
            Language::new(LanguagePolicy::Deterministic, "en"),
        )
        .with_component(Component::body(parameters.clone()));

        if is_auth {
            builder = builder.with_component(Component::button(parameters, "url", 0));
        }

        let payload = SinchTextMessagePayload::new(recipient_phone, builder.build());

        let url = format!("{}/pull-platform-receiver/v2/wa/messages", self.base_url);
        let token = self.get_token().await?;

        let mut res = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&payload)
            .send()
            .await?;

        if res.status() == StatusCode::UNAUTHORIZED {
            let token = self.authenticate().await?;
            res = self
                .http_client
                .post(&url)
                .bearer_auth(&token)
                .json(&payload)
                .send()
                .await?;
        }

        if res.status().is_success() {
            Ok(res.text().await?)
        } else {
            Err(anyhow!("Message send failed: {}", res.status()))
        }
    }
}
