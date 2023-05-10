use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug)]
pub enum AppError {
    Anyhow(anyhow::Error),
    BadRequest(String),
    ServerError(String),
    CodeMessage(i32, String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (code, custom_code, msg) = match self {
            AppError::Anyhow(a) => (StatusCode::INTERNAL_SERVER_ERROR, 0, a.to_string()),
            AppError::BadRequest(s) => (StatusCode::BAD_REQUEST, 0, s),
            AppError::ServerError(s) => (StatusCode::INTERNAL_SERVER_ERROR, 0, s),
            AppError::CodeMessage(c, m) => (StatusCode::BAD_REQUEST, c, m),
        };

        let res_json = serde_json::json!({
            "custom_code": custom_code,
            "message": msg
        });

        (code, Json(res_json)).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Anyhow(err.into())
    }
}

pub type AppResult<T> = anyhow::Result<T, AppError>;
