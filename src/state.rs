use axum::{extract::FromRef, response::Html};
use minijinja::{context, Value};
use sqlx::SqlitePool;

use crate::error::ApiError;

pub type SharedAppState = &'static AppState;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub sqlite: SqlitePool,
    pub reqwest_client: reqwest::Client,
    pub env: minijinja::Environment<'static>,
}

impl AppState {
    pub fn render(&self, template: &str) -> Result<Html<String>, ApiError> {
        let template = self
            .env
            .get_template(template)
            .map_err(|_| ApiError::TemplateNotFound(template.into()))?;

        match template.render(context! {}) {
            Ok(rendered) => return Ok(Html(rendered)),
            Err(_) => return Err(ApiError::TemplateRender(template.name().into())),
        }
    }

    pub fn render_with_context(
        &self,
        template: &str,
        ctx: Value,
    ) -> Result<Html<String>, ApiError> {
        let template = self
            .env
            .get_template(template)
            .map_err(|_| ApiError::TemplateNotFound(template.into()))?;

        let rendered = template
            .render(ctx)
            .map_err(|_| ApiError::TemplateRender(template.name().into()))?;

        return Ok(Html(rendered));
    }
}
