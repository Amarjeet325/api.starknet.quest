mod config;
mod endpoints;
mod models;
mod utils;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use mongodb::{bson::doc, options::ClientOptions, Client};
use std::net::SocketAddr;
use std::sync::Arc;

use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    println!("quest-server: starting v{}", env!("CARGO_PKG_VERSION"));
    let conf = config::load();
    let client_options = ClientOptions::parse(&conf.database.connection_string)
        .await
        .unwrap();

    let shared_state = Arc::new(models::AppState {
        conf: conf.clone(),
        db: Client::with_options(client_options)
            .unwrap()
            .database(&conf.database.name),
    });
    if shared_state
        .db
        .run_command(doc! {"ping": 1}, None)
        .await
        .is_err()
    {
        println!("error: unable to connect to database");
        return;
    } else {
        println!("database: connected")
    }

    let cors = CorsLayer::new().allow_headers(Any).allow_origin(Any);
    let app = Router::new()
        .route("/", get(root))
        .route("/quests/uri", get(endpoints::quests::uri::handler))
        .route("/get_quest", get(endpoints::get_quest::handler))
        .route("/get_quests", get(endpoints::get_quests::handler))
        .route("/get_tasks", get(endpoints::get_tasks::handler))
        .route(
            "/quests/starkfighter/verify_has_played",
            get(endpoints::quests::starkfighter::verify_has_played::handler),
        )
        .route(
            "/quests/starkfighter/verify_has_score_greater_than_50",
            get(endpoints::quests::starkfighter::verify_has_score_greater_than_50::handler),
        )
        .route(
            "/quests/starkfighter/verify_has_score_greater_than_100",
            get(endpoints::quests::starkfighter::verify_has_score_greater_than_100::handler),
        )
        .route(
            "/quests/starknetid/verify_has_domain",
            get(endpoints::quests::starknetid::verify_has_domain::handler),
        )
        .with_state(shared_state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], conf.server.port));
    println!("server: listening on http://0.0.0.0:{}", conf.server.port);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn root() -> (StatusCode, String) {
    (
        StatusCode::ACCEPTED,
        format!("quest-server v{}", env!("CARGO_PKG_VERSION")),
    )
}