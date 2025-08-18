use std::{collections::HashMap, error::Error, time::Duration};

use chromiumoxide::{
    Browser, BrowserConfig, Page, cdp::browser_protocol::target::SetDiscoverTargetsParams,
};
use futures::StreamExt;
use std::fs::{self};
use tokio::time;

use crate::{
    funcs::{dld_driver, sleep},
    log,
};
use chromiumoxide::cdp::browser_protocol::network::CookieParam;

pub async fn main() -> Result<(), Box<dyn Error>> {
    log!("STARTING HEADLESS BROWSER...");

    let env_vars = dotenv::vars().collect::<HashMap<_, _>>();
    let _driver_path = "/usr/bin/google-chrome".to_string();
    let mut is_dev = false;
    let driver_path = match env_vars.get("ENV") {
        Some(env) => {
            if env == "dev" {
                is_dev = true;
                _driver_path
            } else {
                let pth = dld_driver().await?;
                pth
            }
        }
        None => _driver_path,
    };

    log!("DRIVER_PATH: {driver_path}");

    let mut config = BrowserConfig::builder()
        .chrome_executable(driver_path)
        .launch_timeout(Duration::from_secs(5000));

    if is_dev {
        config = config.with_head();
    }

    // and the handler that drives the websocket etc.
    let (mut browser, mut handler) = Browser::launch(config.build()?).await?;

    let handle = tokio::task::spawn(async move { while let Some(_) = handler.next().await {} });

    let cookies_dir = "data/cookies";

    let mut cookies: Vec<CookieParam> = vec![];

    for file in fs::read_dir(cookies_dir).expect("Failed to cookies read dir") {
        if let Ok(entry) = file {
            if entry.file_type().unwrap().is_file()
                && entry.file_name().to_str().unwrap().ends_with(".json")
            {
                log!("{:?}", entry.path());
                let s = fs::read_to_string(entry.path()).expect("Failed to read cookie file");
                let _cookies: Vec<CookieParam> =
                    serde_json::from_str(&s).expect("Failed to parse cookie");
                cookies.extend(_cookies);
            }
        }
    }

    // add cookies
    browser.set_cookies(cookies).await?;
    // go to page
    let page = browser
        .new_page("https://www.like4like.org/user/earn-youtube.php")
        .await?;

    // Enable discovering new targets (popups)
    page.execute(SetDiscoverTargetsParams::builder().discover(true).build()?)
        .await?;

    page.wait_for_navigation().await?;
    find_click_btn(&page, "a.earn_pages_button", "earn").await?;

    let popup_page = wait_for_popup_page(&browser).await?;
    popup_page.wait_for_navigation().await?;

    find_click_btn(&popup_page, ".ytLikeButtonViewModelHost", "yt-like").await?;

    // wait a bit the close popup page
    sleep(1500).await;
    log!("Closing popup page...");
    popup_page.close().await?;

    sleep(500).await;
    // click confirm btn
    find_click_btn(&page, ".cursor.pulse-checkBox", "confirm").await?;
    sleep(500).await;
    browser.close().await?;
    let _ = handle.await;
    Ok(())
}

async fn find_click_btn(page: &Page, selector: &str, name: &str) -> Result<(), Box<dyn Error>> {
    log!("Finding {name} button...");
    match page.find_element(selector).await {
        Ok(btn) => {
            log!("Clicking {name} btn...");
            btn.click().await?;
        }
        Err(_) => {
            time::sleep(time::Duration::from_millis(1000)).await;
            return Box::pin(async { find_click_btn(page, selector, name).await }).await;
        }
    };
    Ok(())
}

async fn wait_for_popup_page(browser: &Browser) -> Result<Page, Box<dyn Error>> {
    let mut popup_page = None;
    while popup_page.is_none() {
        for page in browser.pages().await? {
            if let Ok(Some(url)) = page.url().await {
                if url.contains("youtube.com") {
                    popup_page.replace(page);
                }
            }
        }
        time::sleep(time::Duration::from_millis(1000)).await;
    }
    Ok(popup_page.unwrap())
}
