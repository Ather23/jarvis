pub mod error;
pub mod agent;
pub mod tools;
pub mod streaming;
pub mod providers;

pub use error::{Error, Result};
pub use agent::{JarvisAgent, AgentStream, JarvisMessage, JarvisMessageRole};
pub use streaming::StreamEvent;
pub use providers::{OpenRouterConfig, OpenRouterConfigBuilder};
pub use tools::{ShellExecute, ShellArgs, ShellResult, ShellToolError};
