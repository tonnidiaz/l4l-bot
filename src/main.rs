mod bot;
mod funcs;
mod server;
mod types;
use tokio::time;
use futures::TryFutureExt;
use std::env;

#[tokio::main]
async fn main() {
    log!("Hello, world!");
    dotenv::dotenv().ok();
    tokio::spawn(async move {
        if let Err(err) = bot::main().await {
            log!("[bot_err] {err:?}");
        }
    });
	tokio::task::spawn(async{poll_me().await});
    let _ = server::main().await;
}

#[macro_export]
macro_rules! log {

    () => {
        println!();
    };
    ($($arg:expr),*) => {{
        #[allow(unused_macros)]
        {
            let now = chrono::Local::now();
            let msg = format!($($arg),*);
            println!("[{}] {}", now.format("%Y-%m-%d %H:%M:%S"), msg);
        }
    }};
}

async fn poll_me() {
    log!("[poll_me]");
    let m = 60;
    if let Ok(origin) = env::var("ORIGIN") {
        loop {
            time::sleep(time::Duration::from_secs(2 * m)).await;
            let res = reqwest::get(&format!("{}/", origin))
            .and_then(|res| async move{res.text().await}).await;
            log!("[poll_me] res: {res:?}");
        }
    }
}
