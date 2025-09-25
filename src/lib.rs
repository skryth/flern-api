#![allow(dead_code)] // FIXME: Dev only

use std::path::{Path, PathBuf};

use crate::model::{DbConnection, ModelManager};
use crate::utils::signal::shutdown_signal;
use crate::{error::AppResult, web::AppState};
use axum::Router;
use sqlx::migrate::Migrator;
use tokio::net::TcpListener;

pub mod config;
pub use config::{Config, ConfigError, ConfigResult};

pub mod auth;
pub mod error;
pub mod model;
pub mod utils;
pub mod web;

static APPLICATION_NAME: &str = "flern";

pub async fn build_server() -> AppResult<(AppState, Router)> {
    config::Config::get_or_init().await;

    let config = config::Config::get_or_init().await;
    let db = DbConnection::connect(config.app().database_uri())?;

    let migrator = Migrator::new(Path::new("./migrations")).await.unwrap();
    tracing::debug!("applying migrations...");
    migrator.run(db.pool()).await.unwrap();

    let mm = ModelManager::new(db);
    let state = AppState::new(mm);
    let app = web::routes::build_app(state.clone(), config);
    Ok((state, app))
}

pub async fn build_server_with_pool(db: DbConnection) -> AppResult<(AppState, Router)> {
    let config = config::Config::get_or_init().await;

    let mm = ModelManager::new(db);
    let state = AppState::new(mm);
    let app = web::routes::build_app(state.clone(), config);
    Ok((state, app))
}

#[tracing::instrument]
pub async fn setup_workers() -> AppResult<()> {
    let (_, app) = build_server().await?;
    let config = Config::get_or_init().await;
    let listener = TcpListener::bind(config.host().bindto()).await?;

    tracing::info!("axum is starting at: {}", config.host().bindto());
    let axum_handle = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal());

    axum_handle.await?;
    Ok(())
}

fn setup_trace() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};

    // load .env file for RUST_LOG etc.
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .with(ErrorLayer::default())
        .init();

    tracing::debug!("tracing initialized.");
}

fn log_runtime() {
    let cwd = std::env::current_dir()
        .unwrap_or(PathBuf::new());
    tracing::info!("cwd: {}", cwd.display());
}

#[tracing::instrument]
pub async fn run() -> AppResult<()> {
    setup_trace();
    log_runtime();
    setup_workers().await?;
    Ok(())
}
