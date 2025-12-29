use crate::error::{Error, Result};
use crate::providers::OpenRouterConfig;
use crate::streaming::StreamEvent;
use crate::agent::message::{JarvisMessage, JarvisMessageRole};
use crate::tools::ShellExecute;

use futures::Stream;
use std::pin::Pin;

/// Type alias for the agent's stream output
pub type AgentStream = Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>;

/// The main Jarvis AI agent
pub struct JarvisAgent {
    config: OpenRouterConfig,
    messages: Vec<JarvisMessage>,
    has_shell_tool: bool,
}

impl JarvisAgent {
    /// Create a new JarvisAgent with the given configuration
    pub fn new(config: OpenRouterConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(Error::configuration("API key is required"));
        }

        Ok(Self {
            config,
            messages: Vec::new(),
            has_shell_tool: false,
        })
    }

    /// Enable the shell command execution tool
    pub fn with_shell_tool(mut self) -> Self {
        self.has_shell_tool = true;
        self
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Set the temperature for generation
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = Some(temperature);
        self
    }

    /// Set the system preamble
    pub fn with_preamble(mut self, preamble: impl Into<String>) -> Self {
        self.config.preamble = Some(preamble.into());
        self
    }

    /// Get the current message history
    pub fn messages(&self) -> &[JarvisMessage] {
        &self.messages
    }

    /// Add a message to the history
    pub fn add_message(&mut self, role: JarvisMessageRole, content: impl Into<String>) {
        self.messages.push(JarvisMessage::new(role, content));
    }

    /// Clear the message history
    pub fn clear_history(&mut self) {
        self.messages.clear();
    }

    /// Get the current configuration
    pub fn config(&self) -> &OpenRouterConfig {
        &self.config
    }

    /// Check if shell tool is enabled
    pub fn has_shell_tool(&self) -> bool {
        self.has_shell_tool
    }

    /// Run a prompt and get a complete response (non-streaming)
    pub async fn run(&mut self, prompt: &str) -> Result<String> {
        // Add user message to history
        self.add_message(JarvisMessageRole::User, prompt);

        // Build the request payload
        let messages = self.build_messages();
        let tools = if self.has_shell_tool {
            Some(vec![ShellExecute::tool_definition()])
        } else {
            None
        };

        // Make API request
        let response = self.call_api(&messages, tools.as_deref(), false).await?;

        // Add assistant response to history
        self.add_message(JarvisMessageRole::Assistant, &response);

        Ok(response)
    }

    /// Run a prompt with streaming response
    pub async fn run_stream(&mut self, prompt: &str) -> Result<AgentStream> {
        // Add user message to history
        self.add_message(JarvisMessageRole::User, prompt);

        // Build the request payload
        let messages = self.build_messages();
        let tools = if self.has_shell_tool {
            Some(vec![ShellExecute::tool_definition()])
        } else {
            None
        };

        // Create streaming request
        let stream = self.call_api_stream(&messages, tools.as_deref()).await?;

        Ok(stream)
    }

    fn build_messages(&self) -> Vec<serde_json::Value> {
        let mut messages = Vec::new();

        // Add system preamble if set
        if let Some(ref preamble) = self.config.preamble {
            messages.push(serde_json::json!({
                "role": "system",
                "content": preamble
            }));
        }

        // Add conversation history
        for msg in &self.messages {
            let role = match msg.role {
                JarvisMessageRole::User => "user",
                JarvisMessageRole::Assistant => "assistant",
                JarvisMessageRole::System => "system",
            };
            messages.push(serde_json::json!({
                "role": role,
                "content": msg.content
            }));
        }

        messages
    }

    async fn call_api(
        &self,
        messages: &[serde_json::Value],
        tools: Option<&[serde_json::Value]>,
        _stream: bool,
    ) -> Result<String> {
        let client = reqwest::Client::new();

        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "stream": false
        });

        if let Some(temp) = self.config.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(tools) = tools {
            body["tools"] = serde_json::json!(tools);
        }

        let mut request = client
            .post(&format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json");

        if let Some(ref site_url) = self.config.site_url {
            request = request.header("HTTP-Referer", site_url);
        }

        if let Some(ref app_name) = self.config.app_name {
            request = request.header("X-Title", app_name);
        }

        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::provider(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::provider(format!("API error {}: {}", status, text)));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::provider(e.to_string()))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    async fn call_api_stream(
        &self,
        messages: &[serde_json::Value],
        tools: Option<&[serde_json::Value]>,
    ) -> Result<AgentStream> {
        use futures::stream;

        let client = reqwest::Client::new();

        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "stream": true
        });

        if let Some(temp) = self.config.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(tools) = tools {
            body["tools"] = serde_json::json!(tools);
        }

        let mut request = client
            .post(&format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json");

        if let Some(ref site_url) = self.config.site_url {
            request = request.header("HTTP-Referer", site_url);
        }

        if let Some(ref app_name) = self.config.app_name {
            request = request.header("X-Title", app_name);
        }

        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::provider(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::provider(format!("API error {}: {}", status, text)));
        }

        // Create a stream from the response bytes
        let byte_stream = response.bytes_stream();

        let event_stream = stream::unfold(
            (byte_stream, String::new()),
            |(mut byte_stream, mut buffer)| async move {
                use futures::StreamExt;

                loop {
                    // Check if we have complete SSE events in the buffer
                    if let Some(pos) = buffer.find("\n\n") {
                        let event = buffer[..pos].to_string();
                        buffer = buffer[pos + 2..].to_string();

                        // Parse the SSE event
                        if let Some(data) = event.strip_prefix("data: ") {
                            if data.trim() == "[DONE]" {
                                return Some((Ok(StreamEvent::done()), (byte_stream, buffer)));
                            }

                            match serde_json::from_str::<serde_json::Value>(data) {
                                Ok(json) => {
                                    // Extract content delta
                                    if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                        if !content.is_empty() {
                                            return Some((Ok(StreamEvent::text(content)), (byte_stream, buffer)));
                                        }
                                    }

                                    // Check for reasoning/thinking
                                    if let Some(reasoning) = json["choices"][0]["delta"]["reasoning_content"].as_str() {
                                        if !reasoning.is_empty() {
                                            return Some((Ok(StreamEvent::reasoning(reasoning)), (byte_stream, buffer)));
                                        }
                                    }

                                    // Check for tool calls
                                    if let Some(tool_calls) = json["choices"][0]["delta"]["tool_calls"].as_array() {
                                        for tc in tool_calls {
                                            if let (Some(name), Some(args)) = (
                                                tc["function"]["name"].as_str(),
                                                tc["function"]["arguments"].as_str(),
                                            ) {
                                                return Some((Ok(StreamEvent::tool_call(name, args)), (byte_stream, buffer)));
                                            }
                                        }
                                    }

                                    // Check if this is the final message
                                    if json["choices"][0]["finish_reason"].as_str().is_some() {
                                        return Some((Ok(StreamEvent::done()), (byte_stream, buffer)));
                                    }
                                }
                                Err(_) => {
                                    // Skip malformed JSON
                                }
                            }
                        }
                        continue;
                    }

                    // Read more data from the stream
                    match byte_stream.next().await {
                        Some(Ok(chunk)) => {
                            buffer.push_str(&String::from_utf8_lossy(&chunk));
                        }
                        Some(Err(e)) => {
                            return Some((Err(Error::streaming(e.to_string())), (byte_stream, buffer)));
                        }
                        None => {
                            // Stream ended
                            return None;
                        }
                    }
                }
            },
        );

        Ok(Box::pin(event_stream))
    }
}

impl Clone for JarvisAgent {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            messages: self.messages.clone(),
            has_shell_tool: self.has_shell_tool,
        }
    }
}
