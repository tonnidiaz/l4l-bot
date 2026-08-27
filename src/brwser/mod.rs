mod funcs;
pub use funcs::{*};

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use once_cell::sync::OnceCell;
use playwright_rs::{Browser, LaunchOptions, Playwright, Viewport, install_browsers};
use turs::{Res, log};

use crate::{brwser::funcs::gen_fingerprint, consts::Const};

static BM: OnceCell<Arc<BrMan>> = OnceCell::new();

pub struct BrMan {
    pub browsers: Mutex<Vec<Browser>>,
    pw: playwright_rs::Playwright,
    is_running: AtomicBool,
}

impl BrMan {
    pub async fn new() -> Res<Arc<Self>> {
        if let Some(bm) = BM.get() {
            if bm.is_running.load(Ordering::Relaxed) {
                log!("BM ALREADY RUNNING");
                return Ok(bm.clone());
            }
        }
        // install_browsers(Some(&["chromium", "firefox"])).await?;
        log!("Initializing BrowserManager...");

        let consts = Const::get();
        let pw = match Playwright::launch().await {
            Ok(pw) => pw,

            Err(err) => {
                let err_str = err.to_string();
                log!("Playwright launch error: {}", err_str);

                if matches!(err, playwright_rs::Error::BrowserNotInstalled { .. }) {
                    log!("Browsers not installed. Installing...");

                    install_browsers(Some(&["chromium", "firefox"]))
                        .await
                        .map_err(|install_err| {
                            log!("Failed to install browsers: {}", install_err);
                            install_err
                        })?;

                    log!("Browsers installed successfully.");

                    // Try launching Playwright again
                    Playwright::launch().await?
                } else {
                    return Err(err.into());
                }
            }
        };
        let browsers = vec![pw.firefox().clone()];

        let bm = Arc::new(Self {
            browsers: Mutex::new(vec![]),
            pw,
            is_running: AtomicBool::new(false),
        });

        for (_i, b) in browsers.iter().enumerate() {
            let opts = LaunchOptions::new()
                .args(vec![
                    "--no-sandbox".into(),
                    "--disable-setuid-sandbox".into(),
                    "--disable-blink-features=AutomationControlled".into(),
                    "--mute-audio".into(),
                ])
                .headless(consts.headless);
            let brwser = b.launch_with_options(opts).await?;
            bm.browsers.lock().unwrap().push(brwser);
        }

        bm.is_running.store(true, Ordering::Relaxed);
        if let Err(existing) = BM.set(bm.clone()) {
            log!("BM ALREADY INITIALIZED");
            return Ok(existing);
        }

        log!("BrMan initialized");
        Ok(bm)
    }

    pub async fn new_page(
        tag: &str,
        vp: Option<Viewport>,
        i: Option<usize>,
    ) -> Res<(playwright_rs::BrowserContext, playwright_rs::Page)> {
        let bm = BM.get().expect("BrowserManager not initialized");
        let browser_index = i.unwrap_or(0);
        let mut browsers = bm.browsers.lock().unwrap();
        let browser = browsers
            .get_mut(browser_index)
            .expect("Browser index out of bounds");

        let fprint = gen_fingerprint(vp);

        for ctx in browser.contexts().iter_mut() {
            let _ = ctx.close().await;
        }

        let context = browser
            .new_context_with_options(fprint)
            .await
            .expect("Failed to create new context");
        context
            .clear_cookies(None)
            .await
            .expect("Failed to clear cookies");

        let page = context.new_page().await.expect("Failed to create new page");
        add_listeners(&page, tag).await?;

        Ok((context, page))
    }

    pub async fn close(&self) -> Res<()> {
        let mut browsers = self.browsers.lock().unwrap();
        for browser in browsers.iter_mut() {
            browser.close().await?;
        }
        self.is_running.store(false, Ordering::Relaxed);
        self.pw.shutdown().await?;
        Ok(())
    }
}

async fn add_listeners(page: &playwright_rs::Page, tag: &str) -> Res<()> {
    let tag = tag.to_string();
    log!("{tag} [add_listeners....]");

    page.on_dialog(move |dlg| {
        let tag = tag.clone();
        async move {
            println!("");
            log!("{tag} [dialog] {}", dlg.message());
            if let Err(err) = dlg.dismiss().await {
                log!("{tag} [dialog] dismiss error: {}", err);
            }
            Ok(())
        }
    })
    .await?;
    Ok(())
}
