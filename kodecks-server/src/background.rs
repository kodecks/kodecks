use crate::app::AppState;
use std::sync::Arc;

pub async fn task(app: Arc<AppState>) -> std::io::Result<()> {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(500)).await;
        println!("Background task");
        app.cleanup();
    }
}
