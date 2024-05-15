use axum::{extract::FromRef, response::Html};
use axum_htmx::HxBoosted;
use minijinja::{context, Value};
use sqlx::SqlitePool;

use crate::{asset_cache::AssetCache, error::ApiError, routes::BaseTemplateData};

pub type SharedAppState = &'static AppState;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub sqlite: SqlitePool,
    pub reqwest_client: reqwest::Client,
    pub assets: &'static AssetCache,
    pub base_template_data: &'static BaseTemplateData,
    pub env: minijinja::Environment<'static>,
}

impl AppState {
    pub fn render(
        &self,
        HxBoosted(boosted): HxBoosted,
        template: &str,
    ) -> Result<Html<String>, ApiError> {
        let template = self
            .env
            .get_template(template)
            .map_err(|_| ApiError::TemplateNotFound(template.into()))?;

        if boosted {
            match template.render(context! {}) {
                Ok(rendered) => return Ok(Html(rendered)),
                Err(_) => return Err(ApiError::TemplateRender(template.name().into())),
            }
        }

        match template.render(context! {
            base => Some(self.base_template_data )
        }) {
            Ok(rendered) => Ok(Html(rendered)),
            Err(_) => Err(ApiError::TemplateRender(template.name().into())),
        }
    }

    pub fn render_with_context(
        &self,
        HxBoosted(boosted): HxBoosted,
        template: &str,
        ctx: Value,
    ) -> Result<Html<String>, ApiError> {
        let template = self
            .env
            .get_template(template)
            .map_err(|_| ApiError::TemplateNotFound(template.into()))?;

        if boosted {
            let rendered = template
                .render(ctx)
                .map_err(|_| ApiError::TemplateRender(template.name().into()))?;

            return Ok(Html(rendered));
        }

        match template.render(context! {
            base => Some(self.base_template_data), ..ctx
        }) {
            Ok(rendered) => Ok(Html(rendered)),
            Err(_) => Err(ApiError::TemplateRender(template.name().into())),
        }
    }
}
