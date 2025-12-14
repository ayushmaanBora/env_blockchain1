use axum::{
    routing::{get, post},
    Router, Json, extract::State,
};
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use tower_http::cors::CorsLayer;

pub struct AppState {
    pub blockchain: Arc<Mutex<Blockchain>>,
}

pub async fn start_api_server(blockchain: Arc<Mutex<Blockchain>>) {
    let state = Arc::new(AppState { blockchain });

    let app = Router::new()
        .route("/chain", get(get_chain))
        .route("/wallets", get(get_wallets))
        .route("/submit", post(submit_task_api))
        .layer(CorsLayer::permissive()) 
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    println!("🚀 API Server running on http://0.0.0.0:3030");
    axum::serve(listener, app).await.unwrap();
}

async fn get_chain(State(state): State<Arc<AppState>>) -> Json<Vec<crate::blockchain::Block>> {
    let chain = state.blockchain.lock().unwrap().chain.clone();
    Json(chain)
}

async fn get_wallets(State(state): State<Arc<AppState>>) -> Json<Vec<crate::wallet::Wallet>> {
    let wallets = state.blockchain.lock().unwrap().wallets.get_all_wallets();
    Json(wallets)
}

#[derive(serde::Deserialize)]
struct SubmitRequest {
    wallet: String,
    task_name: String,
    metadata: String,
}

async fn submit_task_api(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SubmitRequest>,
) -> Json<String> {
    let mut bc = state.blockchain.lock().unwrap();
    
    // FIX: Changed function name to match the new Industrial logic
    if let Some(_) = bc.submit_industrial_task(&payload.wallet, payload.task_name, payload.metadata) {
        Json("Task Submitted successfully".to_string())
    } else {
        Json("Submission failed".to_string())
    }
}