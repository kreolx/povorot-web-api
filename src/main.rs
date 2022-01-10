use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json,Router
};
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr};

const REDIS_CON_STRING: &str = "redis://127.0.0.1/";

#[tokio::main]
async fn main() {
    let app = Router::new()
    .route("/price", get(prices));
    //.route("/save", get(all_slots))
    //.route("save", post(create_save));

    let addr = SocketAddr::from(([127,0,0,1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();
}

async fn prices() -> impl IntoResponse {
    let client = redis::Client::open(REDIS_CON_STRING).unwrap();
    let mut con = client.get_connection().unwrap();
    let prices_str: Vec<String> = con.smembers("prices").unwrap();
    let mut prices: Vec<Price> = Vec::new();
    for (i, val) in prices_str.iter().enumerate().step_by(2) {
        let cost: u32 = prices_str[i+1].parse().unwrap();
        let price = Price {
            name: val.to_string(),
            cost: cost
        };
        prices.push(price);
    }
    (StatusCode::OK, Json(prices))
}


#[derive(Serialize)]
struct Price {
    name: String,
    cost: u32,
}
