use chrono::Local;
use playwright_rs::{BrowserContext, ClickOptions, Locator, Page, WaitForOptions};
use turs::{Res, log, sleep};

// pub async fn find_click(
//     page: &Page,
//     sel: &str,
//     tag: &str,
//     timeout: Option<f64>,
//     is_always_single: Option<bool>,
// ) -> Res<()> {
//     log!("[find] {tag} ({sel})");

//     let loc: Locator = find_el(page, sel, timeout, is_always_single).await?;
//     loc.scroll_into_view_if_needed().await?;
//     log!("[click] {sel}");
//     loc.click(None).await?;
//     Ok(())
// }

pub async fn find_el(page: &Page, sel: &str, timeout: Option<f64>) -> Res<Locator> {
    let timeout = timeout.unwrap_or(30_000.0);

    let loc = page.locator(sel).first();
    loc.wait_for(Some(WaitForOptions::builder().timeout(timeout).build()))
        .await?;

    Ok(loc)
}

pub async fn wait_for_popup_page(ctx: &BrowserContext, domain: &str) -> Res<Page> {
    log!("[wait_for_popup_page]");
    let mut popup_page = None;
    while popup_page.is_none() {
        for page in ctx.pages() {
            if page.is_closed() {
                continue;
            }
            let url = page.url();
            if url.contains(domain) {
                popup_page.replace(page);
                break;
            }
        }

        sleep(500).await;
    }
    Ok(popup_page.unwrap())
}

pub async fn click_el(page: &Page, sel: &str, timeout: Option<f64>, force: Option<bool>) -> Res<()> {
    
    page.locator(sel)
        .first()
        .click(
            ClickOptions::builder()
                .timeout(timeout.unwrap_or(30_000.0))
                .force(force.unwrap_or(false))
                .build(),
        )
        .await?;
    Ok(())
}
