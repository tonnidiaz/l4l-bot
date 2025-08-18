
use std::{collections::HashMap, env, time::Instant};

use axum::{
    body::Body,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
};

use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::log;

async fn logging_middleware(req: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path_and_query().unwrap().to_string();

    let response = next.run(req).await;
    let duration = start.elapsed(); 

    if !path.contains("socket.io") {
        println!(
            "\n{} {} {} took {:.2?}",
            method,
            response.status().as_u16(),
            path,
            duration
        );
    }
    response
}

pub async fn main() -> axum::Router {
    // let origins = ["http://localhost:3000".parse().unwrap()];
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(["GET".parse().unwrap(), "POST".parse().unwrap()])
        // .allow_headers(cors::Any)
        .allow_credentials(false);

    let _svc = ServiceBuilder::new()
        .layer(&cors)
        .layer(TraceLayer::new_for_http());
    let app = axum::Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(cors)
        .layer(middleware::from_fn(logging_middleware));

    log!("Starting server");
        let vars: HashMap<_, _> = env::vars().collect();
        let port = match vars.get("PORT") {
            Some(p) => p.parse().unwrap(),
            None => 8000,
        };

        let addr = format!("0.0.0.0:{port}");
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        log!("Listening on: {addr:?}");
        axum::serve(listener, app.clone()).await.unwrap();
    app
}
