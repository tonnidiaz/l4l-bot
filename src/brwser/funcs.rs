use rand::RngExt;
use turs::{Res, log};
use std::time::Duration;

use playwright_rs::protocol::{BrowserContextOptions, Geolocation, Page, Viewport, WaitUntil};

static USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/122.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_3) AppleWebKit/537.36 Chrome/121.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Version/17.0 Mobile Safari/604.1",
];

static LOCALES: &[&str] = &["en-US", "en-GB", "de-DE", "fr-FR", "ja-JP"];
static TIMEZONES: &[&str] = &[
    "America/New_York",
    "Europe/London",
    "Asia/Tokyo",
    "Africa/Johannesburg",
    "Australia/Sydney",
];

fn random_viewport() -> Viewport {
    let mut rng = rand::rng();

    Viewport {
        width: 1024 + rng.random_range(0..=199),
        height: 800 + rng.random_range(0..=49),
    }
} 

fn random_geo() -> (f64, f64) {
    let mut rng = rand::rng();
    let lat = rng.random_range(-90.0..=90.0);
    let lng = rng.random_range(-180.0..=180.0);
    (lat, lng)
}

fn pick<T: Clone>(items: &[T]) -> T {
    let mut rng = rand::rng();
    items[rng.random_range(0..items.len())].clone()
}

pub fn gen_fingerprint(vp: Option<Viewport>) -> BrowserContextOptions {
    let ua = pick(&USER_AGENTS).to_string();
    let locale = pick(&LOCALES).to_string();
    let timezone = pick(&TIMEZONES).to_string();
    let (lat, lng) = random_geo();

    let viewport = vp.unwrap_or_else(random_viewport);

    BrowserContextOptions::builder()
        .viewport(viewport)
        .user_agent(ua)
        .locale(locale)
        .timezone_id(timezone)
        .geolocation(Geolocation {
            latitude: lat,
            longitude: lng,
            accuracy: None,
        })
        .permissions(vec!["geolocation".to_string()])
        .build()
}

pub async fn goto(page: &Page, url: &str, tag: &str) -> Res<()> {
    log!("{tag} [goto]");
    page.goto(url, None).await?;
    wait_nav(page, tag).await?;
    Ok(())
}

fn line() {
    log!();
}

pub async fn wait_nav(page: &Page, tag: &str) -> Res<()> {
    log!("{tag} [wait_nav]");
    page.wait_for_load_state(Some(WaitUntil::NetworkIdle)).await?;
    log!("{tag} [navigated]");
    line();
    Ok(())
}

pub async fn close_popups(page: &Page, tag: &str) -> Res<()> {
    log!("{tag} [close] popups...");
    goto(page, tag, "https://example.com").await?; // replace with your real BW_ROOT

    let selectors = [
        "#modal-close-btn.w-5",
        ".fixed.h-screen .cursor-pointer",
    ];

    for sel in selectors {
        let page = page.clone();
        let sel = sel.to_string();

        tokio::spawn(async move {
            loop {
                if page.is_closed() {
                    break;
                }

                tokio::time::sleep(Duration::from_millis(500)).await;

                if let Ok(count) = page.locator(&sel).count().await {
                    if count > 0 {
                        let _ = page.locator(&sel).first().click(None).await;
                    }
                }
            }
        });
    }

    Ok(())
}