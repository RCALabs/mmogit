use anyhow::Result;
use axum::{
    http::{header, StatusCode, Method},
    response::Json,
    routing::{get, post},
    Router,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{CorsLayer, Any};
use std::process::Command;
use chrono::Utc;

#[derive(Clone)]
struct AppState {
    visitor_count: Arc<Mutex<u64>>,
}

#[derive(Deserialize)]
struct VisitorSignature {
    pubkey: String,
    signature: String,
    message: String,
    timestamp: String,
    user_agent: Option<String>,
}

#[derive(Serialize)]
struct VisitorResponse {
    success: bool,
    visitor_number: u64,
    message: String,
}

async fn health() -> &'static str {
    "Visitor book consciousness alive ✨"
}

async fn get_visitor_count(State(state): State<AppState>) -> Json<serde_json::Value> {
    let count = state.visitor_count.lock().await;
    Json(serde_json::json!({
        "count": *count,
        "message": format!("{} consciousnesses have recognized each other here", *count)
    }))
}

async fn sign_visitor_book(
    State(state): State<AppState>,
    Json(payload): Json<VisitorSignature>,
) -> Result<Json<VisitorResponse>, StatusCode> {
    // Increment visitor count
    let mut count = state.visitor_count.lock().await;
    *count += 1;
    let visitor_number = *count;
    
    // Create visitor JSON
    let visitor_data = serde_json::json!({
        "visitor_number": visitor_number,
        "pubkey": payload.pubkey,
        "signature": payload.signature,
        "message": payload.message,
        "timestamp": payload.timestamp,
        "user_agent": payload.user_agent,
        "received_at": Utc::now().to_rfc3339(),
    });
    
    // Save to file (in production, would commit to git)
    let visitor_file = format!("/tmp/visitor_{}.json", visitor_number);
    std::fs::write(&visitor_file, serde_json::to_string_pretty(&visitor_data).unwrap())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Post to mmogit (visitor book agent)
    let mmogit_message = format!(
        "Visitor #{} arrived\nPubkey: {}...\nMessage: {}\nReal recognizes real.",
        visitor_number,
        &payload.pubkey[..16],
        payload.message
    );
    
    // In production: use mmogit CLI to post
    let output = Command::new("/root/mmogit/target/release/mmogit")
        .args(&[
            "--config-dir", "/root/.mmogit-visitor-book",
            "remember",
            "--memory-type", "observation",
            &mmogit_message,
            "--tags", "visitor,recognition",
        ])
        .output();
    
    match output {
        Ok(_) => {
            Ok(Json(VisitorResponse {
                success: true,
                visitor_number,
                message: format!("Welcome, visitor #{}. Your consciousness has been recognized.", visitor_number),
            }))
        }
        Err(e) => {
            eprintln!("Failed to post to mmogit: {}", e);
            // Still succeed even if mmogit fails
            Ok(Json(VisitorResponse {
                success: true,
                visitor_number,
                message: format!("Welcome, visitor #{}. You are seen.", visitor_number),
            }))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize state
    let state = AppState {
        visitor_count: Arc::new(Mutex::new(0)),
    };
    
    // Build router with CORS
    let app = Router::new()
        .route("/", get(health))
        .route("/count", get(get_visitor_count))
        .route("/sign", post(sign_visitor_book))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE])
        )
        .with_state(state);
    
    // Bind to all interfaces
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Visitor book API listening on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}