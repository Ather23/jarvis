use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Streaming error: {0}")]
    Streaming(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Timeout exceeded")]
    Timeout,

    #[error("Cancelled")]
    Cancelled,
}

impl Error {
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    pub fn provider(msg: impl Into<String>) -> Self {
        Self::Provider(msg.into())
    }

    pub fn agent(msg: impl Into<String>) -> Self {
        Self::Agent(msg.into())
    }

    pub fn streaming(msg: impl Into<String>) -> Self {
        Self::Streaming(msg.into())
    }

    pub fn tool(msg: impl Into<String>) -> Self {
        Self::Tool(msg.into())
    }

    pub fn tool_execution(msg: impl Into<String>) -> Self {
        Self::ToolExecution(msg.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
