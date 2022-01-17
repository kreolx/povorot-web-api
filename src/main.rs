use axum::{
    http::{StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json,Router
};

use http::{Method};
use redis::{Commands,RedisError};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr,env};
use chrono::{DateTime};
use lapin::{
    options::*, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties,
};
use tower_http::cors::{CorsLayer, Origin};

const REDIS_CON_STRING: &str = "REDIS_CON_STRING";
const RABBIT_CON_STRING: &str = "RABBIT_CON_STRING";

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Origin::exact("http://povorot27.ru/".parse().unwrap()))
        .allow_methods(vec![Method::GET, Method::POST]);

    let app = Router::new()
    .route("/price", get(prices)).layer(&cors)
    .route("/requests", get(empty_slots)).layer(&cors)
    .route("/requests", post(create_save_request)).layer(&cors);

    let addr = SocketAddr::from(([0,0,0,0], 8080));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();
}

fn connect() -> Result<redis::Connection, RedisError> {
    let con_str = env::var(REDIS_CON_STRING).unwrap_or_else(|_| "redis://127.0.0.1/".into());
    let client = redis::Client::open(con_str)?;
    Ok(client.get_connection()?)
}

async fn prices() -> impl IntoResponse {
    let mut con = connect().unwrap();
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

async fn empty_slots(Json(payload): Json<EmptySlotRequest>) -> impl IntoResponse {
    let date = DateTime::parse_from_rfc3339(&payload.date).unwrap();
    let mut con = connect().unwrap();
    let slots: Vec<String> = con.lrange(date.format("%d.%m.%Y").to_string(), 0, -1).unwrap();
    (StatusCode::OK, Json(slots))
}

async fn create_save_request(Json(payload): Json<SaveRequest>) -> impl IntoResponse {
    let rabbit_con_str = env::var(RABBIT_CON_STRING).unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".into());
    let conn = Connection::connect(&rabbit_con_str, ConnectionProperties::default().with_default_executor(8))
    .await
    .unwrap();
    let channel = conn.create_channel().await.unwrap();
    let _queue = channel.queue_declare("save-requests", QueueDeclareOptions::default(),
        FieldTable::default())
        .await
        .unwrap();

    let js = serde_json::to_string(&payload).unwrap();
    let _confirm = channel.basic_publish("", "save-requests",
    BasicPublishOptions::default(),
    js.as_bytes().to_vec(),
    BasicProperties::default(),
    ).await
    .expect("basic publish")
    .await
    .expect("publisher confirm");
    (StatusCode::CREATED, Json("OK"))
}

#[derive(Deserialize, Serialize)]
struct  SaveRequest {
    date: String,
    phone: String,
    car: String,
    description: String,
}

#[derive(Deserialize)]
struct  EmptySlotRequest {
    date: String,
}

#[derive(Serialize)]
struct Price {
    name: String,
    cost: u32,
}
