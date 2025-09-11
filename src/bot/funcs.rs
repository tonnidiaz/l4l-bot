use chromiumoxide::{
    Browser, Page,
    cdp::js_protocol::runtime::{CallArgument, CallFunctionOnParams},
};
use std::fs::{self};
use tokio::time;

use crate::{bot::utils::IS_FB, funcs::sleep, log, types::TuError};
use chromiumoxide::cdp::browser_protocol::network::CookieParam;

pub async fn find_click_btn(
    page: &Page,
    sel: (&str, &str),
    max: Option<usize>,
) -> Result<(), TuError> {
    let (selector, name) = sel;
    let mut ok = false;
    for _ in 0..max.unwrap_or_else(|| 5) {
        log!("Finding {name} button...");
        if let Ok(btn) = page.find_element(selector).await {
            log!("Clicking {name} btn...");
            btn.click().await?;
            ok = true;
            break;
        } else {
            time::sleep(time::Duration::from_millis(1500)).await;
        }
    }
    if !ok{
        return Err(format!("Could not find {name} btn!").into());
    }
    Ok(())
}

pub async fn wait_for_popup_page(browser: &Browser, domain: &str) -> Result<Page, TuError> {
    log!("[wait_for_popup_page]");
    let mut popup_page = None;
    while popup_page.is_none() {
        for page in browser.pages().await? {
            if let Ok(Some(url)) = page.url().await {
               
                if url.contains(domain) {
                    log!("{url}");
                    popup_page.replace(page);
                    break;
                }
            }
        }
        time::sleep(time::Duration::from_millis(1000)).await;
    }
    Ok(popup_page.unwrap())
}

pub async fn login_to_yt(page: &Page) -> Result<(), TuError> {
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
        for i in 0..10 {
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
            let body = page.find_element("body").await?.inner_html().await?;
            if i == 0 {
                log!("{}", body.unwrap());
            }
            sleep(1000).await;
        }
        let input = input.ok_or("Failed to find pwd field.")?;
        log!("Typing pwd...");
        /* sleep(1000).await; */
        input.click().await?;
        log!("[clicked]");
        input.type_str(pwd).await?;
        log!("SUBMITTING PWD...");
        input.press_key("Enter").await?;

        log!("WAITING FOR NAV...");
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

pub async fn add_cookes(browser: &Browser) -> Result<(), TuError> {
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
                log!("C: {:?}", _cookies[0].expires.as_ref().unwrap());
                cookies.extend(_cookies);
            }
        }
    }

    // add cookies
    browser.set_cookies(cookies).await?;
    Ok(())
}

pub async fn start_task(browser: &Browser, page: &Page) -> Result<(), TuError> {
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
        if let Err(_err) = find_click_btn(&page, ("a.earn_pages_button", "earn"), None).await {
            // Refresh page
            log!("[{i}] REFRESH_PAGE...");
            sleep(3000).await;
            page.reload().await?;
            page.wait_for_navigation().await?;
        } else {
            break;
        }
    }
    let domain = if IS_FB { "facebook.com" } else { "youtube.com" };
    let popup_page = wait_for_popup_page(&browser, domain).await?;
    popup_page.wait_for_navigation().await?;
    let sel = if IS_FB {
        (r#"div[aria-label="Follow"]"#, "fb-follow")
    } else {
        (".ytLikeButtonViewModelHost", "yt-like")
    };
    popup_page.bring_to_front().await?;
    if let Err(err) = find_click_btn(&popup_page, sel, None).await{
        log!("{err:?}");
    };

    // wait a bit the close popup page
    sleep(1500).await;
    log!("Closing popup page...");
    popup_page.close().await?;

    sleep(1500).await;
    // click confirm btn
    find_click_btn(&page, (".cursor.pulse-checkBox", "confirm"), None).await?;

    Ok(())
}

pub async fn page_url(page: &Page) -> String {
    page.url().await.unwrap().unwrap()
}
