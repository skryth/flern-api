use tokio::signal;

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler")
    };

    tokio::select! {
        _ = ctrl_c => {
            #[cfg(not(windows))]
            println!();
            tracing::info!("Ctrl+C recieved. Please wait, this could take a while.");
            std::process::exit(0);
        }
    }
}
