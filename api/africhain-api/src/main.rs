use std::sync::{Arc, Mutex};

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
    blockchain: Arc<Mutex<Blockchain>>,
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

#[get("/dashboard")]
async fn dashboard(data: web::Data<AppState>) -> impl Responder {
    let blockchain = data.blockchain.lock().unwrap();
    let blocks = blockchain.chain.len();
    let pending = blockchain.pending_transactions.len();
    let valid = blockchain.is_valid();
    let latest_hash = blockchain.last_hash();

    let html = format!(
        r#"<!doctype html>
<html lang="fr">
<head>
  <meta charset="utf-8" />
  <title>AfriChain Dashboard</title>
  <style>
    body {{ font-family: Arial, sans-serif; background: #0b1220; color: #e5f5ef; margin: 0; padding: 2rem; }}
    .container {{ max-width: 900px; margin: 0 auto; }}
    h1 {{ color: #7ef5c6; }}
    .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1rem; }}
    .card {{ background: #111b2a; border: 1px solid #20314d; border-radius: 10px; padding: 1rem; }}
    label {{ display: block; margin-top: 0.8rem; }}
    input, button {{ width: 100%; padding: 0.7rem; border-radius: 8px; border: 1px solid #2c4568; margin-top: 0.3rem; }}
    button {{ background: #22c55e; color: #06210d; font-weight: bold; cursor: pointer; }}
    .small {{ color: #b3d8ca; font-size: 0.9rem; }}
  </style>
</head>
<body>
  <div class="container">
    <h1>AfriChain Dashboard</h1>
    <div class="grid">
      <div class="card"><div class="small">Blocs</div><h2>{blocks}</h2></div>
      <div class="card"><div class="small">Transactions en attente</div><h2>{pending}</h2></div>
      <div class="card"><div class="small">Validité</div><h2>{}</h2></div>
      <div class="card"><div class="small">Dernier hash</div><h2 style="font-size: 0.9rem; word-break: break-all;">{latest_hash}</h2></div>
    </div>

    <div class="card" style="margin-top: 2rem;">
      <h3>Envoyer une transaction</h3>
      <form method="post" action="/tx">
        <label>De<input name="from" value="alice" /></label>
        <label>Vers<input name="to" value="bob" /></label>
        <label>Montant<input name="amount" type="number" value="10" /></label>
        <label>Nonce<input name="nonce" type="number" value="1" /></label>
        <button type="submit">Envoyer</button>
      </form>
    </div>

    <div class="card" style="margin-top: 2rem;">
      <h3>Mine</h3>
      <form method="post" action="/mine">
        <button type="submit">Miner le bloc</button>
      </form>
    </div>
  </div>
</body>
</html>
"#,
        if valid { "OK" } else { "INVALID" }
    );

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
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
        blockchain: Arc::new(Mutex::new(Blockchain::new())),
    };

    println!("Starting AfriChain API on http://127.0.0.1:8080");
    println!("Open http://127.0.0.1:8080/dashboard");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(index)
            .service(status)
            .service(dashboard)
            .service(create_tx)
            .service(mine)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
