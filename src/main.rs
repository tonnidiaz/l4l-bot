mod bot;
mod brwser;
mod consts;
mod pw;

use std::{env::args, thread::park};
use turs::log;

use crate::consts::Const;

#[tokio::main]
async fn main() {
    log!("Hello, world!");
    let _args = args().collect::<Vec<_>>();
    let headless = !_args.contains(&"-h".into());
    log!("Headless = {headless}");

    Const::new(Const { headless });
    let _bm = brwser::BrMan::new()
        .await
        .expect("Failed to initialize BrowserManager");
    bot::main().await.expect("Failed to run bot");
    park();
    _bm.close().await.expect("Failed to close BrowserManager");
}
