use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmIoDebugRecord {
    pub function_id: String,
    pub round: u8,
    pub request_system_prompt: Option<String>,
    pub request_user_prompt: Option<String>,
    pub response_content: Option<String>,
    pub response_transport: Option<String>,
    pub request_error: Option<String>,
    pub candidate_count: usize,
}

impl LlmIoDebugRecord {
    pub fn request_failed(
        function_id: &str,
        round: u8,
        system_prompt: &str,
        user_prompt: &str,
        error: String,
    ) -> Self {
        Self {
            function_id: function_id.to_string(),
            round,
            request_system_prompt: Some(system_prompt.to_string()),
            request_user_prompt: Some(user_prompt.to_string()),
            response_content: None,
            response_transport: None,
            request_error: Some(error),
            candidate_count: 0,
        }
    }

    pub fn request_succeeded(
        function_id: &str,
        round: u8,
        system_prompt: &str,
        user_prompt: &str,
        response_content: Option<String>,
        response_transport: Option<String>,
        candidate_count: usize,
    ) -> Self {
        Self {
            function_id: function_id.to_string(),
            round,
            request_system_prompt: Some(system_prompt.to_string()),
            request_user_prompt: Some(user_prompt.to_string()),
            response_content,
            response_transport,
            request_error: None,
            candidate_count,
        }
    }
}
