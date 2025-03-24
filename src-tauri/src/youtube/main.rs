use std::path::Path;

use anyhow::Result;
use lazy_static::lazy_static;
use rustypipe::client::RustyPipe;
use tauri::async_runtime::Mutex;

lazy_static! {
    pub static ref RP_CLIENT: Mutex<RustyPipe> = Mutex::new(
        RustyPipe::builder()
            .no_storage()
            .no_botguard()
            .build()
            .unwrap()
    );
}

pub async fn build_client(storage_dir: &Path) -> Result<()> {
    let mut client = RP_CLIENT.lock().await;

    *client = RustyPipe::builder()
        .unauthenticated()
        .storage_dir(storage_dir)
        .no_botguard()
        .build()?;

    Ok(())
}
