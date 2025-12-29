use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JarvisMessage {
    pub role: JarvisMessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum JarvisMessageRole {
    User,
    Assistant,
    System,
}

impl JarvisMessage {
    pub fn new(role: JarvisMessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            timestamp: Utc::now(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(JarvisMessageRole::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(JarvisMessageRole::Assistant, content)
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new(JarvisMessageRole::System, content)
    }
}
