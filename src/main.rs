use flern::{error::run_with_error_handler, run};

#[tokio::main]
#[tracing::instrument]
async fn main() {
    run_with_error_handler(run).await;
}
