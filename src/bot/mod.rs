mod funcs;

use self::funcs::*;
use std::{collections::HashMap, time::Duration};

use chromiumoxide::{
    Browser, BrowserConfig, cdp::browser_protocol::target::SetDiscoverTargetsParams,
};
use futures::StreamExt;

use crate::{
    funcs::{dld_driver, sleep},
    log,
    types::TuError,
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
    let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
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

    if false && is_dev {
        config = config.with_head();
    }

    // and the handler that drives the websocket etc.
    let (mut browser, mut handler) = Browser::launch(config.build()?).await?;
    let handle = tokio::task::spawn(async move { while let Some(_) = handler.next().await {} });

    add_cookes(&browser).await?;

    // try login
    let yt_login_url = "https://accounts.google.com/v3/signin/identifier?continue=https%3A%2F%2Fwww.youtube.com%2Fsignin%3Faction_handle_signin%3Dtrue%26app%3Ddesktop%26hl%3Den-GB%26next%3Dhttps%253A%252F%252Fwww.youtube.com%252F%253FthemeRefresh%253D1&ec=65620&hl=en-GB&ifkv=AdBytiMg82rNXjrKk-bDCW2RCTv_AVeJyPNAoE9fE-rMgqAyEWKSQGlLPnbso6JiqWbDQ7fIZWT3&passive=true&service=youtube&uilel=3&flowName=GlifWebSignIn&flowEntry=ServiceLogin&dsh=S1060744614%3A1755524147182790";
    let page = browser.new_page(yt_login_url).await?;
    page.set_user_agent(user_agent).await?;
    page.wait_for_navigation().await?;

    if !page
        .url()
        .await?
        .unwrap()
        .starts_with("https://www.youtube.com")
    {
        login_to_yt(&page)
            .await
            .map_err(|err| format!("Failed to login to yt: {err:?}"))?;
    }

    log!("YT LOGGED IN!");
    page.close().await?;

    // go to like4like page
    let page = browser
        .new_page("https://www.like4like.org/user/earn-youtube.php")
        .await?;

    // Enable discovering new targets (popups)
    page.execute(SetDiscoverTargetsParams::builder().discover(true).build()?)
        .await?;

    for i in 1..=5 {
        let tag = format!("[task#{i}]");
        println!();
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
