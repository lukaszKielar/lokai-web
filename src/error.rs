use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    DatabaseError(sqlx::Error),
    #[from]
    ReqwestError(reqwest::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum ApiError {
    TemplateNotFound(String),
    TemplateRender(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status_code, message) = match self {
            Self::TemplateNotFound(template_name) => (
                StatusCode::NOT_FOUND,
                format!("template \"{template_name}\" does not exist"),
            ),
            Self::TemplateRender(template_name) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to render template \"{template_name}\""),
            ),
        };

        (status_code, message).into_response()
    }
}
