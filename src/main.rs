use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;
use std::sync::Mutex;

use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Verifier, Signature};
use rand::rngs::OsRng;

// ===== TRANSACTION =====
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
    memo: String,
    timestamp: i64,
    #[serde(default)]
    signature: String,
}

impl Transaction {
    fn new(from: &str, to: &str, amount: u64, memo: &str) -> Self {
        Transaction {
            from: from.to_string(),
            to: to.to_string(),
            amount,
            memo: memo.to_string(),
            timestamp: Utc::now().timestamp(),
            signature: String::new(),
        }
    }

    fn sign_data(&self) -> Vec<u8> {
        let data = serde_json::to_string(&(
            &self.from,
            &self.to,
            self.amount,
            &self.memo,
            self.timestamp,
        )).unwrap();
        data.into_bytes()
    }

    fn sign(&mut self, signing_key: &SigningKey) {
        let sig: Signature = signing_key.sign(&self.sign_data());
        self.signature = hex::encode(sig.to_bytes());
    }

    fn verify(&self) -> bool {
        if self.from == "SYSTEM" { return true; }
        if self.signature.is_empty() { return true; }
        let addr_hex = match self.from.strip_prefix("Afri") {
            Some(h) => h,
            None => return true,
        };
        let pub_bytes = match hex::decode(addr_hex) {
            Ok(b) if b.len() == 32 => b,
            _ => return true,
        };
        let pub_arr: [u8; 32] = pub_bytes.try_into().unwrap();
        let verifying_key = match VerifyingKey::from_bytes(&pub_arr) {
            Ok(vk) => vk,
            Err(_) => return false,
        };
        let sig_bytes = match hex::decode(&self.signature) {
            Ok(b) if b.len() == 64 => b,
            _ => return false,
        };
        let sig_arr: [u8; 64] = sig_bytes.try_into().unwrap();
        let signature = Signature::from_bytes(&sig_arr);
        verifying_key.verify(&self.sign_data(), &signature).is_ok()
    }
}

// ===== BLOCK =====
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

// ===== BLOCKCHAIN =====
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        for tx in &self.pending_transactions {
            if !tx.verify() {
                println!("⚠️  Transaction invalide rejetée : {} → {}", tx.from, tx.to);
                return;
            }
        }
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
        self.save_to_file();
    }

    fn save_to_file(&self) {
        let data = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write("blockchain.json", data).ok();
        println!("💾 Blockchain sauvegardée");
    }

    fn load_from_file() -> Option<Self> {
        match std::fs::read_to_string("blockchain.json") {
            Ok(data) => {
                println!("📂 Blockchain chargée depuis le fichier");
                serde_json::from_str(&data).ok()
            }
            Err(_) => None
        }
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

    fn total_transactions(&self) -> usize {
        self.blocks.iter().map(|b| b.transactions.len()).sum()
    }

    fn total_supply(&self) -> i64 {
        self.balances().values().filter(|&&v| v > 0).sum()
    }
}

// ===== WALLET STORE =====
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalletStore {
    wallets: std::collections::HashMap<String, String>,
}

impl WalletStore {
    fn load() -> Self {
        match std::fs::read_to_string("wallets.json") {
            Ok(data) => serde_json::from_str(&data).unwrap_or(WalletStore { wallets: std::collections::HashMap::new() }),
            Err(_) => WalletStore { wallets: std::collections::HashMap::new() },
        }
    }

    fn save(&self) {
        let data = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write("wallets.json", data).ok();
    }

    fn create_wallet(&mut self) -> (String, String) {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let address = format!("Afri{}", hex::encode(verifying_key.to_bytes()));
        let private_key = hex::encode(signing_key.to_bytes());
        self.wallets.insert(address.clone(), private_key.clone());
        self.save();
        (address, private_key)
    }

    fn get_signing_key(&self, address: &str) -> Option<SigningKey> {
        let pk_hex = self.wallets.get(address)?;
        let pk_bytes = hex::decode(pk_hex).ok()?;
        let arr: [u8; 32] = pk_bytes.try_into().ok()?;
        Some(SigningKey::from_bytes(&arr))
    }
}

// ===== USER STORE =====
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserAccount {
    username: String,
    password_hash: String,
    address: String,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserStore {
    users: Vec<UserAccount>,
}

impl UserStore {
    fn load() -> Self {
        match std::fs::read_to_string("users.json") {
            Ok(data) => serde_json::from_str(&data).unwrap_or(UserStore { users: Vec::new() }),
            Err(_) => UserStore { users: Vec::new() },
        }
    }

    fn save(&self) {
        let data = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write("users.json", data).ok();
    }

    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("afririch_salt_{}", password).as_bytes());
        hex::encode(hasher.finalize())
    }

    fn register(&mut self, username: &str, password: &str, wallets: &mut WalletStore) -> Result<UserAccount, String> {
        if self.users.iter().any(|u| u.username == username) {
            return Err("Ce nom d'utilisateur existe déjà".to_string());
        }
        if username.len() < 3 {
            return Err("Le nom doit faire au moins 3 caractères".to_string());
        }
        if password.len() < 4 {
            return Err("Le mot de passe doit faire au moins 4 caractères".to_string());
        }
        let (address, _priv) = wallets.create_wallet();
        let user = UserAccount {
            username: username.to_string(),
            password_hash: Self::hash_password(password),
            address,
            created_at: Utc::now().timestamp(),
        };
        println!("🆕 Utilisateur inscrit : {} → {}", user.username, user.address);
        self.users.push(user.clone());
        self.save();
        Ok(user)
    }

    fn login(&self, username: &str, password: &str) -> Option<&UserAccount> {
        let hash = Self::hash_password(password);
        self.users.iter().find(|u| u.username == username && u.password_hash == hash)
    }

    fn count(&self) -> usize {
        self.users.len()
    }
}

