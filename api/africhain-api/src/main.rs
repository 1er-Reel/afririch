use africhain_core::{Blockchain, Transaction};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChainStatus {
    blocks: usize,
    pending_transactions: usize,
    valid: bool,
    latest_hash: String,
}

#[derive(Deserialize)]
struct TxRequest {
    from: String,
    to: String,
    amount: u64,
    nonce: u64,
}

#[derive(Serialize)]
struct TxResponse {
    ok: bool,
    message: String,
}

#[derive(Clone)]
struct AppState {
    blockchain: std::sync::Mutex<Blockchain>,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("AfriChain API is running")
}

#[get("/status")]
async fn status(data: web::Data<AppState>) -> impl Responder {
    let blockchain = data.blockchain.lock().unwrap();
    let response = ChainStatus {
        blocks: blockchain.chain.len(),
        pending_transactions: blockchain.pending_transactions.len(),
        valid: blockchain.is_valid(),
        latest_hash: blockchain.last_hash(),
    };

    HttpResponse::Ok().json(response)
}

#[post("/tx")]
async fn create_tx(data: web::Data<AppState>, payload: web::Json<TxRequest>) -> impl Responder {
    let mut blockchain = data.blockchain.lock().unwrap();
    blockchain.add_transaction(Transaction {
        from: payload.from.clone(),
        to: payload.to.clone(),
        amount: payload.amount,
        nonce: payload.nonce,
    });
    HttpResponse::Ok().json(TxResponse {
        ok: true,
        message: format!("Transaction queued: {} -> {} ({})", payload.from, payload.to, payload.amount),
    })
}

#[post("/mine")]
async fn mine(data: web::Data<AppState>) -> impl Responder {
    let mut blockchain = data.blockchain.lock().unwrap();
    blockchain.mine_pending_transactions("afri-node");
    HttpResponse::Ok().json(TxResponse {
        ok: true,
        message: "Pending transactions mined into a new block".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = AppState {
        blockchain: std::sync::Mutex::new(Blockchain::new()),
    };

    println!("Starting AfriChain API on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(index)
            .service(status)
            .service(create_tx)
            .service(mine)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
