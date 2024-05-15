use axum::{middleware, routing::get, Router};
use serde::Serialize;

use crate::{asset_cache::SharedAssetCache, cache_control, state::SharedAppState};

pub mod index;
pub mod not_found;
pub mod robots;

use index::index;
use not_found::not_found;
use robots::robots;

pub type SharedBaseTemplateData = &'static BaseTemplateData;

#[derive(Clone, Serialize)]
pub struct BaseTemplateData {
    styles: String,
    scripts: String,
    favicon: String,
}

impl BaseTemplateData {
    pub fn new(assets: SharedAssetCache) -> Self {
        let styles = assets
            .get("index.css")
            .expect("failed to build base template data: index.css")
            .path
            .clone();

        let scripts = assets
            .get("index.js")
            .expect("failed to build base template data: index.js")
            .path
            .clone();

        let favicon = assets
            .get("favicon.ico")
            .expect("failed to build base template date: favicon.ico")
            .path
            .clone();

        Self {
            styles,
            scripts,
            favicon,
        }
    }
}

pub fn route_handler(state: SharedAppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/robots.txt", get(robots))
        .fallback(not_found)
        .with_state(state)
        .layer(middleware::from_fn(cache_control))
}
