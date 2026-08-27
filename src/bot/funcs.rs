use std::fs;

use playwright_rs::{Cookie, Page};
use turs::{Res, log};

pub async fn add_cookes(page: &Page) -> Res<()> {
    log!("ADDING COOKIES...");

    let cookies_dir = "data/cookies";

    let mut cookies: Vec<Cookie> = Vec::new();

    for file in fs::read_dir(cookies_dir).expect("Failed to cookies read dir") {
        if let Ok(entry) = file {
            if entry.file_type().unwrap().is_file()
                && entry.file_name().to_str().unwrap().ends_with(".json")
            {
                log!("{:?}", entry.path());
                let s = fs::read_to_string(entry.path()).expect("Failed to read cookie file");
                let _cookies: Vec<Cookie> =
                    serde_json::from_str(&s).expect("Failed to parse cookie");
                /* log!("C: {:?}", _cookies[0].expires.as_ref().unwrap());
                cookies.extend(_cookies); */
                cookies.extend(_cookies);
            }
        }
    }

    // add cookies
    log!("Collected {} cookies", cookies.len());
    page.context().unwrap().add_cookies(&cookies).await?;
    Ok(())
}
