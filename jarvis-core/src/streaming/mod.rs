#[derive(Debug, Clone)]
pub enum StreamEvent {
    Text(String),
    Reasoning(String),
    ToolCall {
        name: String,
        args: String,
    },
    ToolResult {
        call_id: String,
        result: String,
    },
    Done,
}

impl StreamEvent {
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    pub fn reasoning(s: impl Into<String>) -> Self {
        Self::Reasoning(s.into())
    }

    pub fn tool_call(name: impl Into<String>, args: impl Into<String>) -> Self {
        Self::ToolCall {
            name: name.into(),
            args: args.into(),
        }
    }

    pub fn tool_result(call_id: impl Into<String>, result: impl Into<String>) -> Self {
        Self::ToolResult {
            call_id: call_id.into(),
            result: result.into(),
        }
    }

    pub fn done() -> Self {
        Self::Done
    }
}
