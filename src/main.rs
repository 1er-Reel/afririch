use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

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

    fn balance_of(&self, addr: &str) -> i64 {
        let mut bal: i64 = 0;
        for block in &self.blocks {
            for tx in &block.transactions {
                if tx.from == addr { bal -= tx.amount as i64; }
                if tx.to == addr { bal += tx.amount as i64; }
            }
        }
        bal
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

    fn tx_history(&self, addr: &str) -> Vec<&Transaction> {
        let mut hist = Vec::new();
        for block in &self.blocks {
            for tx in &block.transactions {
                if tx.from == addr || tx.to == addr {
                    hist.push(tx);
                }
            }
        }
        hist
    }
}

fn generate_wallet_address() -> String {
    let seed = format!(
        "{}{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let hash = hex::encode(hasher.finalize());
    format!("Afri{}", &hash[..32])
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
<p style="margin-top:10px;"><a href="/blocks">📊 Blocs</a> | <a href="/balances">💰 Soldes</a> | <a href="/wallet">👛 Wallet</a> | <a href="/api/status">🔌 API</a></p>
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

fn html_wallet(new_addr: Option<&str>, check_addr: Option<&str>, chain: &Blockchain) -> String {
    let mut html = String::from(r##"<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>👛 Wallet AfriRich</title><link rel="manifest" href="/manifest.json"><meta name="theme-color" content="#1a3d2e"><meta name="apple-mobile-web-app-capable" content="yes"><meta name="apple-mobile-web-app-title" content="AfriRich"><link rel="apple-touch-icon" href="/icon.svg"><style>
body{font-family:sans-serif;background:linear-gradient(135deg,#1a3d2e,#0d1f17);color:#f5e9d4;padding:20px;}
h1{color:#d4a437;text-align:center;}
a{color:#d4a437;}
.card{background:rgba(212,164,55,0.1);border:1px solid #d4a437;border-radius:12px;padding:20px;margin:15px auto;max-width:600px;}
input,button{width:100%;padding:12px;margin:6px 0;border:1px solid #d4a437;border-radius:8px;background:rgba(0,0,0,0.3);color:#f5e9d4;font-size:1em;box-sizing:border-box;}
button{background:#d4a437;color:#1a3d2e;font-weight:bold;cursor:pointer;border:none;}
button:hover{background:#e8b547;}
.addr{font-family:monospace;font-size:1.1em;color:#7fcf7f;word-break:break-all;background:rgba(0,0,0,0.3);padding:12px;border-radius:8px;border:1px solid #d4a437;text-align:center;}
.bal{font-size:2em;color:#7fcf7f;text-align:center;font-weight:bold;}
.tx{background:rgba(0,0,0,0.3);padding:8px;margin:6px 0;border-radius:6px;font-size:0.9em;}
label{color:#a8c5a8;display:block;margin-top:8px;}
</style></head>
<body>
<h1>👛 Wallet AfriRich</h1>
<p style="text-align:center;"><a href="/">← Retour</a></p>
"##);

    // Section: Create new wallet
    html.push_str(r#"<div class="card"><h2>🆕 Créer un wallet</h2><p>Clique pour générer une nouvelle adresse AfriRich :</p><a href="/wallet/new"><button>⚡ Générer mon adresse</button></a></div>"#);

    // Show generated address
    if let Some(addr) = new_addr {
        html.push_str(&format!(r#"<div class="card"><h2>✨ Votre nouvelle adresse</h2><div class="addr">{}</div><p style="text-align:center;color:#a8c5a8;margin-top:10px;">💡 Gardez cette adresse précieusement !</p></div>"#, addr));
    }

    // Section: Check balance
    html.push_str(r#"<div class="card"><h2>💰 Voir mon solde</h2><form action="/wallet/balance" method="get"><label>Votre adresse Afri :</label><input name="addr" placeholder="Afri..." /><button type="submit">🔍 Voir le solde</button></form></div>"#);

    // Show balance result
    if let Some(addr) = check_addr {
        let bal = chain.balance_of(addr);
        let history = chain.tx_history(addr);
        html.push_str(&format!(r#"<div class="card"><h2>💰 Solde de {}</h2><div class="bal">{} AFR</div>"#, addr, bal));
        if history.is_empty() {
            html.push_str(r#"<p style="text-align:center;color:#a8c5a8;">Aucune transaction pour cette adresse.</p>"#);
        } else {
            html.push_str("<h3>📜 Historique</h3>");
            for tx in &history {
                let arrow = if tx.from == addr { "📤" } else { "📥" };
                let other = if tx.from == addr { &tx.to[..] } else { &tx.from[..] };
                let sign = if tx.from == addr { "-" } else { "+" };
                html.push_str(&format!(r#"<div class="tx">{} {} <b>{}</b> : {}{} AFR <i>({})</i></div>"#, arrow, if tx.from == addr { "→" } else { "←" }, other, sign, tx.amount, tx.memo));
            }
        }
        html.push_str("</div>");
    }

    // Section: Send AFR
    html.push_str(r#"<div class="card"><h2>💸 Envoyer des AFR</h2><form action="/wallet/send" method="post"><label>De (votre adresse) :</label><input name="from" placeholder="Afri..." /><label>À (adresse destinataire) :</label><input name="to" placeholder="Afri..." /><label>Montant (AFR) :</label><input name="amount" type="number" placeholder="50" /><label>Memo (optionnel) :</label><input name="memo" placeholder="Paiement 💚" /><button type="submit">📤 Envoyer</button></form></div>"#);

    // Section: Mine pending transactions
    html.push_str(r#"<div class="card"><h2>⛏️ Miner les transactions en attente</h2><form action="/wallet/mine" method="post"><label>Adresse du mineur :</label><input name="miner" placeholder="Afri..." /><button type="submit">⛏️ Miner !</button></form></div>"#);

    html.push_str(r#"<script>if('serviceWorker' in navigator){navigator.serviceWorker.register('/sw.js')}</script>"#);
    html.push_str("</body></html>");
    html
}

// Form structs for POST handlers
#[derive(Deserialize)]
struct SendForm {
    from: String,
    to: String,
    amount: u64,
    memo: String,
}

#[derive(Deserialize)]
struct MineForm {
    miner: String,
}

use actix_web::{web, App, HttpServer, HttpResponse};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🦁 AfriChain v0.4 — Wallet Edition");
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
    println!("👛 Wallet sur http://localhost:8080/wallet");
    HttpServer::new(move || {
        let chain_data = chain_data.clone();
        App::new()
            .app_data(chain_data)
            .route("/", web::get().to(|_d: web::Data<Mutex<Blockchain>>| async move {
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
            .route("/wallet", web::get().to(|d: web::Data<Mutex<Blockchain>>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let chain = d.lock().unwrap();
                let new_addr = q.get("new").map(|s| s.as_str());
                let check_addr = q.get("addr").map(|s| s.as_str());
                HttpResponse::Ok().content_type("text/html").body(html_wallet(new_addr, check_addr, &chain))
            }))
            .route("/wallet/new", web::get().to(|| async move {
                let addr = generate_wallet_address();
                HttpResponse::Found()
                    .append_header(("Location", format!("/wallet?new={}", addr)))
                    .finish()
            }))
            .route("/wallet/balance", web::get().to(|_d: web::Data<Mutex<Blockchain>>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let addr = q.get("addr").cloned().unwrap_or_default();
                HttpResponse::Found()
                    .append_header(("Location", format!("/wallet?addr={}", addr)))
                    .finish()
            }))
            .route("/wallet/send", web::post().to(|d: web::Data<Mutex<Blockchain>>, form: web::Form<SendForm>| async move {
                let tx = Transaction::new(&form.from, &form.to, form.amount, &form.memo);
                let mut chain = d.lock().unwrap();
                chain.add_transaction(tx);
                println!("💸 Transaction en attente : {} → {} ({} AFR)", form.from, form.to, form.amount);
                HttpResponse::Found()
                    .append_header(("Location", "/wallet"))
                    .finish()
            }))
            .route("/wallet/mine", web::post().to(|d: web::Data<Mutex<Blockchain>>, form: web::Form<MineForm>| async move {
                let mut chain = d.lock().unwrap();
                chain.mine_pending(&form.miner);
                println!("⛏️ Bloc miné pour {}", form.miner);
                HttpResponse::Found()
                    .append_header(("Location", "/wallet"))
                    .finish()
            }))
            .route("/api/blocks", web::get().to(|d: web::Data<Mutex<Blockchain>>| async move {
                let chain = d.lock().unwrap();
                HttpResponse::Ok().json(&chain.blocks)
            }))
            .route("/api/status", web::get().to(|d: web::Data<Mutex<Blockchain>>| async move {
                let chain = d.lock().unwrap();
                let json = format!(r#"{{"name":"AfriChain","blocks":{},"valid":{},"token":"AFR","version":"0.4"}}"#, chain.blocks.len(), chain.is_valid());
                HttpResponse::Ok().content_type("application/json").body(json)
            }))
            .route("/manifest.json", web::get().to(|| async move {
                let manifest = r##"{"name":"AfriRich Wallet","short_name":"AfriRich","description":"🦁 Wallet AfriChain — La crypto 100% africaine","start_url":"/wallet","display":"standalone","background_color":"#0d1f17","theme_color":"#1a3d2e","icons":[{"src":"/icon.svg","sizes":"any","type":"image/svg+xml","purpose":"any maskable"}]}"##;
                HttpResponse::Ok().content_type("application/json").body(manifest)
            }))
            .route("/icon.svg", web::get().to(|| async move {
                let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><rect width="512" height="512" rx="80" fill="#1a3d2e"/><text x="256" y="360" font-size="320" text-anchor="middle">🦁</text></svg>"##;
                HttpResponse::Ok().content_type("image/svg+xml").body(svg)
            }))
            .route("/sw.js", web::get().to(|| async move {
                let sw = "const C='afri-v0.4';self.addEventListener('install',e=>{e.waitUntil(caches.open(C).then(c=>c.addAll(['/wallet','/manifest.json','/icon.svg'])))});self.addEventListener('fetch',e=>{e.respondWith(caches.match(e.request).then(r=>r||fetch(e.request)))});";
                HttpResponse::Ok().content_type("application/javascript").body(sw)
            }))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
