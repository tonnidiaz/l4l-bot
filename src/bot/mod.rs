mod funcs;
mod utils;

use self::funcs::*;
use std::{collections::HashMap, time::Duration};

use chromiumoxide::{
    Browser, BrowserConfig, cdp::browser_protocol::target::SetDiscoverTargetsParams,
};
use futures::StreamExt;
use utils::IS_FB;

use crate::{
    funcs::{dld_driver, sleep},
    log,
    types::{Res, TuError},
};

pub async fn main() -> Result<(), TuError> {
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
    let _user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
Chrome/115.0.0.0 Safari/537.36";
    let mut config = BrowserConfig::builder()
        .chrome_executable(driver_path)
        .launch_timeout(Duration::from_secs(5000))
        .args([
            "--disable-blink-features=AutomationControlled",
            "--disable-infobars",
            "--disable-extensions",
            "--start-maximized",
            "--window-size=1280,720",
            "--no-sandbox",
            "--disable-setuid-sandbox",
        ]);

    if is_dev {
        config = config.with_head();
    }

    // and the handler that drives the websocket etc.
    let (mut browser, mut handler) = Browser::launch(config.build()?).await?;
    let handle = tokio::task::spawn(async move { while let Some(_) = handler.next().await {} });

    add_cookes(&browser).await?;

    // check platform auth
    check_fb_auth(&browser).await?;
    if false {
        return Ok(());
    }
    // go to like4like page
    let url = if IS_FB {
        "https://www.like4like.org/user/earn-facebook-subscribes.php"
    } else {
        "https://www.like4like.org/user/earn-youtube.php"
    };
    let page = browser.new_page(url).await?;

    // Enable discovering new targets (popups)
    page.execute(SetDiscoverTargetsParams::builder().discover(true).build()?)
        .await?;

    for i in 1..=100 {
        let tag = format!("[task#{i}]");
        println!("\n");
        log!("{tag} START TASK...");
        if let Err(err) = start_task(&browser, &page).await {
            log!("{tag} TASK FAILED: {err:?}");
        } else {
            log!("{tag} TASK OK!");
        };
    }
    sleep(500).await;
    browser.close().await?;
    let _ = handle.await;
    Ok(())
}

async fn check_yt_auth(browser: &Browser) -> Res<()> {
    let yt_login_url = "https://www.youtube.com/account";
    let page = browser.new_page(yt_login_url).await?;
    // page.set_user_agent(user_agent).await?;
    page.wait_for_navigation().await?;

    let url = page_url(&page).await;
    if url.starts_with("https://consent.youtube") {
        log!("FINDING ACCEPT BTN...");
        find_click_btn(
            &page,
            (r#"button[aria-label="Accept all"]"#, "accept all"),
            None,
        )
        .await?;
        page.wait_for_navigation().await?;
    }
    while !page_url(&page).await.starts_with("https://www.youtube.com") {
        log!("{}", page_url(&page).await);
        sleep(500).await;
    }

    let url = page_url(&page).await;
    if !url.starts_with("https://www.youtube.com") {
        return Err("NOT LOGGED IN TO GOOGLE".into());
    }

    log!("YT LOGGED IN!");
    page.close().await?;
    Ok(())
}

async fn check_fb_auth(browser: &Browser) -> Res<()> {
    let page = browser.new_page("https://www.facebook.com").await?;
    page.wait_for_navigation().await?;
    log!("ON FACEBOOK");
    // check avatar
    page.find_element(r#"div[aria-label="Your profile"]"#)
        .await?;
    Ok(())
}
