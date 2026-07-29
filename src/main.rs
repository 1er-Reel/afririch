use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
    memo: String,
    timestamp: i64,
}

impl Transaction {
    fn new(from: &str, to: &str, amount: u64, memo: &str) -> Self {
        Transaction {
            from: from.to_string(),
            to: to.to_string(),
            amount,
            memo: memo.to_string(),
            timestamp: Utc::now().timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: i64,
    transactions: Vec<Transaction>,
    previous_hash: String,
    nonce: u64,
    hash: String,
}

impl Block {
    fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let mut block = Block {
            index,
            timestamp: Utc::now().timestamp(),
            transactions,
            previous_hash,
            nonce: 0,
            hash: String::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    fn calculate_hash(&self) -> String {
        let data = serde_json::to_string(&(
            self.index,
            self.timestamp,
            &self.transactions,
            &self.previous_hash,
            self.nonce,
        )).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        println!("⛏️  Mining bloc #{}", self.index);
        loop {
            self.hash = self.calculate_hash();
            if self.hash.starts_with(&target) {
                println!("✅ Miné ! Nonce: {}", self.nonce);
                break;
            }
            self.nonce += 1;
        }
    }
}

#[derive(Debug, Clone)]
struct Blockchain {
    blocks: Vec<Block>,
    difficulty: usize,
    pending_transactions: Vec<Transaction>,
    mining_reward: u64,
}

impl Blockchain {
    fn new() -> Self {
        let mut chain = Blockchain {
            blocks: Vec::new(),
            difficulty: 2,
            pending_transactions: Vec::new(),
            mining_reward: 100,
        };
        chain.blocks.push(Block::new(0, Vec::new(), "0".repeat(64)));
        chain
    }

    fn add_transaction(&mut self, tx: Transaction) {
        self.pending_transactions.push(tx);
    }

    fn mine_pending(&mut self, miner: &str) {
        self.pending_transactions.push(
            Transaction::new("SYSTEM", miner, self.mining_reward, "Récompense 💚")
        );
        let prev_hash = self.blocks.last().unwrap().hash.clone();
        let mut block = Block::new(
            self.blocks.len() as u64,
            self.pending_transactions.clone(),
            prev_hash,
        );
        block.mine(self.difficulty);
        self.blocks.push(block);
        self.pending_transactions.clear();
    }

    fn is_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let cur = &self.blocks[i];
            let prev = &self.blocks[i - 1];
            if cur.hash != cur.calculate_hash() { return false; }
            if cur.previous_hash != prev.hash { return false; }
        }
        true
    }

    fn balances(&self) -> std::collections::HashMap<String, i64> {
        let mut map = std::collections::HashMap::new();
        for block in &self.blocks {
            for tx in &block.transactions {
                *map.entry(tx.from.clone()).or_insert(0) -= tx.amount as i64;
                *map.entry(tx.to.clone()).or_insert(0) += tx.amount as i64;
            }
        }
        map
    }
}

fn html_home() -> String {
    format!(r#"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>🦁 AfriChain Explorer</title>
<style>
body {{ font-family: sans-serif; background: linear-gradient(135deg,#1a3d2e,#0d1f17); color:#f5e9d4; padding:20px; min-height:100vh; }}
header {{ text-align:center; padding:20px; border-bottom:2px solid #d4a437; margin-bottom:30px; }}
h1 {{ color:#d4a437; font-size:2.2em; }}
.cards {{ max-width:800px; margin:0 auto; }}
.card {{ background:rgba(212,164,55,0.1); border:1px solid #d4a437; border-radius:12px; padding:20px; margin:15px 0; }}
.stat {{ display:flex; justify-content:space-between; padding:8px 0; border-bottom:1px solid rgba(212,164,55,0.2); }}
.label {{ color:#a8c5a8; }}
.value {{ color:#f5e9d4; font-weight:bold; }}
a {{ color:#d4a437; }}
footer {{ text-align:center; margin-top:40px; color:#a8c5a8; }}
</style>
</head>
<body>
<header>
<h1>🦁 AfriChain Explorer</h1>
<p>La blockchain 100% africaine 💚</p>
<p style="margin-top:10px;"><a href="/blocks">📊 Blocs</a> | <a href="/balances">💰 Soldes</a> | <a href="/api/status">🔌 API</a></p>
</header>
<div class="cards">
<div class="card">
<h2>📊 Vue d'ensemble</h2>
<div class="stat"><span class="label">🦁 Nom</span><span class="value">AfriChain</span></div>
<div class="stat"><span class="label">🪙 Token</span><span class="value">AfriRich (AFR)</span></div>
<div class="stat"><span class="label">🌍 Lien</span><span class="value">Monnaie AES</span></div>
<div class="stat"><span class="label">🛡️ Statut</span><span class="value">Souveraine 💚</span></div>
</div>
</div>
<footer>🦁 Codée from scratch par Machine-senpai — L'Afrique n'a pas besoin de permission</footer>
</body>
</html>"#)
}

fn html_blocks(chain: &Blockchain) -> String {
    let mut html = String::from(r#"<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>📊 Blocs</title><style>body{font-family:sans-serif;background:linear-gradient(135deg,#1a3d2e,#0d1f17);color:#f5e9d4;padding:20px;}h1{color:#d4a437;text-align:center;}a{color:#d4a437;}.block{background:rgba(212,164,55,0.1);border:1px solid #d4a437;border-radius:12px;padding:15px;margin:15px auto;max-width:800px;}.hash{font-family:monospace;font-size:0.85em;color:#a8c5a8;word-break:break-all;}.badge{display:inline-block;background:#d4a437;color:#1a3d2e;padding:3px 10px;border-radius:12px;font-size:0.8em;margin-right:5px;}.tx{background:rgba(0,0,0,0.3);padding:8px;margin:6px 0;border-radius:6px;font-size:0.9em;}</style></head><body><h1>📊 Tous les blocs</h1><p style="text-align:center;"><a href="/">← Retour</a></p>"#);
    for block in &chain.blocks {
        html.push_str(&format!(r#"<div class="block"><h2>🧱 Bloc #{}</h2><p><span class="badge">nonce {}</span><span class="badge">{} tx</span></p><p><b>Hash:</b> <span class="hash">{}</span></p><p><b>Préc.:</b> <span class="hash">{}</span></p>"#, block.index, block.nonce, block.transactions.len(), block.hash, block.previous_hash));
        for tx in &block.transactions {
            html.push_str(&format!(r#"<div class="tx">💸 <b>{}</b> → <b>{}</b> : {} AFR <i>({})</i></div>"#, tx.from, tx.to, tx.amount, tx.memo));
        }
        html.push_str("</div>");
    }
    html.push_str("</body></html>");
    html
}

fn html_balances(chain: &Blockchain) -> String {
    let balances = chain.balances();
    let mut html = String::from(r#"<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>💰 Soldes</title><style>body{font-family:sans-serif;background:linear-gradient(135deg,#1a3d2e,#0d1f17);color:#f5e9d4;padding:20px;}h1{color:#d4a437;text-align:center;}a{color:#d4a437;}.card{background:rgba(212,164,55,0.1);border:1px solid #d4a437;border-radius:12px;padding:15px;margin:10px auto;max-width:600px;}.pos{color:#7fcf7f;font-weight:bold;font-size:1.3em;}.neg{color:#cf7f7f;}.empty{text-align:center;color:#a8c5a8;}</style></head><body><h1>💰 Soldes des wallets</h1><p style="text-align:center;"><a href="/">← Retour</a></p>"#);
    if balances.is_empty() {
        html.push_str(r#"<p class="empty">Aucun wallet pour le moment.</p>"#);
    } else {
        let mut entries: Vec<_> = balances.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        for (name, bal) in entries {
            let cls = if *bal >= 0 { "pos" } else { "neg" };
            html.push_str(&format!(r#"<div class="card">👛 <b>{}</b> : <span class="{}">{} AFR</span></div>"#, name, cls, bal));
        }
    }
    html.push_str("</body></html>");
    html
}

use actix_web::{web, App, HttpServer, HttpResponse};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🦁 AfriChain v0.3 — Démarrage");
    println!("💚 L'Afrique n'a pas besoin de permission");

    let mut chain = Blockchain::new();
    println!("🌱 Bloc Genesis créé\n");

    chain.add_transaction(Transaction::new("Alice", "Bob", 100, "Premier transfert"));
    chain.add_transaction(Transaction::new("Bob", "Charlie", 50, "Partage"));
    chain.mine_pending("mineur-senegal");

    println!();
    chain.add_transaction(Transaction::new("Charlie", "Alice", 25, "Merci!"));
    chain.mine_pending("mineur-burkina");

    println!("\n📊 Blocs : {}", chain.blocks.len());
    println!("✅ Valide : {}", chain.is_valid());

    let chain_data = web::Data::new(Mutex::new(chain));

    println!("\n🌐 Serveur sur http://localhost:8080");
    HttpServer::new(move || {
        let chain_data = chain_data.clone();
        App::new()
            .app_data(chain_data)
            .route("/", web::get().to(|d: web::Data<Mutex<Blockchain>>| async move {
                HttpResponse::Ok().content_type("text/html").body(html_home())
            }))
            .route("/blocks", web::get().to(|d: web::Data<Mutex<Blockchain>>| async move {
                let chain = d.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_blocks(&chain))
            }))
            .route("/balances", web::get().to(|d: web::Data<Mutex<Blockchain>>| async move {
                let chain = d.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_balances(&chain))
            }))
            .route("/api/blocks", web::get().to(|d: web::Data<Mutex<Blockchain>>| async move {
                let chain = d.lock().unwrap();
                HttpResponse::Ok().json(&chain.blocks)
            }))
            .route("/api/status", web::get().to(|d: web::Data<Mutex<Blockchain>>| async move {
                let chain = d.lock().unwrap();
                let json = format!(r#"{{"name":"AfriChain","blocks":{},"valid":{},"token":"AFR"}}"#, chain.blocks.len(), chain.is_valid());
                HttpResponse::Ok().content_type("application/json").body(json)
            }))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}



