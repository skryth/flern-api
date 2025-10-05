use tokio::signal;
use tracing::warn;

/// Waits for SIGINT or SIGTERM and triggers graceful shutdown.
pub async fn shutdown_signal() {
    #[cfg(unix)]
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("failed to install SIGTERM handler");

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler")
    };

    #[cfg(unix)]
    let terminate = async {
        sigterm.recv().await;
    };

    #[cfg(unix)]
    tokio::select! {
        _ = ctrl_c => {
            println!();
            warn!("SIGINT received, shutting down gracefully...");
        }
        _ = terminate => {
            warn!("SIGTERM received, shutting down gracefully...");
        }
    }

    #[cfg(not(unix))]
    tokio::select! {
        _ = ctrl_c => {
            warn!("SIGINT (Ctrl+C) received, shutting down gracefully...");
        }
    }

    std::process::exit(0);
}