// ===== ADMIN PASSWORD =====
const ADMIN_PASSWORD: &str = "africhain2026";

// ===== HTML: SHARED STYLE =====
const STYLE: &str = r##"<style>body{font-family:sans-serif;background:linear-gradient(135deg,#1a3d2e,#0d1f17);color:#f5e9d4;padding:20px;margin:0;}h1{color:#d4a437;text-align:center;}a{color:#d4a437;}.card{background:rgba(212,164,55,0.1);border:1px solid #d4a437;border-radius:12px;padding:20px;margin:15px auto;max-width:600px;}input,button{width:100%;padding:12px;margin:6px 0;border:1px solid #d4a437;border-radius:8px;background:rgba(0,0,0,0.3);color:#f5e9d4;font-size:1em;box-sizing:border-box;}button{background:#d4a437;color:#1a3d2e;font-weight:bold;cursor:pointer;border:none;}button:hover{background:#e8b547;}.addr{font-family:monospace;font-size:1.1em;color:#7fcf7f;word-break:break-all;background:rgba(0,0,0,0.3);padding:12px;border-radius:8px;border:1px solid #d4a437;text-align:center;}.priv{font-family:monospace;font-size:0.9em;color:#cf7f7f;word-break:break-all;background:rgba(0,0,0,0.3);padding:12px;border-radius:8px;border:1px solid #cf7f7f;text-align:center;}.bal{font-size:2em;color:#7fcf7f;text-align:center;font-weight:bold;}.tx{background:rgba(0,0,0,0.3);padding:8px;margin:6px 0;border-radius:6px;font-size:0.9em;}label{color:#a8c5a8;display:block;margin-top:8px;}.msg{background:rgba(127,207,127,0.2);border:1px solid #7fcf7f;border-radius:8px;padding:12px;margin:10px 0;text-align:center;color:#7fcf7f;}.err{background:rgba(207,127,127,0.2);border:1px solid #cf7f7f;border-radius:8px;padding:12px;margin:10px 0;text-align:center;color:#cf7f7f;}.nav{text-align:center;padding:10px;}.nav a{margin:0 8px;}.stat-box{display:inline-block;background:rgba(212,164,55,0.15);border:1px solid #d4a437;border-radius:12px;padding:15px 20px;margin:8px;text-align:center;min-width:120px;}.stat-num{font-size:2em;color:#d4a437;font-weight:bold;}.stat-label{color:#a8c5a8;font-size:0.85em;}.bar{height:30px;background:#d4a437;border-radius:4px;display:flex;align-items:center;justify-content:center;color:#1a3d2e;font-weight:bold;margin:4px 0;}</style>"##;

