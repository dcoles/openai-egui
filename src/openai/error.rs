//! Errors.

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorObject,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct ErrorObject {
    pub message: String,
    pub r#type: String,
    pub param: Option<String>,
    pub code: Option<String>,
}
