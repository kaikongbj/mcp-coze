//! Minimal adapter for rmcp 0.5-style local tool responses during the 3.x migration.
//! Keep stdio tool behavior unchanged; convert only at the ServerHandler boundary.
use rmcp::model::{CallToolResult as NativeCallToolResult, ContentBlock};
use serde_json::Value;

pub struct CallToolResult {
    pub content: Option<Vec<ContentBlock>>,
    pub is_error: Option<bool>,
    pub structured_content: Option<Value>,
}

impl From<CallToolResult> for NativeCallToolResult {
    fn from(legacy: CallToolResult) -> Self {
        let mut result = NativeCallToolResult::default();
        result.content = legacy.content.unwrap_or_default();
        result.is_error = legacy.is_error;
        result.structured_content = legacy.structured_content;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::CallToolResult;
    use rmcp::model::CallToolResult as NativeCallToolResult;
    use serde_json::json;

    #[test]
    fn preserves_structured_success_and_error() {
        for is_error in [false, true] {
            let legacy = CallToolResult {
                content: None,
                is_error: Some(is_error),
                structured_content: Some(json!({"ok": !is_error})),
            };
            let native: NativeCallToolResult = legacy.into();
            assert!(native.content.is_empty());
            assert_eq!(native.is_error, Some(is_error));
            assert_eq!(native.structured_content, Some(json!({"ok": !is_error})));
        }
    }
}
