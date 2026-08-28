mod funcs;

use std::env::args;

use playwright_rs::{BrowserContext, ClickOptions, GotoOptions, Locator, Page, WaitUntil};
use turs::{Res, elog, log, sleep};

use crate::{
    bot::funcs::add_cookes, brwser::{self, BrMan, wait_nav}, pw::funcs::{click_el, find_any, find_el, wait_for_popup_page},
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

    let site = args().nth(1).unwrap_or("insta".to_string());
    log!("{tag} site = {site}");
    bot.worker_all(bot.page.clone(), &site).await?;

    Ok(())
}

impl Bot {
    pub async fn check_site_auth(&self, site: &str, tag: &str) -> Res<()> {
        log!("CHECKING [{site}] AUTH...");

        let url;
        let auth_sel;
        match site {
            "yt" => {
                url = "https://www.youtube.com/account";
                auth_sel = "#avatar-btn";
            }
            "insta" | "insta-like" => {
                url = "https://www.instagram.com/accounts/edit";
                auth_sel = r#"[alt*="profile picture"]"#;
            }
            _ => {
                return Err("Invalid site".into());
            }
        }

        brwser::goto(&self.page, url, tag).await?;
        brwser::wait_nav(&self.page, tag).await?;
        log!("{tag} [find] avatar-btn...");
        if let Err(err) = find_el(&self.page, auth_sel, None).await {
            elog!("{tag} No avatar bnt. Not logged in");
            return Err(err.into());
        }

        log!("{tag} [{site} auth successful!");
        Ok(())
    }

    async fn worker_all(&self, page: Page, site: &str) -> Res<()> {
        let url;
        let mut done = 0;
        let tag = "[bot_insta]";
        let act_btn_sel: &[&str];
        let domain;

        match site {
            "yt" => {
                url = "https://www.like4like.org/user/earn-youtube.php";
                act_btn_sel = &["like-button-view-model"];
                domain = "youtube.com";
            }

            "insta" => {
                url = "https://www.like4like.org/user/earn-instagram-follow.php";
                act_btn_sel = &["header > section button"];
                domain = "instagram.com";
            }

            "insta-like" => {
                url = "https://www.like4like.org/user/earn-instagram-like.php";
                act_btn_sel = &[
                    "span.x1qfufaz > div:nth-child(1) > div:nth-child(1)",
                    "div.xg7h5cd:nth-child(1) > div:nth-child(1) > div:nth-child(1) > div:nth-child(2) > div:nth-child(1) > span:nth-child(1) > div:nth-child(1) > div:nth-child(1) > div:nth-child(1)",
                    "div.x1mfogq2:nth-child(1) > div:nth-child(1) > div:nth-child(1) > div:nth-child(2) > div:nth-child(1) > span:nth-child(1) > div:nth-child(1) > div:nth-child(1)",
                ];
                domain = "instagram.com";
            }
            _ => {
                return Err("Invalid site".into());
            }
        }
        let wk = async |page: &Page, done: i32| -> Res<()> {
            let tag = &format!("[{tag}][{done}]");
            log!("{tag} url: {}", page.url());
            let sel = "a.earn_pages_button";

            if let Err(err) = click_el(page, sel, Some(30_000.0), Some(true)).await {
                elog!("{tag} Error clicking l4l action btn.\n{err:?}");
                if let Ok(cnt) = page.locator(sel).count().await {
                    log!("{tag} COUNT: {cnt}");
                };
                page.reload(GotoOptions::new().wait_until(WaitUntil::NetworkIdle))
                    .await?;
                return Ok(());
                // return Err(err);
            };

            log!("{tag} [wait] popup page...");
            let popup_page = wait_for_popup_page(&self.ctx, domain).await?;
            let popup_url = popup_page.url();
            log!("{tag} [popup_url] {}", popup_url);
            if !popup_url.contains("post") {
                let _ = wait_nav(&popup_page, tag).await;
                log!("{tag} bring to front...");
                let _ = popup_page.bring_to_front().await;
                log!("{tag} [find] like-btn...");

                let (like_btn_loc, _): (Locator, usize) = find_any(&popup_page,act_btn_sel, 60_000.into()).await?;
                let like_btn_cnt = like_btn_loc.count().await.unwrap();

                let like_btn_i = if like_btn_cnt > 1 { 1 } else { 0 };

                log!("{tag} [like_btn_i] {like_btn_i}");

                if let Err(err) = like_btn_loc
                    .nth(like_btn_i)
                    .click(ClickOptions::builder().force(true).build())
                    .await
                {
                    elog!("Failed to click like btn: {err:?}");
                    // sleep(200_000_000).await;
                };

                sleep(2500).await;
            } else {
                log!("{tag} Is a post!!");
            }

            log!("Closing popup page...");
            popup_page.close().await?;

            sleep(1000).await;
            log!("{tag} [click] confirm btn...");
            let sel = ".cursor.pulse-checkBox";
            if let Err(err) = click_el(page, sel, Some(30_000.0), Some(true)).await {
                if let Ok(cnt) = page.locator(sel).count().await {
                    log!("{tag} COUNT: {cnt}");
                };
                return Err(err);
            };
            sleep(1000).await;
            Ok(())
        };

        if let Err(e) = self.check_site_auth(site, tag).await {
            elog!("{tag} ERROR YT AUTH: {e}");
            return Err("".into());
        }

        brwser::goto(&page, url, tag).await?;
        if true {
            loop {
                sleep(100).await;
                wk(&page, done).await?;
                done += 1;
            }
        }

        Ok(())
    }
}
