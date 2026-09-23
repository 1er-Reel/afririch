use std::sync::{Arc, Mutex};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

use africhain_core::{Blockchain, Transaction, Wallet};

#[derive(Serialize)]
struct BalanceResponse {
    address: String,
    balance: i64,
}

#[derive(Serialize)]
struct TransactionsResponse {
    transactions: Vec<Transaction>,
}

#[derive(Deserialize)]
struct TxRequest {
    from: String,
    to: String,
    amount: i64,
    nonce: u64,
    note: String,
}

#[derive(Serialize)]
struct TxResponse {
    ok: bool,
    message: String,
}

#[derive(Clone)]
struct AppState {
    blockchain: Arc<Mutex<Blockchain>>,
    wallet: Arc<Mutex<Wallet>>,
}

#[get("/balance")]
async fn balance(data: web::Data<AppState>) -> impl Responder {
    let wallet = data.wallet.lock().unwrap();
    HttpResponse::Ok().json(BalanceResponse {
        address: wallet.address.clone(),
        balance: wallet.balance(),
    })
}

#[get("/transactions")]
async fn transactions(data: web::Data<AppState>) -> impl Responder {
    let wallet = data.wallet.lock().unwrap();
    HttpResponse::Ok().json(TransactionsResponse {
        transactions: wallet.transactions.clone(),
    })
}

#[post("/tx")]
async fn create_tx(data: web::Data<AppState>, payload: web::Json<TxRequest>) -> impl Responder {
    let mut wallet = data.wallet.lock().unwrap();
    let tx = Transaction {
        from: payload.from.clone(),
        to: payload.to.clone(),
        amount: payload.amount,
        nonce: payload.nonce,
        note: payload.note.clone(),
    };

    wallet.add_transaction(tx.clone());

    let mut blockchain = data.blockchain.lock().unwrap();
    blockchain.add_transaction(tx);

    HttpResponse::Ok().json(TxResponse {
        ok: true,
        message: format!("Transaction recorded: {} -> {} ({})", payload.from, payload.to, payload.amount),
    })
}

#[get("/dashboard")]
async fn dashboard() -> impl Responder {
    HttpResponse::Ok().body(
        r#"<!doctype html>
<html lang="fr">
<head><meta charset="utf-8"><title>AfriChain Wallet</title></head>
<body>
  <h1>AfriChain Wallet</h1>
  <p>Balance: <span id="balance">chargement...</span></p>
  <ul id="transactions"></ul>
  <script>
    async function load() {
      const b = await fetch('/balance');
      const balance = await b.json();
      document.getElementById('balance').textContent = balance.balance;

      const t = await fetch('/transactions');
      const txs = await t.json();
      const list = document.getElementById('transactions');
      list.innerHTML = '';
      txs.transactions.forEach(tx => {
        const li = document.createElement('li');
        li.textContent = tx.from + ' -> ' + tx.to + ' : ' + tx.amount + ' | ' + tx.note;
        list.appendChild(li);
      });
    }
    setInterval(load, 1000);
    load();
  </script>
</body>
</html>"#,
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let wallet = Wallet::new("afri-wallet-001");
    let state = AppState {
        blockchain: Arc::new(Mutex::new(Blockchain::new())),
        wallet: Arc::new(Mutex::new(wallet)),
    };

    println!("AfriChain wallet API on http://127.0.0.1:8080");
    println!("Open http://127.0.0.1:8080/dashboard");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(balance)
            .service(transactions)
            .service(create_tx)
            .service(dashboard)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
