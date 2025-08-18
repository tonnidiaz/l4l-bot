use std::{collections::HashMap, error::Error, time::Duration};

use chromiumoxide::{
    Browser, BrowserConfig, Page,
    cdp::{
        browser_protocol::target::SetDiscoverTargetsParams,
        js_protocol::runtime::{CallArgument, CallFunctionOnParams},
    },
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

async fn find_click_btn(
    page: &Page,
    selector: &str,
    name: &str,
    max: Option<usize>,
) -> Result<(), Box<dyn Error>> {
    for _ in 0..max.unwrap_or_else(|| 5) {
        log!("Finding {name} button...");
        match page.find_element(selector).await {
            Ok(btn) => {
                log!("Clicking {name} btn...");
                btn.click().await?;
                break;
            }
            Err(_) => {
                time::sleep(time::Duration::from_millis(1000)).await;
            }
        };
    }

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

async fn login_to_yt(page: &Page) -> Result<(), Box<dyn Error>> {
    log!("[LOGIN_TO_YT]");
    let login = "therealhackingknight@gmail.com";
    let pwd = "Baseline@therealhackingknight@072";

    log!("Find email input...");
    if let Ok(input) = page.find_element(r#"input[type="email"]"#).await {
        log!("Typing email...");
        input.click().await?;
        input.type_str(login).await?;
        input.press_key("Enter").await?;

        page.wait_for_navigation().await?;

        log!("Find pwd input...");
        let mut input = None;
        loop {
            let mut done = false;
            if let Ok(inp) = page.find_element(r#"input[type="password"]"#).await {
                if inp.clickable_point().await.is_ok() {
                    done = true;
                }
                input.replace(inp);
            };

            if done {
                break;
            }
        }
        let input = input.unwrap();
        log!("Typing pwd...");
        /* sleep(1000).await; */
        input.click().await?;
        input.type_str(pwd).await?;
        input.press_key("Enter").await?;

        page.wait_for_navigation().await?;
        log!("URL: {:?}", page.url().await.unwrap());
        match page.url().await {
            Ok(url) => {
                if !url.unwrap().starts_with("https://www.youtube.com") {
                    return Err("Did not navigate to youtuble".into());
                }
            }
            Err(_) => {
                return Err("Failed to login to youtube".into());
            }
        }
    };
    Ok(())
}

async fn add_cookes(browser: &Browser) -> Result<(), Box<dyn Error>> {
    log!("ADDING COOKIES...");

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
    Ok(())
}

async fn start_task(browser: &Browser, page: &Page) -> Result<(), Box<dyn Error>> {
    page.wait_for_navigation().await?;

    // remove popover
    let js = r#"
  selector => {
    const el = document.querySelector(selector);
    if (el) {
      el.remove();
    }
  }
"#;
    let js = CallFunctionOnParams::builder()
        .function_declaration(js)
        .argument(CallArgument::builder().value("#popunder").build())
        .build()?;
    page.evaluate_function(js).await?;

    for i in 0..5 {
        if let Err(_err) = find_click_btn(&page, "a.earn_pages_button", "earn", None).await {
            // Refresh page
            log!("[{i}] REFRESH_PAGE...");
            sleep(3000).await;
            page.reload().await?;
            page.wait_for_navigation().await?;
        } else {
            break;
        }
    }

    let popup_page = wait_for_popup_page(&browser).await?;
    popup_page.wait_for_navigation().await?;

    find_click_btn(&popup_page, ".ytLikeButtonViewModelHost", "yt-like", None).await?;

    // wait a bit the close popup page
    sleep(1500).await;
    log!("Closing popup page...");
    popup_page.close().await?;

    sleep(1500).await;
    // click confirm btn
    find_click_btn(&page, ".cursor.pulse-checkBox", "confirm", None).await?;

    Ok(())
}
