mod funcs;

use playwright_rs::{BrowserContext, ClickOptions, Locator, Page, WaitForOptions};
use turs::{Res, elog, log, sleep};

use crate::{
    bot::funcs::add_cookes,
    brwser::{self, BrMan, wait_nav},
    pw::funcs::{click_el, find_el, wait_for_popup_page},
};

pub struct Bot {
    pub ctx: BrowserContext,
    pub page: Page,
}

pub async fn main() -> Res<()> {
    let tag = "[tubot]";
    log!("{tag} STARTING...");
    let (ctx, page) = BrMan::new_page(tag, None, None)
        .await
        .expect("Failed to initialize BrowserManager");
    add_cookes(&page).await?;
    let bot = Bot { ctx, page };
    if let Err(e) = bot.check_yt_auth(tag).await {
        elog!("{tag} ERROR YT AUTH: {e}");
        return Err("".into());
    }
    log!("{tag} YT AUTH SUCCESSFUL");

    let mut i = 0;
    let url = "https://www.like4like.org/user/earn-youtube.php";

    log!("{tag} [goto] {url}");
    brwser::goto(&bot.page, url, tag).await?;

    loop {
        i += 1;
        let tag = format!("[task-{i}]");
        if let Err(err) = bot.worker(&tag).await {
            elog!("worker error: {:?}", err);
            sleep(200_000_000).await;
            break;
        }
    }

    Ok(())
}

impl Bot {
    pub async fn check_yt_auth(&self, tag: &str) -> Res<()> {
        log!("CHECKING YT AUTH...");
        let yt_login_url = "https://www.youtube.com/account";
        brwser::goto(&self.page, yt_login_url, tag).await?;
        brwser::wait_nav(&self.page, tag).await?;
        log!("{tag} [find] avatar-btn...");
        if let Err(err) = find_el(&self.page, "#avatar-btn", None).await {
            elog!("{tag} No avatar bnt. Not logged in");
            return Err(err.into());
        }
        Ok(())
    }

    async fn worker(&self, tag: &str) -> Res<()> {
        let page = &self.page;
        log!("{tag} url: {}", page.url());
        let sel = "a.earn_pages_button";
        let _ = page.locator(sel).scroll_into_view_if_needed().await;

        if let Err(err) = click_el(page, sel, Some(30_000.0), Some(true)).await {
            if let Ok(cnt) = page.locator(sel).count().await {
                log!("{tag} COUNT: {cnt}");
            };
            return Err(err);
        };

        log!("{tag} [wait] popup page...");
        let popup_page = wait_for_popup_page(&self.ctx, "youtube.com").await?;
        log!("{tag} [popup_url] {}", popup_page.url());
        wait_nav(&popup_page, tag).await?;
        log!("{tag} bring to front...");
        popup_page.bring_to_front().await?;
        log!("{tag} [find] like-btn...");

        let like_btn_loc = popup_page.locator("like-button-view-model");
        let mut like_btn_cnt = 0;
        while like_btn_cnt <= 0 {
            sleep(500).await;
            like_btn_cnt = like_btn_loc.count().await.unwrap_or(0);
        }

        let like_btn_i = if like_btn_cnt > 1 { 1 } else { 0 };

        log!("{tag} [like_btn_i] {like_btn_i}");

        if let Err(err) = like_btn_loc.nth(like_btn_i).click(ClickOptions::builder().force(true).build()).await {
            elog!("Failed to click like btn: {err:?}");
            sleep(200_000_000).await;
        };

        sleep(1000).await;
        log!("Closing popup page...");
        popup_page.close().await?;

        sleep(1000).await;
        let _ = page.bring_to_front().await;
        log!("{tag} [click] confirm btn...");
        let sel = ".cursor.pulse-checkBox";
        let _ = page.locator(sel).scroll_into_view_if_needed().await;
        if let Err(err) = click_el(page, sel, Some(30_000.0), Some(true)).await {
            if let Ok(cnt) = page.locator(sel).count().await {
                log!("{tag} COUNT: {cnt}");
            };
            return Err(err);
        };
        Ok(())
    }
}
