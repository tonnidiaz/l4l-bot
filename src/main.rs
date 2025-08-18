mod bot;
mod funcs;
mod server;
mod types;

#[tokio::main]
async fn main() {
    log!("Hello, world!");
    dotenv::dotenv().ok();
    tokio::spawn(async move {
        if let Err(err) = bot::main().await {
            log!("[bot_err] {err:?}");
        }
    });

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
            println!("\n[{}] {}", now.format("%Y-%m-%d %H:%M:%S"), msg);
        }
    }};
}
