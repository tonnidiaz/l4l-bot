use std::path::Path;

use chromiumoxide::{BrowserFetcher, BrowserFetcherOptions};
use tokio::time;

use crate::{log, types::TuError};

pub async fn dld_driver() -> Result<String, TuError> {

    log!("DOWNLOADING DRIVER...");
    let download_path = Path::new("./download");
    tokio::fs::create_dir_all(&download_path).await?;
    let fetcher = BrowserFetcher::new(
        BrowserFetcherOptions::builder()
            .with_path(&download_path)
            .build()?,
    );
    let info = fetcher.fetch().await?;
    let pth = info.executable_path.to_str().unwrap().to_string();

    log!("DRIVER DOWNLOADED");
    Ok(pth)
}

pub async fn sleep(ms: u64){
    time::sleep(time::Duration::from_millis(ms)).await;
}