fn html_head(title: &str) -> String {
    format!(r##"<html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>{}</title><link rel="manifest" href="/manifest.json"><meta name="theme-color" content="#1a3d2e"><meta name="apple-mobile-web-app-capable" content="yes"><link rel="apple-touch-icon" href="/icon.svg">{}</head><body>"##, title, STYLE)
}

// ===== HTML PAGES =====
fn html_home(chain: &Blockchain, users: &UserStore) -> String {
    let mut html = html_head("🦁 AfriChain");
    html.push_str(&format!(r#"<h1>🦁 AfriChain</h1><p style="text-align:center;">La blockchain 100% africaine 💚</p><div class="nav\"><a href="/register">🆕 S'inscrire</a> | <a href="/login">🔑 Connexion</a> | <a href="/wallet">👛 Wallet</a> | <a href="/admin">🔐 Admin</a> | <a href="/api/status">🔌 API</a></div><div style="text-align:center;\"><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">Blocs</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">Transactions</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">Utilisateurs</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">AFR en circulation</div></div></div><div class="card"><div style="display:flex;justify-content:space-between;padding:8px 0;border-bottom:1px solid rgba(212,164,55,0.2);\"><span style="color:#a8c5a8;\">🪙 Token</span><b>AfriRich (AFR)</b></div><div style="display:flex;justify-content:space-between;padding:8px 0;border-bottom:1px solid rgba(212,164,55,0.2);\"><span style="color:#a8c5a8;\">🌍 Lien</span><b>Monnaie AES</b></div><div style="display:flex;justify-content:space-between;padding:8px 0;border-bottom:1px solid rgba(212,164,55,0.2);\"><span style="color:#a8c5a8;\">🛡️ Statut</span><b>Souveraine 💚</b></div><div style="display:flex;justify-content:space-between;padding:8px 0;\"><span style="color:#a8c5a8;\">🔐 Crypto</span><b>Ed25519</b></div></div><footer style="text-align:center;margin-top:40px;color:#a8c5a8;\">🦁 Codée from scratch par Machine-senpai</footer>"#,
        chain.blocks.len(),
        chain.total_transactions(),
        users.count(),
        chain.total_supply(),
    ));
    html.push_str("</body></html>");
    html
}

fn html_blocks(chain: &Blockchain) -> String {
    let mut html = html_head("📊 Blocs");
    html.push_str(r#"<h1>📊 Tous les blocs</h1><div class="nav"><a href="/dashboard">← Dashboard</a> | <a href="/balances">💰 Soldes</a></div>"#);
    for block in &chain.blocks {
        html.push_str(&format!(r#"<div class="card"><h2>🧱 Bloc #{}</h2><p><b>Nonce:</b> {} | <b>TX:</b> {}</p><p style="font-family:monospace;font-size:0.85em;color:#a8c5a8;word-break:break-all;"><b>Hash:</b> {}</p><p style="font-family:monospace;font-size:0.85em;color:#a8c5a8;word-break:break-all;"><b>Préc.:</b> {}</p>"#,
            block.index, block.nonce, block.transactions.len(), block.hash, block.previous_hash));
        for tx in &block.transactions {
            let sig_icon = if tx.signature.is_empty() { "🔓" } else { "🔐" };
            html.push_str(&format!(r#"<div class="tx">{} <b>{}</b> → <b>{}</b> : {} AFR <i>({})</i></div>"#, sig_icon, tx.from, tx.to, tx.amount, tx.memo));
        }
        html.push_str("</div>");
    }
    html.push_str("</body></html>");
    html
}

fn html_balances(chain: &Blockchain) -> String {
    let balances = chain.balances();
    let mut html = html_head("💰 Soldes");
    html.push_str(r#"<h1>💰 Soldes</h1><div class="nav"><a href="/dashboard">← Dashboard</a> | <a href="/blocks">📊 Blocs</a></div>"#);
    if balances.is_empty() {
        html.push_str(r#"<p style="text-align:center;color:#a8c5a8;">Aucun wallet.</p>"#);
    } else {
        let mut entries: Vec<_> = balances.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        let max_bal = entries.iter().map(|(_, v)| v.abs()).max().unwrap_or(1).max(1);
        for (name, bal) in entries {
            let cls = if *bal >= 0 { "pos" } else { "neg" };
            let pct = ((*bal as f64) / (max_bal as f64)).abs() * 100.0;
            let short_name = if name.len() > 20 { format!("{}...", &name[..17]) } else { name.clone() };
            html.push_str(&format!(r#"<div class="card"><div style="display:flex;justify-content:space-between;align-items:center;"><span>👛 <b>{}</b></span><span class="{}" style="font-size:1.3em;font-weight:bold;">{} AFR</span></div><div style="background:rgba(0,0,0,0.3);border-radius:4px;margin-top:8px;height:8px;"><div style="background:{};height:8px;border-radius:4px;width:{}%;"></div></div></div>"#,
                short_name, cls, bal, if *bal >= 0 { "#7fcf7f" } else { "#cf7f7f" }, pct as u32));
        }
    }
    html.push_str("</body></html>");
    html
}

fn html_wallet(new_addr: Option<&str>, new_priv: Option<&str>, check_addr: Option<&str>, msg: Option<&str>, chain: &Blockchain) -> String {
    let mut html = html_head("👛 Wallet AfriRich");
    html.push_str(r#"<h1>👛 Wallet AfriRich</h1><div class="nav"><a href="/">← Retour</a> | <a href="/register">🆕 S'inscrire</a> | <a href="/login">🔑 Connexion</a></div>"#);

    if let Some(m) = msg {
        html.push_str(&format!(r#"<div class="msg">{}</div>"#, m));
    }

    html.push_str(r#"<div class="card"><h2>🆕 Créer un wallet</h2><p>Génère une adresse + clé privée Ed25519 :</p><a href="/wallet/new"><button>⚡ Générer mon adresse</button></a></div>"#);

    if let Some(addr) = new_addr {
        html.push_str(&format!(r#"<div class="card"><h2>✨ Votre nouvelle adresse</h2><div class="addr">{}</div>"#, addr));
        if let Some(priv_key) = new_priv {
            html.push_str(&format!(r#"<p style="color:#cf7f7f;margin-top:10px;">⚠️ Clé privée (gardez-la secrète !) :</p><div class="priv">{}</div>"#, priv_key));
        }
        html.push_str("</div>");
    }

    html.push_str(r#"<div class="card"><h2>💰 Voir mon solde</h2><form action="/wallet/balance" method="get"><label>Votre adresse Afri :</label><input name="addr" placeholder="Afri..." /><button type="submit">🔍 Voir le solde</button></form></div>"#);

    if let Some(addr) = check_addr {
        let bal = chain.balance_of(addr);
        let history = chain.tx_history(addr);
        html.push_str(&format!(r#"<div class="card"><h2>💰 Solde</h2><div class="addr">{}</div><div class="bal">{} AFR</div>"#, addr, bal));
        if !history.is_empty() {
            html.push_str("<h3>📜 Historique</h3>");
            for tx in &history {
                let arrow = if tx.from == addr { "📤" } else { "📥" };
                let other = if tx.from == addr { &tx.to[..] } else { &tx.from[..] };
                let sign = if tx.from == addr { "-" } else { "+" };
                html.push_str(&format!(r#"<div class="tx">{} <b>{}</b> : {}{} AFR <i>({})</i></div>"#, arrow, other, sign, tx.amount, tx.memo));
            }
        }
        html.push_str("</div>");
    }

    html.push_str(r#"<div class="card"><h2>💸 Envoyer des AFR</h2><form action="/wallet/send" method="post"><label>De (votre adresse) :</label><input name="from" placeholder="Afri..." /><label>À (adresse destinataire) :</label><input name="to" placeholder="Afri..." /><label>Montant (AFR) :</label><input name="amount" type="number" placeholder="50" /><label>Memo :</label><input name="memo" placeholder="Paiement 💚" /><button type="submit">📤 Envoyer (signé Ed25519 🔐)</button></form></div>"#);

    html.push_str(r#"<div class="card"><h2>⛏️ Miner</h2><form action="/wallet/mine" method="post"><label>Adresse du mineur :</label><input name="miner" placeholder="Afri..." /><button type="submit">⛏️ Miner !</button></form></div>"#);

    html.push_str(r#"<script>if('serviceWorker' in navigator){navigator.serviceWorker.register('/sw.js')}</script>"#);
    html.push_str("</body></html>");
    html
}

fn html_register(msg: Option<&str>) -> String {
    let mut html = html_head("🆕 Inscription AfriRich");
    html.push_str(r#"<h1>🆕 Inscription</h1><div class="nav"><a href="/">← Retour</a></div>"#);
    if let Some(m) = msg {
        html.push_str(&format!(r#"<div class="err">{}</div>"#, m));
    }
    html.push_str(r#"<div class="card"><h2>Créer ton compte AfriRich</h2><p>Choisis un nom d'utilisateur et un mot de passe. Un wallet Ed25519 sera créé automatiquement !</p><form action="/register" method="post"><label>Nom d'utilisateur :</label><input name="username" placeholder="Ex: machine" /><label>Mot de passe :</label><input name="password" type="password" placeholder="••••••" /><button type="submit">✨ S'inscrire</button></form><p style="text-align:center;margin-top:15px;"><a href="/login">Déjà inscrit ? 🔑 Connexion</a></p></div>"#);
    html.push_str("</body></html>");
    html
}

fn html_login(msg: Option<&str>) -> String {
    let mut html = html_head("🔑 Connexion AfriRich");
    html.push_str(r#"<h1>🔑 Connexion</h1><div class="nav"><a href="/">← Retour</a></div>"#);
    if let Some(m) = msg {
        html.push_str(&format!(r#"<div class="err">{}</div>"#, m));
    }
    html.push_str(r#"<div class="card"><h2>Connecte-toi</h2><form action="/login" method="post"><label>Nom d'utilisateur :</label><input name="username" placeholder="Ex: machine" /><label>Mot de passe :</label><input name="password" type="password" placeholder="••••••" /><button type="submit">🔑 Se connecter</button></form><p style="text-align:center;margin-top:15px;"><a href="/register">Pas encore inscrit ? 🆕 S'inscrire</a></p></div>"#);
    html.push_str("</body></html>");
    html
}

fn html_admin_login(err: Option<&str>) -> String {
    let mut html = html_head("🔐 Admin AfriChain");
    html.push_str(r#"<h1>🔐 Administration</h1><div class="nav"><a href="/">← Accueil</a></div>"#);
    if let Some(e) = err {
        html.push_str(&format!(r#"<div class="err">{}</div>"#, e));
    }
    html.push_str(r#"<div class="card"><h2>🔑 Connexion Admin</h2><p>Réservé à l'administrateur de AfriChain.</p><form action="/admin" method="post"><label>Mot de passe admin :</label><input name="password" type="password" placeholder="••••••••" /><button type="submit">🔐 Se connecter</button></form></div>"#);
    html.push_str("</body></html>");
    html
}

fn html_account(user: &UserAccount, chain: &Blockchain, msg: Option<&str>) -> String {
    let mut html = html_head("Mon compte AfriRich");
    let bal = chain.balance_of(&user.address);
    let history = chain.tx_history(&user.address);
    html.push_str(&format!(r#"<h1>👋 Bonjour {}</h1><div class="nav"><a href="/">← Accueil</a> | <a href="/logout">🚪 Déconnexion</a></div>"#, user.username));

    if let Some(m) = msg {
        html.push_str(&format!(r#"<div class="msg">{}</div>"#, m));
    }

    html.push_str(&format!(r#"<div class="card"><h2>👛 Mon Wallet</h2><label>Adresse :</label><div class="addr">{}</div><div class="bal">{} AFR</div></div>"#, user.address, bal));

    if !history.is_empty() {
        html.push_str(r#"<div class="card"><h2>📜 Mes transactions</h2>"#);
        for tx in &history {
            let arrow = if tx.from == user.address { "📤" } else { "📥" };
            let other = if tx.from == user.address { &tx.to[..] } else { &tx.from[..] };
            let sign = if tx.from == user.address { "-" } else { "+" };
            let sig_icon = if tx.signature.is_empty() { "🔓" } else { "🔐" };
            html.push_str(&format!(r#"<div class="tx">{} {} <b>{}</b> : {}{} AFR <i>({})</i></div>"#, sig_icon, arrow, other, sign, tx.amount, tx.memo));
        }
        html.push_str("</div>");
    } else {
        html.push_str(r#"<div class="card"><p style="text-align:center;color:#a8c5a8;">Aucune transaction. Mine pour gagner des AFR ! ⛏️</p></div>"#);
    }

    html.push_str(r#"<div class="card"><h2>💸 Envoyer des AFR</h2><form action="/account/send" method="post"><input type="hidden" name="from" value=""#);
    html.push_str(&user.address);
    html.push_str(r#"" /><label>À (adresse destinataire) :</label><input name="to" placeholder="Afri..." /><label>Montant (AFR) :</label><input name="amount" type="number" placeholder="50" /><label>Memo :</label><input name="memo" placeholder="Paiement 💚" /><button type="submit">📤 Envoyer (signé 🔐)</button></form></div>"#);

    html.push_str(r#"<div class="card"><h2>⛏️ Miner</h2><form action="/account/mine" method="post"><input type="hidden" name="miner" value=""#);
    html.push_str(&user.address);
    html.push_str(r#"" /><button type="submit">⛏️ Miner ! (+100 AFR)</button></form></div>"#);

    html.push_str("</body></html>");
    html
}

fn html_dashboard(chain: &Blockchain, users: &UserStore) -> String {
    let mut html = html_head("📈 Dashboard AfriChain");
    let balances = chain.balances();
    let total_tx = chain.total_transactions();
    let total_supply = chain.total_supply();
    let num_wallets = balances.len();
    let num_users = users.count();

    html.push_str(r#"<h1>📈 Dashboard Admin</h1><div class="nav"><a href="/">← Accueil</a> | <a href="/blocks">📊 Blocs</a> | <a href="/balances">💰 Soldes</a> | <a href="/logout">🚪 Déconnexion</a></div>"#);

    // Stats boxes
    html.push_str(&format!(r#"<div style="text-align:center;"><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">🧱 Blocs</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">💸 Transactions</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">👛 Wallets</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">👥 Utilisateurs</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">🪙 AFR total</div></div></div>"#,
        chain.blocks.len(), total_tx, num_wallets, num_users, total_supply));

    // Chart: Blocks per day
    html.push_str(r#"<div class="card"><h2>📊 Blocs minés</h2>"#);
    let max_tx = chain.blocks.iter().map(|b| b.transactions.len()).max().unwrap_or(1).max(1);
    for block in &chain.blocks {
        let tx_count = block.transactions.len();
        let pct = (tx_count as f64 / max_tx as f64) * 100.0;
        let date = chrono::DateTime::from_timestamp(block.timestamp, 0)
            .map(|d| d.format("%d/%m %H:%M").to_string())
            .unwrap_or_else(|| block.timestamp.to_string());
        html.push_str(&format!(r#"<div style="margin:4px 0;"><span style="color:#a8c5a8;font-size:0.85em;">Bloc #{} — {}</span><div style="background:rgba(0,0,0,0.3);border-radius:4px;height:24px;margin-top:2px;"><div class="bar" style="width:{}%;height:24px;border-radius:4px;font-size:0.8em;">{} tx</div></div></div>"#,
            block.index, date, pct as u32, tx_count));
    }
    html.push_str("</div>");

    // Chart: Top balances
    html.push_str(r#"<div class="card"><h2>💰 Top Wallets</h2>"#);
    let mut entries: Vec<_> = balances.iter().collect();
    entries.sort_by(|a, b| b.1.cmp(a.1));
    entries.truncate(10);
    let max_bal = entries.iter().map(|(_, v)| v.abs()).max().unwrap_or(1).max(1);
    for (name, bal) in &entries {
        let pct = ((bal.abs() as f64) / (max_bal as f64)) * 100.0;
        let short = if name.len() > 20 { format!("{}...", &name[..17]) } else { name.to_string() };
        let color = if **bal >= 0 { "#7fcf7f" } else { "#cf7f7f" };
        html.push_str(&format!(r#"<div style="margin:4px 0;"><span style="color:#a8c5a8;font-size:0.85em;">👛 {}</span><div style="background:rgba(0,0,0,0.3);border-radius:4px;height:24px;margin-top:2px;"><div style="background:{};height:24px;border-radius:4px;width:{}%;display:flex;align-items:center;justify-content:center;color:#1a3d2e;font-weight:bold;font-size:0.8em;">{} AFR</div></div></div>"#,
            short, color, pct as u32, bal));
    }
    html.push_str("</div>");

    // Chart: Transaction flow
    html.push_str(r#"<div class="card"><h2>💸 Flux des transactions</h2>"#);
    for block in &chain.blocks {
        for tx in &block.transactions {
            let from_short = if tx.from.len() > 15 { format!("{}...", &tx.from[..12]) } else { tx.from.clone() };
            let to_short = if tx.to.len() > 15 { format!("{}...", &tx.to[..12]) } else { tx.to.clone() };
            let sig = if tx.signature.is_empty() { "🔓" } else { "🔐" };
            html.push_str(&format!(r#"<div class="tx">{} {} → {} : <b>{} AFR</b> <i>({})</i></div>"#, sig, from_short, to_short, tx.amount, tx.memo));
        }
    }
    html.push_str("</div>");

    // Users list
    if num_users > 0 {
        html.push_str(r#"<div class="card"><h2>👥 Utilisateurs inscrits</h2>"#);
        for user in &users.users {
            let date = chrono::DateTime::from_timestamp(user.created_at, 0)
                .map(|d| d.format("%d/%m/%Y").to_string())
                .unwrap_or_else(|| user.created_at.to_string());
            let bal = chain.balance_of(&user.address);
            html.push_str(&format!(r#"<div class="tx">👤 <b>{}</b> — {} AFR <span style="color:#a8c5a8;font-size:0.8em;">(inscrit le {})</span></div>"#, user.username, bal, date));
        }
        html.push_str("</div>");
    }

    html.push_str("</body></html>");
    html
}

// ===== FORMS =====
#[derive(Deserialize)]
struct SendForm { from: String, to: String, amount: u64, memo: String }
#[derive(Deserialize)]
struct MineForm { miner: String }
#[derive(Deserialize)]
struct RegisterForm { username: String, password: String }
#[derive(Deserialize)]
struct LoginForm { username: String, password: String }
#[derive(Deserialize)]
struct AdminForm { password: String }

// ===== SERVER =====
use actix_web::{web, App, HttpServer, HttpResponse};

struct AppState {
    chain: Mutex<Blockchain>,
    wallets: Mutex<WalletStore>,
    users: Mutex<UserStore>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🦁 AfriChain v0.6 — Accounts + Dashboard");
    println!("💚 L'Afrique n'a pas besoin de permission");

    let chain = match Blockchain::load_from_file() {
        Some(c) => { println!("📊 {} blocs chargés", c.blocks.len()); c }
        None => {
            println!("🌱 Nouvelle blockchain — Genesis créé");
            let mut c = Blockchain::new();
            c.add_transaction(Transaction::new("Alice", "Bob", 100, "Premier transfert"));
            c.add_transaction(Transaction::new("Bob", "Charlie", 50, "Partage"));
            c.mine_pending("mineur-senegal");
            c.add_transaction(Transaction::new("Charlie", "Alice", 25, "Merci!"));
            c.mine_pending("mineur-burkina");
            c
        }
    };
    println!("✅ Valide : {}", chain.is_valid());

    let wallets = WalletStore::load();
    let users = UserStore::load();
    println!("👥 {} utilisateurs inscrits", users.count());

    let state = web::Data::new(AppState {
        chain: Mutex::new(chain),
        wallets: Mutex::new(wallets),
        users: Mutex::new(users),
    });

    println!("\n🌐 Serveur sur http://localhost:8080");
    println!("👛 Wallet sur http://localhost:8080/wallet");
    println!("🆕 Inscription sur http://localhost:8080/register");
    println!("📈 Dashboard sur http://localhost:8080/dashboard");

    HttpServer::new(move || {
        let state = state.clone();
        App::new()
            .app_data(state)
            .route("/", web::get().to(|s: web::Data<AppState>| async move {
                let chain = s.chain.lock().unwrap();
                let users = s.users.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_home(&chain, &users))
            }))
            .route("/blocks", web::get().to(|s: web::Data<AppState>, req: actix_web::HttpRequest| async move {
                if req.cookie("afri_admin").map(|c| c.value().to_string()) != Some("1".to_string()) {
                    return HttpResponse::Found().append_header(("Location", "/admin")).finish();
                }
                let chain = s.chain.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_blocks(&chain))
            }))
            .route("/balances", web::get().to(|s: web::Data<AppState>, req: actix_web::HttpRequest| async move {
                if req.cookie("afri_admin").map(|c| c.value().to_string()) != Some("1".to_string()) {
                    return HttpResponse::Found().append_header(("Location", "/admin")).finish();
                }
                let chain = s.chain.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_balances(&chain))
            }))
            .route("/wallet", web::get().to(|s: web::Data<AppState>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let chain = s.chain.lock().unwrap();
                let new_addr = q.get("new").map(|s| s.as_str());
                let new_priv = q.get("priv").map(|s| s.as_str());
                let check_addr = q.get("addr").map(|s| s.as_str());
                let msg = q.get("msg").map(|s| s.as_str());
                HttpResponse::Ok().content_type("text/html").body(html_wallet(new_addr, new_priv, check_addr, msg, &chain))
            }))
            .route("/wallet/new", web::get().to(|s: web::Data<AppState>| async move {
                let mut wallets = s.wallets.lock().unwrap();
                let (addr, priv_key) = wallets.create_wallet();
                println!("🆕 Wallet créé : {}", addr);
                HttpResponse::Found()
                    .append_header(("Location", format!("/wallet?new={}&priv={}", addr, priv_key)))
                    .finish()
            }))
            .route("/wallet/balance", web::get().to(|_s: web::Data<AppState>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let addr = q.get("addr").cloned().unwrap_or_default();
                HttpResponse::Found()
                    .append_header(("Location", format!("/wallet?addr={}", addr)))
                    .finish()
            }))
            .route("/wallet/send", web::post().to(|s: web::Data<AppState>, form: web::Form<SendForm>| async move {
                let wallets = s.wallets.lock().unwrap();
                let mut chain = s.chain.lock().unwrap();
                let mut tx = Transaction::new(&form.from, &form.to, form.amount, &form.memo);
                if let Some(sk) = wallets.get_signing_key(&form.from) {
                    tx.sign(&sk);
                    println!("🔐 Transaction signée Ed25519 : {} → {} ({} AFR)", form.from, form.to, form.amount);
                    chain.add_transaction(tx);
                    HttpResponse::Found()
                        .append_header(("Location", "/wallet?msg=✅ Transaction signée et ajoutée !"))
                        .finish()
                } else {
                    HttpResponse::Found()
                        .append_header(("Location", "/wallet?msg=⚠️ Adresse non trouvée dans ce wallet"))
                        .finish()
                }
            }))
            .route("/wallet/mine", web::post().to(|s: web::Data<AppState>, form: web::Form<MineForm>| async move {
                let mut chain = s.chain.lock().unwrap();
                chain.mine_pending(&form.miner);
                println!("⛏️ Bloc miné pour {}", form.miner);
                HttpResponse::Found()
                    .append_header(("Location", "/wallet?msg=⛏️ Bloc miné ! +100 AFR pour le mineur"))
                    .finish()
            }))
            // ===== REGISTER =====
            .route("/register", web::get().to(|_s: web::Data<AppState>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let msg = q.get("err").map(|s| s.as_str());
                HttpResponse::Ok().content_type("text/html").body(html_register(msg))
            }))
            .route("/register", web::post().to(|s: web::Data<AppState>, form: web::Form<RegisterForm>| async move {
                let mut wallets = s.wallets.lock().unwrap();
                let mut users = s.users.lock().unwrap();
                match users.register(&form.username, &form.password, &mut wallets) {
                    Ok(user) => {
                        HttpResponse::Found()
                            .append_header(("Location", format!("/account?user={}", user.username)))
                            .finish()
                    }
                    Err(e) => {
                        HttpResponse::Found()
                            .append_header(("Location", format!("/register?err={}", e)))
                            .finish()
                    }
                }
            }))
            // ===== LOGIN =====
            .route("/login", web::get().to(|_s: web::Data<AppState>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let msg = q.get("err").map(|s| s.as_str());
                HttpResponse::Ok().content_type("text/html").body(html_login(msg))
            }))
            .route("/login", web::post().to(|s: web::Data<AppState>, form: web::Form<LoginForm>| async move {
                let users = s.users.lock().unwrap();
                match users.login(&form.username, &form.password) {
                    Some(user) => {
                        HttpResponse::Found()
                            .append_header(("Location", format!("/account?user={}", user.username)))
                            .finish()
                    }
                    None => {
                        HttpResponse::Found()
                            .append_header(("Location", "/login?err=Nom d'utilisateur ou mot de passe incorrect"))
                            .finish()
                    }
                }
            }))
            // ===== ACCOUNT =====
            .route("/account", web::get().to(|s: web::Data<AppState>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let username = q.get("user").cloned().unwrap_or_default();
                let users = s.users.lock().unwrap();
                let chain = s.chain.lock().unwrap();
                let msg = q.get("msg").map(|s| s.as_str());
                match users.users.iter().find(|u| u.username == username) {
                    Some(user) => HttpResponse::Ok().content_type("text/html").body(html_account(user, &chain, msg)),
                    None => HttpResponse::Found().append_header(("Location", "/login")).finish(),
                }
            }))
            .route("/account/send", web::post().to(|s: web::Data<AppState>, form: web::Form<SendForm>| async move {
                let wallets = s.wallets.lock().unwrap();
                let mut chain = s.chain.lock().unwrap();
                let users = s.users.lock().unwrap();
                let user = users.users.iter().find(|u| u.address == form.from);
                let username = user.map(|u| u.username.clone()).unwrap_or_default();
                let mut tx = Transaction::new(&form.from, &form.to, form.amount, &form.memo);
                if let Some(sk) = wallets.get_signing_key(&form.from) {
                    tx.sign(&sk);
                    chain.add_transaction(tx);
                    HttpResponse::Found()
                        .append_header(("Location", format!("/account?user={}&msg=✅ Envoyé ! {} AFR signés", username, form.amount)))
                        .finish()
                } else {
                    HttpResponse::Found()
                        .append_header(("Location", format!("/account?user={}&msg=⚠️ Clé privée introuvable", username)))
                        .finish()
                }
            }))
            .route("/account/mine", web::post().to(|s: web::Data<AppState>, form: web::Form<MineForm>| async move {
                let mut chain = s.chain.lock().unwrap();
                let users = s.users.lock().unwrap();
                let user = users.users.iter().find(|u| u.address == form.miner);
                let username = user.map(|u| u.username.clone()).unwrap_or_default();
                chain.mine_pending(&form.miner);
                HttpResponse::Found()
                    .append_header(("Location", format!("/account?user={}&msg=⛏️ Miné ! +100 AFR", username)))
                    .finish()
            }))
            // ===== ADMIN =====
            .route("/admin", web::get().to(|_s: web::Data<AppState>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let err = q.get("err").map(|s| s.as_str());
                HttpResponse::Ok().content_type("text/html").body(html_admin_login(err))
            }))
            .route("/admin", web::post().to(|_s: web::Data<AppState>, form: web::Form<AdminForm>| async move {
                if form.password == ADMIN_PASSWORD {
                    HttpResponse::Found()
                        .cookie(actix_web::cookie::Cookie::build("afri_admin", "1").path("/").finish())
                        .append_header(("Location", "/dashboard"))
                        .finish()
                } else {
                    HttpResponse::Found()
                        .append_header(("Location", "/admin?err=Mot de passe incorrect"))
                        .finish()
                }
            }))
            .route("/logout", web::get().to(|| async move {
                HttpResponse::Found()
                    .cookie(actix_web::cookie::Cookie::build("afri_admin", "").path("/").max_age(actix_web::cookie::time::Duration::seconds(0)).finish())
                    .append_header(("Location", "/"))
                    .finish()
            }))
            // ===== DASHBOARD (ADMIN ONLY) =====
            .route("/dashboard", web::get().to(|s: web::Data<AppState>, req: actix_web::HttpRequest| async move {
                if req.cookie("afri_admin").map(|c| c.value().to_string()) != Some("1".to_string()) {
                    return HttpResponse::Found().append_header(("Location", "/admin")).finish();
                }
                let chain = s.chain.lock().unwrap();
                let users = s.users.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_dashboard(&chain, &users))
            }))
            // ===== API =====
            .route("/api/blocks", web::get().to(|s: web::Data<AppState>| async move {
                let chain = s.chain.lock().unwrap();
                HttpResponse::Ok().json(&chain.blocks)
            }))
            .route("/api/status", web::get().to(|s: web::Data<AppState>| async move {
                let chain = s.chain.lock().unwrap();
                let users = s.users.lock().unwrap();
                let json = format!(r#"{{"name":"AfriChain","blocks":{},"transactions":{},"users":{},"valid":{},"token":"AFR","version":"0.6","crypto":"Ed25519","supply":{}}}"#,
                    chain.blocks.len(), chain.total_transactions(), users.count(), chain.is_valid(), chain.total_supply());
                HttpResponse::Ok().content_type("application/json").body(json)
            }))
            // ===== PWA =====
            .route("/manifest.json", web::get().to(|| async move {
                let manifest = r##"{"name":"AfriRich Wallet","short_name":"AfriRich","start_url":"/wallet","display":"standalone","background_color":"#0d1f17","theme_color":"#1a3d2e","icons":[{"src":"/icon.svg","sizes":"any","type":"image/svg+xml","purpose":"any maskable"}]}"##;
                HttpResponse::Ok().content_type("application/json").body(manifest)
            }))
            .route("/icon.svg", web::get().to(|| async move {
                let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><rect width="512" height="512" rx="80" fill="#1a3d2e"/><text x="256" y="360" font-size="320" text-anchor="middle">🦁</text></svg>"##;
                HttpResponse::Ok().content_type("image/svg+xml").body(svg)
            }))
            .route("/sw.js", web::get().to(|| async move {
                let sw = "const C='afri-v0.6';self.addEventListener('install',e=>{e.waitUntil(caches.open(C).then(c=>c.addAll(['/wallet','/manifest.json','/icon.svg'])))});self.addEventListener('fetch',e=>{e.respondWith(caches.match(e.request).then(r=>r||fetch(e.request)))});";
                HttpResponse::Ok().content_type("application/javascript").body(sw)
            }))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
