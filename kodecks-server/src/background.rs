use crate::app::AppState;
use std::sync::Arc;
use tracing::info;

pub async fn task(app: Arc<AppState>) -> std::io::Result<()> {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(500)).await;
        info!("Running background task");
        app.cleanup();
    }
}
