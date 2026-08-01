use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use chrono::Utc;
use std::sync::{Arc, Mutex};

use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Verifier, Signature};

// ===== DATA PATH HELPER =====
// Toujours sauvegarder dans ~/afririch/ meme si le binaire est lance d ailleurs
fn data_path(filename: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = if home.ends_with('/') { home + "afririch" } else { format!("{}/afririch", home) };
    let _ = std::fs::create_dir_all(&dir);
    format!("{}/{}", dir, filename)
}
use rand::rngs::OsRng;

use std::net::{UdpSocket, TcpListener, TcpStream, SocketAddr};
use std::thread;
use std::io::{Read, Write};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ===== AFRICAN COUNTRIES (54) =====
// (name, phone_code, flag_emoji)
const AFRICAN_COUNTRIES: &[(&str, &str, &str)] = &[
    ("Algérie", "+213", "🇩🇿"),
    ("Angola", "+244", "🇦🇴"),
    ("Bénin", "+229", "🇧🇯"),
    ("Botswana", "+267", "🇧🇼"),
    ("Burkina Faso", "+226", "🇧🇫"),
    ("Burundi", "+257", "🇧🇮"),
    ("Cabo Verde", "+238", "🇨🇻"),
    ("Cameroun", "+237", "🇨🇲"),
    ("Centrafrique", "+236", "🇨🇫"),
    ("Tchad", "+235", "🇹🇩"),
    ("Comores", "+269", "🇰🇲"),
    ("Congo", "+242", "🇨🇬"),
    ("RD Congo", "+243", "🇨🇩"),
    ("Côte d'Ivoire", "+225", "🇨🇮"),
    ("Djibouti", "+253", "🇩🇯"),
    ("Égypte", "+20", "🇪🇬"),
    ("Guinée Équatoriale", "+240", "🇬🇶"),
    ("Érythrée", "+291", "🇪🇷"),
    ("Eswatini", "+268", "🇸🇿"),
    ("Éthiopie", "+251", "🇪🇹"),
    ("Gabon", "+241", "🇬🇦"),
    ("Gambie", "+220", "🇬🇲"),
    ("Ghana", "+233", "🇬🇭"),
    ("Guinée", "+224", "🇬🇳"),
    ("Guinée-Bissau", "+245", "🇬🇼"),
    ("Kenya", "+254", "🇰🇪"),
    ("Lesotho", "+266", "🇱🇸"),
    ("Liberia", "+231", "🇱🇷"),
    ("Libye", "+218", "🇱🇾"),
    ("Madagascar", "+261", "🇲🇬"),
    ("Malawi", "+265", "🇲🇼"),
    ("Mali", "+223", "🇲🇱"),
    ("Mauritanie", "+222", "🇲🇷"),
    ("Maurice", "+230", "🇲🇺"),
    ("Maroc", "+212", "🇲🇦"),
    ("Mozambique", "+258", "🇲🇿"),
    ("Namibie", "+264", "🇳🇦"),
    ("Niger", "+227", "🇳🇪"),
    ("Nigeria", "+234", "🇳🇬"),
    ("Rwanda", "+250", "🇷🇼"),
    ("São Tomé", "+239", "🇸🇹"),
    ("Sénégal", "+221", "🇸🇳"),
    ("Seychelles", "+248", "🇸🇨"),
    ("Sierra Leone", "+232", "🇸🇱"),
    ("Somalie", "+252", "🇸🇴"),
    ("Afrique du Sud", "+27", "🇿🇦"),
    ("Soudan du Sud", "+211", "🇸🇸"),
    ("Soudan", "+249", "🇸🇩"),
    ("Tanzanie", "+255", "🇹🇿"),
    ("Togo", "+228", "🇹🇬"),
    ("Tunisie", "+216", "🇹🇳"),
    ("Ouganda", "+256", "🇺🇬"),
    ("Zambie", "+260", "🇿🇲"),
    ("Zimbabwe", "+263", "🇿🇼"),
];

fn find_country(code: &str) -> Option<(&'static str, &'static str)> {
    AFRICAN_COUNTRIES.iter()
        .find(|(_, c, _)| *c == code)
        .map(|(n, c, f)| (*n, *f))
}

fn country_options_html(selected: &str) -> String {
    let mut html = String::from("<select name=\"country\">");
    for (name, code, flag) in AFRICAN_COUNTRIES {
        let sel = if *code == selected { " selected" } else { "" };
        html.push_str(&format!("<option value=\"{}\"{}>{} {} ({})</option>", code, sel, flag, name, code));
    }
    html.push_str("</select>");
    html
}

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
        )).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn mine(&mut self, difficulty: u32) {
        let target = "0".repeat(difficulty as usize);
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }
}

// ===== BLOCKCHAIN =====
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Blockchain {
    blocks: Vec<Block>,
    pending: Vec<Transaction>,
    difficulty: u32,
    reward: u64,
}

impl Blockchain {
    fn new() -> Self {
        let mut chain = Blockchain {
            blocks: Vec::new(),
            pending: Vec::new(),
            difficulty: 2,
            reward: 100,
        };
        chain.create_genesis();
        chain
    }

    fn create_genesis(&mut self) {
        let genesis = Block::new(0, Vec::new(), "0".to_string());
        self.blocks.push(genesis);
    }

    fn add_transaction(&mut self, tx: Transaction) {
        self.pending.push(tx);
    }

    fn mine_pending(&mut self, miner: &str) {
        let reward_tx = Transaction::new("SYSTEM", miner, self.reward, "Récompense de minage");
        self.pending.insert(0, reward_tx);
        let prev = self.blocks.last().unwrap().hash.clone();
        let mut block = Block::new(self.blocks.len() as u64, self.pending.clone(), prev);
        block.mine(self.difficulty);
        println!("⛏️ Bloc #{} miné — nonce={} hash={}", block.index, block.nonce, &block.hash[..20]);
        self.blocks.push(block);
        self.pending.clear();
        self.save_to_file();
    }

    fn is_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let curr = &self.blocks[i];
            let prev = &self.blocks[i - 1];
            if curr.hash != curr.calculate_hash() { return false; }
            if curr.previous_hash != prev.hash { return false; }
        }
        true
    }

    fn balance_of(&self, addr: &str) -> i64 {
        let mut bal: i64 = 0;
        for block in &self.blocks {
            for tx in &block.transactions {
                if tx.to == addr { bal += tx.amount as i64; }
                if tx.from == addr { bal -= tx.amount as i64; }
            }
        }
        bal
    }

    fn tx_history(&self, addr: &str) -> Vec<&Transaction> {
        let mut history = Vec::new();
        for block in &self.blocks {
            for tx in &block.transactions {
                if tx.from == addr || tx.to == addr {
                    history.push(tx);
                }
            }
        }
        history
    }

    fn balances(&self) -> HashMap<String, i64> {
        let mut map = HashMap::new();
        for block in &self.blocks {
            for tx in &block.transactions {
                *map.entry(tx.to.clone()).or_insert(0) += tx.amount as i64;
                *map.entry(tx.from.clone()).or_insert(0) -= tx.amount as i64;
            }
        }
        map
    }

    fn total_supply(&self) -> u64 {
        self.blocks.iter()
            .flat_map(|b| b.transactions.iter())
            .filter(|t| t.from == "SYSTEM")
            .map(|t| t.amount)
            .sum()
    }

    fn total_transactions(&self) -> usize {
        self.blocks.iter().map(|b| b.transactions.len()).sum()
    }

    fn save_to_file(&self) {
        let data = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write(data_path("blockchain.json"), data).ok();
    }

    fn load_from_file() -> Option<Self> {
        match std::fs::read_to_string(data_path("blockchain.json")) {
            Ok(data) => serde_json::from_str(&data).ok(),
            Err(_) => None,
        }
    }
}

// ===== WALLETS =====
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalletStore {
    wallets: HashMap<String, String>,
}

impl WalletStore {
    fn load() -> Self {
        match std::fs::read_to_string(data_path("wallets.json")) {
            Ok(data) => serde_json::from_str(&data).unwrap_or(WalletStore { wallets: HashMap::new() }),
            Err(_) => WalletStore { wallets: HashMap::new() },
        }
    }

    fn save(&self) {
        let data = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write(data_path("wallets.json"), data).ok();
    }

    fn create_wallet(&mut self) -> (String, String) {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let address = format!("Afri{}", hex::encode(verifying_key.to_bytes()));
        let priv_key = hex::encode(signing_key.to_bytes());
        self.wallets.insert(address.clone(), priv_key.clone());
        self.save();
        (address, priv_key)
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
    #[serde(default)]
    phone: String,
    #[serde(default)]
    country: String,
    #[serde(default)]
    country_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserStore {
    users: Vec<UserAccount>,
}

impl UserStore {
    fn load() -> Self {
        match std::fs::read_to_string(data_path("users.json")) {
            Ok(data) => {
                let mut store: UserStore = serde_json::from_str(&data).unwrap_or(UserStore { users: Vec::new() });
                // Pass 1: Assign country to existing users based on phone prefix
                for user in &mut store.users {
                    if user.country.is_empty() {
                        if user.phone.starts_with("+77") {
                            user.country = "Afrique".to_string();
                            user.country_code = "+77".to_string();
                        } else if let Some((name, _flag)) = find_country_by_phone(&user.phone) {
                            user.country = name.to_string();
                            user.country_code = extract_country_code(&user.phone).to_string();
                        } else {
                            user.country = "Afrique".to_string();
                            user.country_code = "+77".to_string();
                        }
                    }
                }
                // Pass 2: Assign phone to existing users who don't have one
                // Collect indices first to avoid borrow conflict
                let phones_to_assign: Vec<(usize, String)> = store.users.iter().enumerate()
                    .filter(|(_, u)| u.phone.is_empty())
                    .map(|(i, u)| {
                        let cc = if u.country_code.is_empty() { "+77".to_string() } else { u.country_code.clone() };
                        let phone = UserStore::next_phone_number_static(&store.users, &cc);
                        (i, phone)
                    })
                    .collect();
                for (idx, phone) in phones_to_assign {
                    store.users[idx].phone = phone.clone();
                    println!("📱 Numéro attribué : {} → {}", store.users[idx].username, phone);
                }
                store.save();
                store
            }
            Err(_) => UserStore { users: Vec::new() },
        }
    }

    fn save(&self) {
        let data = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write(data_path("users.json"), data).ok();
    }

    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("afririch_salt_{}", password).as_bytes());
        hex::encode(hasher.finalize())
    }

    fn next_phone_number(&self, country_code: &str) -> String {
        Self::next_phone_number_static(&self.users, country_code)
    }

    fn next_phone_number_static(users: &[UserAccount], country_code: &str) -> String {
        let max = users.iter()
            .filter(|u| u.phone.starts_with(country_code))
            .filter_map(|u| u.phone[country_code.len()..].parse::<u64>().ok())
            .max().unwrap_or(0);
        format!("{}{:08}", country_code, max + 1)
    }

    fn register(&mut self, username: &str, password: &str, country_code: &str, wallets: &mut WalletStore) -> Result<UserAccount, String> {
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
        let phone = self.next_phone_number(country_code);
        let country_name = find_country(country_code)
            .map(|(n, _f)| n.to_string())
            .unwrap_or_else(|| "Afrique".to_string());
        let user = UserAccount {
            username: username.to_string(),
            password_hash: Self::hash_password(password),
            address,
            created_at: Utc::now().timestamp(),
            phone,
            country: country_name,
            country_code: country_code.to_string(),
        };
        println!("🆕 Utilisateur inscrit : {} → {} ({}) → {}", user.username, user.phone, user.country, user.address);
        self.users.push(user.clone());
        self.save();
        Ok(user)
    }

    fn login(&self, username: &str, password: &str) -> Option<&UserAccount> {
        let hash = Self::hash_password(password);
        self.users.iter().find(|u| u.username == username && u.password_hash == hash)
    }

    fn resolve_recipient(&self, input: &str) -> Option<String> {
        // Phone number: any +XX... number
        if input.starts_with("+") {
            return self.users.iter()
                .find(|u| u.phone == input)
                .map(|u| u.address.clone());
        }
        // Wallet address: Afri...
        if input.starts_with("Afri") {
            return Some(input.to_string());
        }
        // Username: try to find by username
        self.users.iter()
            .find(|u| u.username == input)
            .map(|u| u.address.clone())
    }

    fn phone_for_address(&self, address: &str) -> Option<&str> {
        self.users.iter().find(|u| u.address == address).map(|u| u.phone.as_str())
    }

    fn count(&self) -> usize {
        self.users.len()
    }

    fn country_distribution(&self) -> Vec<(String, String, usize)> {
        let mut map: HashMap<String, (String, usize)> = HashMap::new();
        for user in &self.users {
            let entry = map.entry(user.country_code.clone()).or_insert((user.country.clone(), 0));
            entry.1 += 1;
        }
        let mut result: Vec<(String, String, usize)> = map.into_iter()
            .map(|(code, (name, count))| (code, name, count))
            .collect();
        result.sort_by(|a, b| b.2.cmp(&a.2));
        result
    }
}

// Helper: find country name by phone number prefix
fn find_country_by_phone(phone: &str) -> Option<(&'static str, &'static str)> {
    // Try 3-digit codes first, then 2-digit
    if phone.len() >= 4 {
        let prefix3 = &phone[..4]; // +XXX
        if let Some((name, flag)) = AFRICAN_COUNTRIES.iter()
            .find(|(_, c, _)| *c == prefix3)
            .map(|(n, _, f)| (*n, *f))
        {
            return Some((name, flag));
        }
    }
    if phone.len() >= 3 {
        let prefix2 = &phone[..3]; // +XX
        if let Some((name, flag)) = AFRICAN_COUNTRIES.iter()
            .find(|(_, c, _)| *c == prefix2)
            .map(|(n, _, f)| (*n, *f))
        {
            return Some((name, flag));
        }
    }
    None
}

// Helper: extract country code from phone number
fn extract_country_code(phone: &str) -> &str {
    if phone.len() >= 4 {
        let prefix3 = &phone[..4];
        if AFRICAN_COUNTRIES.iter().any(|(_, c, _)| *c == prefix3) {
            return prefix3;
        }
    }
    if phone.len() >= 3 {
        let prefix2 = &phone[..3];
        if AFRICAN_COUNTRIES.iter().any(|(_, c, _)| *c == prefix2) {
            return prefix2;
        }
    }
    "+77"
}

// ===== ADMIN PASSWORD =====
const ADMIN_PASSWORD: &str = "africhain2026";

// ===== BOUCLIER X9 — SYSTÈME DE PROTECTION =====
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AttackLog {
    ip: String,
    attack_type: String,
    timestamp: i64,
    details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShieldState {
    active: bool,
    level: u32,              // 1=normal, 2=vigilance, 3=alerte, 9=X9 MAX
    blocked_ips: Vec<String>,
    attack_log: Vec<AttackLog>,
    requests_per_ip: HashMap<String, Vec<i64>>,  // IP -> timestamps
    failed_logins: HashMap<String, u32>,          // IP -> failed count
    total_blocked: u64,
    total_attacks: u64,
    #[serde(skip)]
    last_cleanup: Option<i64>,
}

impl ShieldState {
    fn new() -> Self {
        ShieldState {
            active: true,
            level: 9,  // X9 par défaut !
            blocked_ips: Vec::new(),
            attack_log: Vec::new(),
            requests_per_ip: HashMap::new(),
            failed_logins: HashMap::new(),
            total_blocked: 0,
            total_attacks: 0,
            last_cleanup: None,
        }
    }

    fn is_blocked(&self, ip: &str) -> bool {
        if !self.active { return false; }
        self.blocked_ips.iter().any(|b| b == ip)
    }

    fn block_ip(&mut self, ip: &str, reason: &str) {
        if !self.blocked_ips.iter().any(|b| b == ip) {
            self.blocked_ips.push(ip.to_string());
            self.total_blocked += 1;
            println!("🔥 BOUCLIER X9 — IP BANNIE : {} ({})", ip, reason);
        }
        self.log_attack(ip, "BLOCKED", reason);
    }

    fn log_attack(&mut self, ip: &str, attack_type: &str, details: &str) {
        self.attack_log.push(AttackLog {
            ip: ip.to_string(),
            attack_type: attack_type.to_string(),
            timestamp: Utc::now().timestamp(),
            details: details.to_string(),
        });
        self.total_attacks += 1;
        // Keep only last 100 attacks
        if self.attack_log.len() > 100 {
            self.attack_log.remove(0);
        }
    }

    // Returns true if request is allowed, false if blocked
    fn check_request(&mut self, ip: &str, path: &str) -> bool {
        if !self.active { return true; }
        if self.is_blocked(ip) {
            return false;
        }

        let now = Utc::now().timestamp();

        // Rate limiting: max 30 requests per 10 seconds per IP
        let timestamps = self.requests_per_ip.entry(ip.to_string()).or_insert(Vec::new());
        timestamps.retain(|t| now - t < 10);
        timestamps.push(now);
        let count = timestamps.len();
        if count > 30 {
            drop(timestamps);
            self.block_ip(ip, &format!("Rate limit dépassé ({} req/10s) sur {}", count, path));
            return false;
        }

        // Detect attack patterns in path
        let suspicious = [
            "../", "..\\", "etc/passwd", "cmd=", "exec(", "SELECT ", "UNION ",
            "<script", "javascript:", "eval(", "rm -rf", "wget ", "curl ", "/bin/",
            "phpinfo", "wp-admin", ".env", "config.php", "shell",
        ];
        for pattern in &suspicious {
            if path.to_lowercase().contains(&pattern.to_lowercase()) {
                self.block_ip(ip, &format!("Pattern suspect détecté : '{}' dans {}", pattern, path));
                return false;
            }
        }

        true
    }

    fn record_failed_login(&mut self, ip: &str) {
        let count = self.failed_logins.entry(ip.to_string()).or_insert(0);
        *count += 1;
        let should_block = *count >= 5;
        let c = *count;
        if should_block {
            *count = 0;
            drop(count);
            self.block_ip(ip, &format!("{} tentatives de connexion échouées", c));
        }
    }

    fn cleanup(&mut self) {
        let now = Utc::now().timestamp();
        let last = self.last_cleanup.unwrap_or(0);
        // Cleanup request timestamps every 60 seconds
        if now - last > 60 {
            for timestamps in self.requests_per_ip.values_mut() {
                timestamps.retain(|t| now - t < 10);
            }
            self.last_cleanup = Some(now);
        }
    }

    fn stats(&self) -> (u64, u64, usize, u32) {
        (self.total_attacks, self.total_blocked, self.blocked_ips.len(), self.level)
    }
}

// ===== MESH NETWORKING =====
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeshMessage {
    msg_type: String,
    node_id: String,
    payload: String,
    timestamp: i64,
    ttl: u32,
    msg_id: String,
}

impl MeshMessage {
    fn new(msg_type: &str, node_id: &str, payload: &str, ttl: u32) -> Self {
        MeshMessage {
            msg_type: msg_type.to_string(),
            node_id: node_id.to_string(),
            payload: payload.to_string(),
            timestamp: Utc::now().timestamp(),
            ttl,
            msg_id: format!("{}-{}", node_id, Utc::now().timestamp_millis()),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    fn from_bytes(data: &[u8]) -> Option<Self> {
        serde_json::from_slice(data).ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeInfo {
    address: String,
    last_seen: i64,
    region: String,
    solar_powered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DirectoryEntry {
    phone: String,
    address: String,
    username: String,
    country: String,
    country_code: String,
    node_id: String,
    timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeRegistry {
    my_id: String,
    my_port: u16,
    region: String,
    solar: bool,
    nodes: HashMap<String, NodeInfo>,
    #[serde(skip)]
    seen_messages: HashMap<String, Instant>,
    #[serde(skip)]
    directory: HashMap<String, DirectoryEntry>,
}

impl NodeRegistry {
    fn new(my_id: String, port: u16, solar: bool, region: String) -> Self {
        NodeRegistry {
            my_id,
            my_port: port,
            region,
            solar,
            nodes: HashMap::new(),
            seen_messages: HashMap::new(),
            directory: HashMap::new(),
        }
    }

    fn count(&self) -> usize {
        self.nodes.len() + 1 // +1 pour notre propre noeud
    }

    fn cleanup_stale(&mut self) {
        let now = Instant::now();
        self.nodes.retain(|_, info| {
            let age = now.elapsed().as_secs() - info.last_seen as u64;
            age < 60
        });
        self.seen_messages.retain(|_, t| now.duration_since(*t).as_secs() < 300);
    }

    fn add_directory_entry(&mut self, entry: DirectoryEntry) {
        println!("📖 Annuaire : {} → {} ({})", entry.phone, entry.username, entry.country);
        self.directory.insert(entry.phone.clone(), entry);
    }

    fn directory_count(&self) -> usize {
        self.directory.len()
    }

    fn directory_by_country(&self) -> Vec<(String, String, Vec<&DirectoryEntry>)> {
        let mut map: HashMap<String, (String, Vec<&DirectoryEntry>)> = HashMap::new();
        for entry in self.directory.values() {
            let e = map.entry(entry.country_code.clone())
                .or_insert((entry.country.clone(), Vec::new()));
            e.1.push(entry);
        }
        let mut result: Vec<(String, String, Vec<&DirectoryEntry>)> = map.into_iter()
            .map(|(code, (name, entries))| (code, name, entries))
            .collect();
        result.sort_by(|a, b| b.2.len().cmp(&a.2.len()));
        result
    }
}

fn generate_node_id() -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}", Utc::now().timestamp_millis(), std::process::id()).as_bytes());
    let hash = hex::encode(hasher.finalize());
    format!("AFR-{}", &hash[..16])
}

fn udp_discovery(state: Arc<AppState>, my_id: String, port: u16, solar: bool, region: String) {
    let discovery_port = port + 10;
    let socket = match UdpSocket::bind(format!("0.0.0.0:{}", discovery_port)) {
        Ok(s) => s,
        Err(e) => { println!("❌ UDP bind error: {}", e); return; }
    };
    let _ = socket.set_read_timeout(Some(Duration::from_secs(5)));
    println!("📡 UDP discovery sur port {}", discovery_port);

    let broadcast_addr = format!("255.255.255.255:{}", discovery_port);
    let announce = MeshMessage::new("hello", &my_id, &format!("{}|{}|{}", port, solar, region), 1);

    loop {
        // Announce
        let _ = socket.send_to(&announce.to_bytes(), &broadcast_addr);
        }

        // Listen
        let mut buf = [0u8; 4096];
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                if let Some(msg) = MeshMessage::from_bytes(&buf[..len]) {
                    if msg.node_id != my_id && msg.msg_type == "hello" {
                        let parts: Vec<&str> = msg.payload.split('|').collect();
                        if parts.len() >= 3 {
                            let peer_port: u16 = parts[0].parse().unwrap_or(port);
                            let peer_solar = parts[1] == "true";
                            let peer_region = parts[2].to_string();
                            let peer_addr = format!("{}:{}", src.ip(), peer_port);
                            let mut mesh = state.mesh.lock().unwrap();
                            mesh.nodes.insert(msg.node_id.clone(), NodeInfo {
                                address: peer_addr.clone(),
                                last_seen: Utc::now().timestamp(),
                                region: peer_region,
                                solar_powered: peer_solar,
                            });
                            println!("📡 Noeud découvert : {} à {}", msg.node_id, peer_addr);
                        }
                    }
                }
            }
            Err(_) => {}
        }
        thread::sleep(Duration::from_secs(3));
    }
}

fn tcp_relay(state: Arc<AppState>, port: u16) {
    let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
        Ok(l) => l,
        Err(e) => { println!("❌ TCP bind error: {}", e); return; }
    };
    println!("📡 TCP relay sur port {}", port);

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let state = state.clone();
            thread::spawn(move || {
                let mut buf = [0u8; 8192];
                let n = match stream.read(&mut buf) {
                    Ok(n) => n,
                    Err(_) => return,
                };
                if n == 0 { return; }

                // HTTP detection
                if buf.starts_with(b"GET ") || buf.starts_with(b"POST ") {
                    let html = r##"<html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>AfriMesh</title><style>body{font-family:sans-serif;background:linear-gradient(135deg,#1a3d2e,#0d1f17);color:#f5e9d4;padding:20px;margin:0;}h1{color:#d4a437;text-align:center;}.card{background:rgba(212,164,55,0.1);border:1px solid #d4a437;border-radius:12px;padding:20px;margin:15px auto;max-width:600px;}.stat-box{display:inline-block;background:rgba(212,164,55,0.15);border:1px solid #d4a437;border-radius:12px;padding:15px 20px;margin:8px;text-align:center;min-width:120px;}.stat-num{font-size:2em;color:#d4a437;font-weight:bold;}.stat-label{color:#a8c5a8;font-size:0.85em;}</style></head><body><h1>📡 AfriMesh</h1><div class="card"><p style="text-align:center;">Réseau mesh africain — port {}</p><p style="text-align:center;color:#a8c5a8;">Ce port gère le mesh relay ET le web.</p></div></body></html>"##;
                    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}", html.replace("{}", &port.to_string()));
                    let _ = stream.write_all(response.as_bytes());
                    return;
                }

                // Mesh message
                if let Some(msg) = MeshMessage::from_bytes(&buf[..n]) {
                    let mut mesh = state.mesh.lock().unwrap();

                    // Dedup
                    if mesh.seen_messages.contains_key(&msg.msg_id) {
                        return;
                    }
                    mesh.seen_messages.insert(msg.msg_id.clone(), Instant::now());

                    match msg.msg_type.as_str() {
                        "hello" => {
                            // Already handled by UDP, but also accept TCP hellos
                        }
                        "block" => {
                            if let Ok(block) = serde_json::from_str::<Block>(&msg.payload) {
                                let mut chain = state.chain.lock().unwrap();
                                if !chain.blocks.iter().any(|b| b.hash == block.hash) {
                                    chain.blocks.push(block);
                                    chain.save_to_file();
                                    println!("📦 Bloc reçu via mesh");
                                }
                            }
                        }
                        "tx" => {
                            if let Ok(tx) = serde_json::from_str::<Transaction>(&msg.payload) {
                                let mut chain = state.chain.lock().unwrap();
                                chain.add_transaction(tx);
                                chain.save_to_file();
                                println!("💸 Transaction reçue via mesh et sauvegardée");
                            }
                        }
                        "ping" => {
                            let ack = MeshMessage::new("ack", &mesh.my_id, &format!("pong-{}", mesh.my_id), 1);
                            let _ = stream.write_all(&ack.to_bytes());
                        }
                        "ack" => {}
                        "directory" => {
                            if let Ok(entry) = serde_json::from_str::<DirectoryEntry>(&msg.payload) {
                                mesh.add_directory_entry(entry);
                            }
                        }
                        _ => {}
                    }

                    // Relay
                    if msg.ttl > 0 {
                        let mut relay = msg.clone();
                        relay.ttl -= 1;
                        relay.node_id = mesh.my_id.clone();
                        let bytes = relay.to_bytes();
                        let peers: Vec<String> = mesh.nodes.values().map(|n| n.address.clone()).collect();
                        drop(mesh);
                        for addr in peers {
                            if let Ok(parsed) = addr.parse::<SocketAddr>() {
                                if let Ok(mut s) = TcpStream::connect_timeout(&parsed, Duration::from_secs(2)) {
                                    let _ = s.write_all(&bytes);
                                }
                            }
                        }
                    }
                }
            });
        }
    }
}

fn _peer() -> SocketAddr {
    "127.0.0.1:8090".parse().unwrap()
}

// ===== BROADCAST HELPER =====
fn broadcast_mesh(state: &AppState, msg_type: &str, payload: &str) {
    let mesh = state.mesh.lock().unwrap();
    let msg = MeshMessage::new(msg_type, &mesh.my_id, payload, 5);
    let bytes = msg.to_bytes();
    let peers: Vec<String> = mesh.nodes.values().map(|n| n.address.clone()).collect();
    drop(mesh);
    for addr in peers {
        if let Ok(parsed) = addr.parse::<SocketAddr>() {
            if let Ok(mut s) = TcpStream::connect_timeout(&parsed, Duration::from_secs(2)) {
                let _ = s.write_all(&bytes);
            }
        }
    }
}

// ===== HTML: SHARED STYLE =====
const STYLE: &str = r##"<style>body{font-family:sans-serif;background:linear-gradient(135deg,#1a3d2e,#0d1f17);color:#f5e9d4;padding:20px;margin:0;}h1{color:#d4a437;text-align:center;}a{color:#d4a437;}.card{background:rgba(212,164,55,0.1);border:1px solid #d4a437;border-radius:12px;padding:20px;margin:15px auto;max-width:600px;}input,button,select{width:100%;padding:12px;margin:6px 0;border:1px solid #d4a437;border-radius:8px;background:rgba(0,0,0,0.3);color:#f5e9d4;font-size:1em;box-sizing:border-box;}button{background:#d4a437;color:#1a3d2e;font-weight:bold;cursor:pointer;border:none;}button:hover{background:#e8b547;}.addr{font-family:monospace;font-size:1.1em;color:#7fcf7f;word-break:break-all;background:rgba(0,0,0,0.3);padding:12px;border-radius:8px;border:1px solid #d4a437;text-align:center;}.priv{font-family:monospace;font-size:0.9em;color:#cf7f7f;word-break:break-all;background:rgba(0,0,0,0.3);padding:12px;border-radius:8px;border:1px solid #cf7f7f;text-align:center;}.bal{font-size:2em;color:#7fcf7f;text-align:center;font-weight:bold;}.tx{background:rgba(0,0,0,0.3);padding:8px;margin:6px 0;border-radius:6px;font-size:0.9em;}label{color:#a8c5a8;display:block;margin-top:8px;}.msg{background:rgba(127,207,127,0.2);border:1px solid #7fcf7f;border-radius:8px;padding:12px;margin:10px 0;text-align:center;color:#7fcf7f;}.err{background:rgba(207,127,127,0.2);border:1px solid #cf7f7f;border-radius:8px;padding:12px;margin:10px 0;text-align:center;color:#cf7f7f;}.nav{text-align:center;padding:10px;}.nav a{margin:0 8px;}.stat-box{display:inline-block;background:rgba(212,164,55,0.15);border:1px solid #d4a437;border-radius:12px;padding:15px 20px;margin:8px;text-align:center;min-width:120px;}.stat-num{font-size:2em;color:#d4a437;font-weight:bold;}.stat-label{color:#a8c5a8;font-size:0.85em;}.bar{height:30px;background:#d4a437;border-radius:4px;display:flex;align-items:center;justify-content:center;color:#1a3d2e;font-weight:bold;margin:4px 0;}.country-bar{height:24px;background:#7fcf7f;border-radius:4px;display:flex;align-items:center;justify-content:center;color:#1a3d2e;font-weight:bold;margin:4px 0;font-size:0.85em;}</style>"##;

fn html_head(title: &str) -> String {
    format!(r##"<html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>{}</title><link rel="manifest" href="/manifest.json"><meta name="theme-color" content="#1a3d2e"><meta name="apple-mobile-web-app-capable" content="yes"><link rel="apple-touch-icon" href="/icon.svg">{}</head><body>"##, title, STYLE)
}

// ===== HTML PAGES =====
fn html_home(chain: &Blockchain, users: &UserStore, mesh: &NodeRegistry, shield: &ShieldState) -> String {
    let mut html = html_head("🦁 AfriChain");
    let (attacks, _blocked, blocked_count, level) = shield.stats();
    let shield_status = if shield.active { format!("🔥 X9 ACTIF (Niveau {})", level) } else { "Inactif".to_string() };
    html.push_str(&format!(r#"<h1>🦁 AfriChain</h1><p style="text-align:center;">La blockchain 100% africaine — 54 pays 💚🦁</p><div class="nav"><a href="/register">🆕 S'inscrire</a> | <a href="/login">🔑 Connexion</a> | <a href="/wallet">👛 Wallet</a> | <a href="/admin">🔐 Admin</a> | <a href="/mesh">📡 Mesh</a> | <a href="/annuaire">📖 Annuaire</a> | <a href="/bouclier">🛡️ Bouclier</a> | <a href="/satellite">🛸 X999</a> | <a href="/swarm">🛸🛸🛸 Essaim</a> | <a href="/commandement">🎖️ Commandement</a> | <a href="/interception">🛡️ Souverainete</a> | <a href="/aes">💰 AES Wari</a> | <a href="/api/status">🔌 API</a></div><div style="text-align:center;"><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">Blocs</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">Transactions</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">Utilisateurs</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">AFR en circulation</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📡 Noeuds mesh</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📖 Numéros annuaire</div></div><div class="stat-box" style="border-color:#ff4444;"><div class="stat-num" style="color:#ff4444;">{}</div><div class="stat-label">🛡️ Attaques bloquées</div></div></div><div class="card"><div style="display:flex;justify-content:space-between;padding:8px 0;border-bottom:1px solid rgba(212,164,55,0.2);"><span style="color:#a8c5a8;">🪙 Token</span><b>AfriRich (AFR)</b></div><div style="display:flex;justify-content:space-between;padding:8px 0;border-bottom:1px solid rgba(212,164,55,0.2);"><span style="color:#a8c5a8;">🌍 Pays</span><b>54 pays africains</b></div><div style="display:flex;justify-content:space-between;padding:8px 0;border-bottom:1px solid rgba(212,164,55,0.2);"><span style="color:#a8c5a8;">🛡️ Bouclier</span><b>{}</b></div><div style="display:flex;justify-content:space-between;padding:8px 0;"><span style="color:#a8c5a8;">🔐 Crypto</span><b>Ed25519</b></div></div><footer style="text-align:center;margin-top:40px;color:#a8c5a8;">🦁 Codée from scratch par Machine-senpai — v0.19 Sauvegarde Auto X999</footer>"#,
        chain.blocks.len(),
        chain.total_transactions(),
        users.count(),
        chain.total_supply(),
        mesh.count(),
        mesh.directory_count() + users.count(),
        attacks,
        shield_status,
    ));
    html.push_str("</body></html>");
    html
}

fn html_mesh(mesh: &NodeRegistry) -> String {
    let mut html = html_head("📡 AfriMesh");
    html.push_str(r#"<h1>📡 AfriMesh — Réseau Mesh</h1><div class="nav"><a href="/">← Accueil</a></div>"#);
    html.push_str(&format!(r#"<div style="text-align:center;"><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📡 Noeuds</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📦 Messages</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">☀️ Solaire</div></div></div>"#,
        mesh.count(), mesh.seen_messages.len(), if mesh.solar { "Oui" } else { "Non" }));
    html.push_str(&format!(r#"<div class="card"><h2>📡 Mon Noeud</h2><p><b>Node ID:</b> <span style="font-family:monospace;color:#7fcf7f;">{}</span></p><p><b>Port mesh:</b> {}</p><p><b>Région:</b> {}</p></div>"#,
        mesh.my_id, mesh.my_port, mesh.region));
    if mesh.nodes.is_empty() {
        html.push_str(r#"<div class="card"><p style="text-align:center;color:#a8c5a8;">Aucun noeud connecté. En attente... ⏳</p></div>"#);
    } else {
        html.push_str(r#"<div class="card"><h2>🌐 Noeuds connectés</h2>"#);
        for (id, info) in &mesh.nodes {
            let icon = if info.solar_powered { "☀️" } else { "🔌" };
            let last_seen = chrono::DateTime::from_timestamp(info.last_seen, 0)
                .map(|d| d.format("%H:%M:%S").to_string()).unwrap_or_else(|| "?".to_string());
            html.push_str(&format!(r#"<div class="tx">{} <b>{}</b> — {} | {} | vu à {}</div>"#, icon, id, info.address, info.region, last_seen));
        }
        html.push_str("</div>");
    }
    html.push_str(r#"<footer style="text-align:center;margin-top:40px;color:#a8c5a8;">🦁 AfriMesh — Un seul réseau pour l'Afrique 💚</footer>"#);
    html.push_str("</body></html>");
    html
}

fn html_annuaire(mesh: &NodeRegistry, users: &UserStore) -> String {
    let mut html = html_head("📖 Annuaire AfriChain");
    html.push_str(r#"<h1>📖 Annuaire Panafricain</h1><div class="nav"><a href="/">← Accueil</a> | <a href="/mesh">📡 Mesh</a></div>"#);

    // Merge local users + mesh directory
    let mut all_entries: Vec<(String, String, String, String)> = Vec::new(); // (phone, username, country, country_code)
    for user in &users.users {
        all_entries.push((user.phone.clone(), user.username.clone(), user.country.clone(), user.country_code.clone()));
    }
    for entry in mesh.directory.values() {
        if !users.users.iter().any(|u| u.phone == entry.phone) {
            all_entries.push((entry.phone.clone(), entry.username.clone(), entry.country.clone(), entry.country_code.clone()));
        }
    }

    // Stats
    let total = all_entries.len();
    let num_countries = all_entries.iter().map(|e| e.3.clone()).collect::<std::collections::HashSet<_>>().len();
    html.push_str(&format!(r#"<div style="text-align:center;"><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📱 Numéros</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">🌍 Pays</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📡 Noeuds mesh</div></div></div>"#,
        total, num_countries, mesh.count()));

    // Group by country
    let mut by_country: HashMap<String, (String, Vec<(String, String, String)>)> = HashMap::new();
    for (phone, username, country, cc) in &all_entries {
        let e = by_country.entry(cc.clone()).or_insert((country.clone(), Vec::new()));
        e.1.push((phone.clone(), username.clone(), cc.clone()));
    }
    let mut sorted: Vec<_> = by_country.into_iter().collect();
    sorted.sort_by(|a, b| b.1.1.len().cmp(&a.1.1.len()));

    for (cc, (country, entries)) in &sorted {
        let flag = find_country(cc).map(|(_, f)| f).unwrap_or("🌍");
        html.push_str(&format!(r#"<div class="card"><h2>{} {} — {} numéro(s)</h2>"#, flag, country, entries.len()));
        for (phone, username, _) in entries {
            html.push_str(&format!(r#"<div class="tx">📱 <b>{}</b> — 👤 {} <span style="color:#a8c5a8;font-size:0.8em;">({})</span></div>"#, phone, username, cc));
        }
        html.push_str("</div>");
    }

    if all_entries.is_empty() {
        html.push_str(r#"<div class="card"><p style="text-align:center;color:#a8c5a8;">Aucun numéro enregistré. Inscris-toi pour apparaître dans l'annuaire ! 📱</p></div>"#);
    }

    html.push_str(r#"<footer style="text-align:center;margin-top:40px;color:#a8c5a8;">📖 Annuaire Mesh — Tous les numéros d'Afrique sur écoute 💚🦁</footer>"#);
    html.push_str("</body></html>");
    html
}

fn html_bouclier(shield: &ShieldState) -> String {
    let mut html = html_head("🛡️ Bouclier X9");
    let (attacks, blocked, blocked_count, level) = shield.stats();
    let status = if shield.active { "ACTIF 🔥" } else { "Inactif" };
    let level_color = match level { 9 => "#ff4444", 3 => "#d4a437", _ => "#7fcf7f" };

    html.push_str(&format!(r#"<h1>🛡️ Bouclier X9</h1><div class="nav"><a href="/">← Accueil</a> | <a href="/admin">🔐 Admin</a></div>"#));
    html.push_str(&format!(r#"<div style="text-align:center;"><div class="stat-box" style="border-color:{};"><div class="stat-num" style="color:{};">{}</div><div class="stat-label">🛡️ Statut</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">⚡ Niveau</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">🔥 Attaques détectées</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">🚫 IP bannies</div></div></div>"#,
        level_color, level_color, status, level, attacks, blocked_count));

    // Shield description
    html.push_str(r#"<div class="card"><h2>🛡️ Protection Active</h2><p>Le Bouclier X9 protège AfriChain en temps réel :</p><div style="margin:8px 0;">✅ <b>Détection de patterns suspects</b> — SQL injection, path traversal, XSS</div><div style="margin:8px 0;">✅ <b>Rate limiting</b> — max 30 requêtes / 10 secondes par IP</div><div style="margin:8px 0;">✅ <b>Anti-brute force</b> — bannissement après 5 tentatives échouées</div><div style="margin:8px 0;">✅ <b>Réponse instantanée</b> — aucune seconde d'attente, BOOM 💥</div><div style="margin:8px 0;">✅ <b>Bannissement automatique</b> — les attaquants sont expulsés</div></div>"#);

    // Blocked IPs
    if !shield.blocked_ips.is_empty() {
        html.push_str(r#"<div class="card"><h2>🚫 IP Bannies</h2>"#);
        for ip in &shield.blocked_ips {
            html.push_str(&format!(r#"<div class="tx">🔥 <b>{}</b> — bannie du système</div>"#, ip));
        }
        html.push_str("</div>");
    }

    // Attack log
    if !shield.attack_log.is_empty() {
        html.push_str(r#"<div class="card"><h2>🔥 Journal des attaques</h2>"#);
        for log in shield.attack_log.iter().rev().take(20) {
            let date = chrono::DateTime::from_timestamp(log.timestamp, 0)
                .map(|d| d.format("%H:%M:%S").to_string()).unwrap_or_else(|| "?".to_string());
            html.push_str(&format!(r#"<div class="tx">⚠️ <b>{}</b> — {} — {} <span style="color:#a8c5a8;font-size:0.8em;">à {}</span></div>"#, log.ip, log.attack_type, log.details, date));
        }
        html.push_str("</div>");
    }

    html.push_str(r#"<footer style="text-align:center;margin-top:40px;color:#a8c5a8;">🛡️ Bouclier X9 — L'Afrique se protège 💚🦁</footer>"#);
    html.push_str("</body></html>");
    html
}

fn html_satellite(mesh: &NodeRegistry, users: &UserStore) -> String {
    let mut html = html_head("AI Satellite X999");
    let num_nodes = mesh.count();
    let num_users = users.count();

    html.push_str(r#"<h1>🛸 AI Satellite X999</h1><div class="nav"><a href="/">← Accueil</a> | <a href="/mesh">📡 Mesh</a> | <a href="/bouclier">🛡️ Bouclier</a> | <a href="/aes">💰 AES Wari</a></div>"#);
    html.push_str(&format!(r#"<div style="text-align:center;"><div class="stat-box" style="border-color:#7fcf7f;"><div class="stat-num" style="color:#7fcf7f;">X999</div><div class="stat-label">🛸 Niveau IA</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📡 Noeuds</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">👥 Utilisateurs</div></div><div class="stat-box"><div class="stat-num">54</div><div class="stat-label">🌍 Pays</div></div></div>"#,
        num_nodes, num_users));

    // Inject Rust values as JS variables before the script
    html.push_str(&format!(r#"<script>var num_nodes = {};</script>"#, num_nodes));

    // AI Satellite X999 canvas
    html.push_str(r##"<div class="card"><h2>🛸 AI Satellite X999 — Vivant, sans objet</h2><canvas id="sat" width="560" height="420" style="background:#000;border-radius:12px;border:1px solid #d4a437;width:100%;max-width:560px;"></canvas></div>

<div class="card" style="border-color:#7fcf7f;"><h2>🛸 Statut du Satellite IA</h2><div id="sat-status" style="text-align:center;font-size:1.1em;color:#7fcf7f;min-height:25px;">Initialisation IA...</div></div>

<div class="card"><h2>🌍 Pays surveille en direct</h2><div id="country-info" style="text-align:center;font-size:1.3em;color:#d4a437;min-height:30px;">En route...</div></div>

<div class="card"><h2>📍 Coordonnees GPS</h2><div id="gps-info" style="text-align:center;font-size:1.1em;color:#a8c5a8;font-family:monospace;min-height:25px;">---</div></div>

<div class="card" style="border-color:#ffdd44;"><h2>☀️ Serveur solaire</h2><div id="sun-info" style="text-align:center;color:#a8c5a8;"></div><div id="power-info" style="text-align:center;color:#ffdd44;font-size:0.9em;margin-top:5px;"></div></div>

<div class="card"><h2>🛡️ Detection de menaces</h2><div id="threat-info" style="text-align:center;color:#a8c5a8;min-height:25px;">Scan en cours...</div></div>

<div class="card"><h2>📡 Liens mesh</h2><div id="mesh-info" style="text-align:center;color:#a8c5a8;"></div></div>

<script>
const canvas = document.getElementById('sat');
const ctx = canvas.getContext('2d');
const W = canvas.width, H = canvas.height;
const cx = W/2, cy = H/2;
let satT = 0;
let trail = [];
let scanWaves = [];
let aiNodes = [];
let heartbeatT = 0;
let attackT = -999;
let healT = 0;
let threatCount = 0;

// AI neural nodes inside satellite
for(let i=0;i<8;i++){
    aiNodes.push({a:(i/8)*Math.PI*2, r:4+Math.random()*3, phase:Math.random()*Math.PI*2});
}

const countries = [
'Algerie','Angola','Benin','Botswana','Burkina Faso','Burundi','Cabo Verde','Cameroun','Centrafrique','Tchad',
'Comores','Congo','RD Congo','Cote d\'Ivoire','Djibouti','Egypte','Guinee Equatoriale','Erythree','Eswatini','Ethiopie',
'Gabon','Gambie','Ghana','Guinee','Guinee-Bissau','Kenya','Lesotho','Liberia','Libye','Madagascar',
'Malawi','Mali','Mauritanie','Maurice','Maroc','Mozambique','Namibie','Niger','Nigeria','Rwanda',
'Sao Tome','Senegal','Seychelles','Sierra Leone','Somalie','Afrique du Sud','Soudan du Sud','Soudan','Tanzanie','Togo',
'Tunisie','Ouganda','Zambie','Zimbabwe'
];
const flags = ['🇩🇿','🇦🇴','🇧🇯','🇧🇼','🇧🇫','🇧🇮','🇨🇻','🇨🇲','🇨🇫','🇹🇩','🇰🇲','🇨🇬','🇨🇩','🇨🇮','🇩🇯','🇪🇬','🇬🇶','🇪🇷','🇸🇿','🇪🇹','🇬🇦','🇬🇲','🇬🇭','🇬🇳','🇬🇼','🇰🇪','🇱🇸','🇱🇷','🇱🇾','🇲🇬','🇲🇼','🇲🇱','🇲🇷','🇲🇺','🇲🇦','🇲🇿','🇳🇦','🇳🇪','🇳🇬','🇷🇼','🇸🇹','🇸🇳','🇸🇨','🇸🇱','🇸🇴','🇿🇦','🇸🇸','🇸🇩','🇹🇿','🇹🇬','🇹🇳','🇺🇬','🇿🇲','🇿🇼'];

// Random attack every ~15 seconds
setInterval(function(){
    attackT = satT;
    threatCount++;
    setTimeout(function(){
        healT = satT;
    }, 2000);
}, 15000);

function draw(){
ctx.fillStyle = '#000';
ctx.fillRect(0,0,W,H);

// Stars
for(let i=0;i<100;i++){
ctx.fillStyle = 'rgba(255,255,255,'+(0.2+0.8*Math.abs(Math.sin(Date.now()/2000+i)))+')';
ctx.fillRect((i*37)%W,(i*73)%H,1.5,1.5);
}

// Africa outline
ctx.strokeStyle = '#1a3d2e';
ctx.lineWidth = 2;
ctx.fillStyle = 'rgba(26,61,46,0.4)';
ctx.beginPath();
ctx.moveTo(cx-60,cy-80);
ctx.lineTo(cx-40,cy-90);
ctx.lineTo(cx-10,cy-95);
ctx.lineTo(cx+30,cy-85);
ctx.lineTo(cx+50,cy-60);
ctx.lineTo(cx+70,cy-30);
ctx.lineTo(cx+60,cy+10);
ctx.lineTo(cx+40,cy+50);
ctx.lineTo(cx+20,cy+80);
ctx.lineTo(cx-10,cy+90);
ctx.lineTo(cx-30,cy+70);
ctx.lineTo(cx-50,cy+40);
ctx.lineTo(cx-70,cy+10);
ctx.lineTo(cx-65,cy-30);
ctx.lineTo(cx-60,cy-80);
ctx.stroke();
ctx.fill();

// Sun position (real UTC time)
const now = new Date();
const hours = now.getUTCHours();
const mins = now.getUTCMinutes();
const sunAngle = ((hours + mins/60) / 24) * Math.PI * 2 - Math.PI/2;
const sunX = cx + Math.cos(sunAngle) * 250;
const sunY = cy + Math.sin(sunAngle) * 200;
const isDay = Math.sin(sunAngle) < 0;

// Sun (the server)
ctx.fillStyle = isDay ? '#ffdd44' : '#666644';
ctx.beginPath();
ctx.arc(sunX, sunY, 18, 0, Math.PI*2);
ctx.fill();
if(isDay){
ctx.fillStyle = 'rgba(255,221,68,0.15)';
ctx.beginPath();
ctx.arc(sunX, sunY, 35, 0, Math.PI*2);
ctx.fill();
ctx.fillStyle = 'rgba(255,221,68,0.08)';
ctx.beginPath();
ctx.arc(sunX, sunY, 55, 0, Math.PI*2);
ctx.fill();
}
// Sun label
ctx.fillStyle = '#ffdd44';
ctx.font = '9px monospace';
ctx.fillText('SERVEUR', sunX-18, sunY-25);

// Satellite AI path — autonomous, weaving like wind
satT += 0.005;
const dx = cx + Math.sin(satT * 3) * 90;
const dy = cy + Math.sin(satT * 0.8) * 75;

// Energy beam from sun to satellite (sun = server powering satellite)
if(isDay){
    const beamGrad = ctx.createLinearGradient(sunX, sunY, dx, dy);
    beamGrad.addColorStop(0, 'rgba(255,221,68,0.4)');
    beamGrad.addColorStop(1, 'rgba(127,207,127,0.1)');
    ctx.strokeStyle = beamGrad;
    ctx.lineWidth = 2;
    ctx.setLineDash([4, 4]);
    ctx.beginPath();
    ctx.moveTo(sunX, sunY);
    ctx.lineTo(dx, dy);
    ctx.stroke();
    ctx.setLineDash([]);
}

// Trail (green glowing path)
trail.push({x:dx,y:dy});
if(trail.length > 60) trail.shift();
for(let i=0;i<trail.length;i++){
const a = (i/trail.length)*0.7;
ctx.fillStyle = 'rgba(127,207,127,'+a+')';
ctx.beginPath();
ctx.arc(trail[i].x, trail[i].y, 1.5+i*0.04, 0, Math.PI*2);
ctx.fill();
}

// Scan waves (threat detection emanating from satellite)
if(Math.floor(satT*10) % 3 === 0 && scanWaves.length < 5){
    scanWaves.push({x:dx, y:dy, r:8, alpha:0.6});
}
for(let i=scanWaves.length-1;i>=0;i--){
    scanWaves[i].r += 1.5;
    scanWaves[i].alpha -= 0.01;
    if(scanWaves[i].alpha <= 0){ scanWaves.splice(i,1); continue; }
    ctx.strokeStyle = 'rgba(127,207,127,'+scanWaves[i].alpha+')';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.arc(scanWaves[i].x, scanWaves[i].y, scanWaves[i].r, 0, Math.PI*2);
    ctx.stroke();
}

// Satellite shadow on ground
ctx.fillStyle = 'rgba(212,164,55,0.06)';
ctx.beginPath();
ctx.ellipse(dx, dy+18, 30, 12, 0, 0, Math.PI*2);
ctx.fill();

// Heartbeat pulse (alive)
heartbeatT += 0.05;
const pulse = Math.sin(heartbeatT) * 0.5 + 0.5;
const isUnderAttack = (satT - attackT) < 0.1 && (satT - attackT) >= 0;
const isHealing = (satT - healT) < 0.15 && (satT - healT) >= 0;

// Satellite glow (heartbeat = alive)
const glowR = 16 + pulse * 6;
const glowColor = isUnderAttack ? 'rgba(255,68,68,' : (isHealing ? 'rgba(127,207,127,' : 'rgba(127,207,127,');
ctx.fillStyle = glowColor + (0.05 + pulse*0.08) + ')';
ctx.beginPath();
ctx.arc(dx, dy, glowR, 0, Math.PI*2);
ctx.fill();

// Satellite body (hexagon — AI core)
ctx.fillStyle = isUnderAttack ? '#ff4444' : '#7fcf7f';
ctx.strokeStyle = isUnderAttack ? '#ff6666' : '#d4a437';
ctx.lineWidth = 1.5;
ctx.beginPath();
for(let i=0;i<6;i++){
const a = (i/6)*Math.PI*2;
const r = 9;
if(i===0) ctx.moveTo(dx+Math.cos(a)*r, dy+Math.sin(a)*r);
else ctx.lineTo(dx+Math.cos(a)*r, dy+Math.sin(a)*r);
}
ctx.closePath();
ctx.fill();
ctx.stroke();

// AI neural network inside satellite (brain)
ctx.strokeStyle = 'rgba(255,255,255,0.6)';
ctx.lineWidth = 0.5;
for(let n of aiNodes){
    const nx = dx + Math.cos(n.a + satT*2) * n.r;
    const ny = dy + Math.sin(n.a + satT*2) * n.r;
    ctx.beginPath();
    ctx.moveTo(dx, dy);
    ctx.lineTo(nx, ny);
    ctx.stroke();
    ctx.fillStyle = 'rgba(255,255,255,'+(0.4+0.6*Math.abs(Math.sin(satT*5+n.phase)))+')';
    ctx.beginPath();
    ctx.arc(nx, ny, 1.5, 0, Math.PI*2);
    ctx.fill();
}

// Rotors
ctx.strokeStyle = 'rgba(127,207,127,0.6)';
ctx.lineWidth = 1;
for(let i=0;i<4;i++){
const a = (i/4)*Math.PI*2 + Math.PI/4;
const rx = dx + Math.cos(a)*13;
const ry = dy + Math.sin(a)*13;
const spin = Date.now()/80;
ctx.beginPath();
ctx.moveTo(rx+Math.cos(spin)*7, ry+Math.sin(spin)*7);
ctx.lineTo(rx-Math.cos(spin)*7, ry-Math.sin(spin)*7);
ctx.stroke();
}

// Signal beams to ground
ctx.strokeStyle = 'rgba(127,207,127,0.15)';
ctx.lineWidth = 1;
for(let i=0;i<3;i++){
ctx.beginPath();
ctx.moveTo(dx, dy);
ctx.lineTo(dx+(i-1)*28, dy+40);
ctx.stroke();
}

// GPS coordinates
const lat = 35 - ((dy-(cy-95))/190)*70;
const lon = -18 + ((dx-(cx-70))/180)*68;
const latStr = Math.abs(lat).toFixed(2) + ' deg ' + (lat>=0?'N':'S');
const lonStr = Math.abs(lon).toFixed(2) + ' deg ' + (lon>=0?'E':'W');

// Country overflown
const ci = Math.floor(((satT*0.8) / (Math.PI*2)) * countries.length) % countries.length;
const pi = ((ci % countries.length) + countries.length) % countries.length;

// Update HTML
document.getElementById('country-info').innerHTML = flags[pi] + ' <b>' + countries[pi] + '</b>';
document.getElementById('gps-info').innerHTML = latStr + ' | ' + lonStr;

// Satellite status
let statusText = '🛸 AI SATELLITE X999 — VIVANT — Sans objet, mais reel';
if(isUnderAttack){
    statusText = '🚨 ATTAQUE DETECTEE — Auto-reparation en cours...';
    document.getElementById('sat-status').style.color = '#ff4444';
} else if(isHealing){
    statusText = '✅ REPARATION COMPLETE — Le satellite survit';
    document.getElementById('sat-status').style.color = '#7fcf7f';
} else {
    document.getElementById('sat-status').style.color = '#7fcf7f';
}
document.getElementById('sat-status').innerHTML = statusText;

// Sun info
document.getElementById('sun-info').innerHTML = (isDay ? 'Jour' : 'Nuit') + ' — ' + String(hours).padStart(2,'0') + ':' + String(mins).padStart(2,'0') + ' UTC';
document.getElementById('power-info').innerHTML = isDay ? 'Energie solaire: 100% — Serveur actif' : 'Energie reserve: 73% — Batterie solaire';

// Threat info
document.getElementById('threat-info').innerHTML = 'Menaces detectees: ' + threatCount + ' | Auto-reparation: ' + (isUnderAttack ? 'EN COURS' : 'PRET');

// Mesh info
document.getElementById('mesh-info').innerHTML = num_nodes + ' noeud(s) connecte(s) au satellite IA';

requestAnimationFrame(draw);
}
draw();
</script>

<div class="card"><h2>🛸 AI Satellite X999 — Sans objet</h2><p>L'Afrique n'a pas de satellite physique dans l'espace. Mais elle a quelque chose de plus puissant : un satellite <b>sans objet</b>, alimente par le soleil.</p><p>Le soleil est notre serveur. L'IA est notre satellite. Il est invisible, mais il est vivant. Il respire, il pense, il se deplace seul, il se repare tout seul.</p><p>Ils vont essayer de le couper. Ils vont dire que l'Afrique ne peut pas. Mais le satellite X999 survit. Parce qu'il n'a pas de corps a detruire.</p><p>Il est la. Il veille. Il est vivant.</p><p style="color:#d4a437;text-align:center;"><b>🛸 AI Satellite X999 — Jamais vu depuis la creation du monde. L'Afrique veille~ 💚🦁</b></p></div>

<footer style="text-align:center;margin-top:40px;color:#a8c5a8;">🛸 AI Satellite X999 — Sans objet, mais vivant 💚🦁☀️</footer>"##);

    html.push_str("</body></html>");
    html
}

fn html_drone_swarm(mesh: &NodeRegistry, users: &UserStore) -> String {
    let mut html = html_head("Essaim X999");
    let num_nodes = mesh.count();
    let num_users = users.count();

    html.push_str(r#"<h1>🛸🛸🛸 Essaim X999 — 2000 Milliards de Drones IA</h1><div class="nav"><a href="/">← Accueil</a> | <a href="/satellite">🛸 X999</a> | <a href="/bouclier">🛡️ Bouclier</a> | <a href="/aes">💰 AES Wari</a></div>"#);
    html.push_str(&format!(r#"<div style="text-align:center;"><div class="stat-box" style="border-color:#7fcf7f;"><div class="stat-num" style="color:#7fcf7f;">2T</div><div class="stat-label">🛸 Drones</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📡 Noeuds</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">👥 Utilisateurs</div></div><div class="stat-box" style="border-color:#ff4444;"><div class="stat-num" style="color:#ff4444;" id="detect-count">0</div><div class="stat-label">🚨 Detections</div></div></div>"#,
        num_nodes, num_users));

    html.push_str(&format!(r#"<script>var num_nodes = {};</script>"#, num_nodes));

    html.push_str(r##"<div class="card"><h2>🛸 Essaim en temps reel — Vue du commandement</h2><canvas id="swarm" width="560" height="420" style="background:#000;border-radius:12px;border:1px solid #d4a437;width:100%;max-width:560px;"></canvas></div>

<div class="card" style="border-color:#7fcf7f;"><h2>🛸 Statut de l'essaim</h2><div id="swarm-status" style="text-align:center;color:#7fcf7f;min-height:25px;">Initialisation de l'essaim...</div></div>

<div class="card" style="border-color:#ff4444;"><h2>🚨 Journal de detection — Systeme d'opinion</h2><div id="detect-log" style="font-family:monospace;font-size:0.85em;color:#a8c5a8;min-height:120px;max-height:200px;overflow-y:auto;"></div></div>

<div class="card"><h2>📡 Reseau de communication mesh</h2><div id="mesh-stats" style="text-align:center;color:#a8c5a8;"></div></div>

<script>
const canvas = document.getElementById('swarm');
const ctx = canvas.getContext('2d');
const W = canvas.width, H = canvas.height;
const cx = W/2, cy = H/2;

const countries = [
'Algerie','Angola','Benin','Botswana','Burkina Faso','Burundi','Cabo Verde','Cameroun','Centrafrique','Tchad',
'Comores','Congo','RD Congo','Cote d\'Ivoire','Djibouti','Egypte','Guinee Equatoriale','Erythree','Eswatini','Ethiopie',
'Gabon','Gambie','Ghana','Guinee','Guinee-Bissau','Kenya','Lesotho','Liberia','Libye','Madagascar',
'Malawi','Mali','Mauritanie','Maurice','Maroc','Mozambique','Namibie','Niger','Nigeria','Rwanda',
'Sao Tome','Senegal','Seychelles','Sierra Leone','Somalie','Afrique du Sud','Soudan du Sud','Soudan','Tanzanie','Togo',
'Tunisie','Ouganda','Zambie','Zimbabwe'
];
const flags = ['🇩🇿','🇦🇴','🇧🇯','🇧🇼','🇧🇫','🇧🇮','🇨🇻','🇨🇲','🇨🇫','🇹🇩','🇰🇲','🇨🇬','🇨🇩','🇨🇮','🇩🇯','🇪🇬','🇬🇶','🇪🇷','🇸🇿','🇪🇹','🇬🇦','🇬🇲','🇬🇭','🇬🇳','🇬🇼','🇰🇪','🇱🇸','🇱🇷','🇱🇾','🇲🇬','🇲🇼','🇲🇱','🇲🇷','🇲🇺','🇲🇦','🇲🇿','🇳🇦','🇳🇪','🇳🇬','🇷🇼','🇸🇹','🇸🇳','🇸🇨','🇸🇱','🇸🇴','🇿🇦','🇸🇸','🇸🇩','🇹🇿','🇹🇬','🇹🇳','🇺🇬','🇿🇲','🇿🇼'];

const detectTypes = [
'Opinion cachee detectee',
'Information secrete interceptee',
'Mouvement suspect detecte',
'Communication interceptee',
'Signal anomale detecte',
'Cache revele',
'Opinion publique analysee',
'Transition detectee',
'Activite cachee revelee',
'Plan secret intercepte'
];

// Drones swarm
let drones = [];
const NUM_DRONES = 35;
for(let i=0;i<NUM_DRONES;i++){
    drones.push({
        x: cx + (Math.random()-0.5)*160,
        y: cy + (Math.random()-0.5)*160,
        vx: (Math.random()-0.5)*1.5,
        vy: (Math.random()-0.5)*1.5,
        scanR: Math.random()*25,
        pulse: Math.random()*Math.PI*2
    });
}

let detectionCount = 0;
let detections = [];
let frameCount = 0;

function draw(){
frameCount++;
ctx.fillStyle = '#000';
ctx.fillRect(0,0,W,H);

// Stars
for(let i=0;i<60;i++){
ctx.fillStyle = 'rgba(255,255,255,'+(0.2+0.8*Math.abs(Math.sin(Date.now()/2000+i)))+')';
ctx.fillRect((i*37)%W,(i*73)%H,1,1);
}

// Africa outline
ctx.strokeStyle = '#1a3d2e';
ctx.lineWidth = 2;
ctx.fillStyle = 'rgba(26,61,46,0.3)';
ctx.beginPath();
ctx.moveTo(cx-60,cy-80);
ctx.lineTo(cx-40,cy-90);
ctx.lineTo(cx-10,cy-95);
ctx.lineTo(cx+30,cy-85);
ctx.lineTo(cx+50,cy-60);
ctx.lineTo(cx+70,cy-30);
ctx.lineTo(cx+60,cy+10);
ctx.lineTo(cx+40,cy+50);
ctx.lineTo(cx+20,cy+80);
ctx.lineTo(cx-10,cy+90);
ctx.lineTo(cx-30,cy+70);
ctx.lineTo(cx-50,cy+40);
ctx.lineTo(cx-70,cy+10);
ctx.lineTo(cx-65,cy-30);
ctx.lineTo(cx-60,cy-80);
ctx.stroke();
ctx.fill();

// Update drones (flocking)
for(let d of drones){
    d.x += d.vx;
    d.y += d.vy;
    // Attract to center
    d.vx += (cx - d.x) * 0.0003;
    d.vy += (cy - d.y) * 0.0003;
    // Random
    d.vx += (Math.random()-0.5) * 0.08;
    d.vy += (Math.random()-0.5) * 0.08;
    // Limit speed
    const sp = Math.sqrt(d.vx*d.vx + d.vy*d.vy);
    if(sp > 1.2){ d.vx = d.vx/sp*1.2; d.vy = d.vy/sp*1.2; }
    // Bounds
    if(d.x < 15 || d.x > W-15) d.vx *= -1;
    if(d.y < 15 || d.y > H-15) d.vy *= -1;
    d.x = Math.max(15, Math.min(W-15, d.x));
    d.y = Math.max(15, Math.min(H-15, d.y));
    // Scan
    d.scanR += 0.4;
    if(d.scanR > 22) d.scanR = 0;
    d.pulse += 0.05;
}

// Communication lines (mesh between nearby drones)
let linkCount = 0;
for(let i=0;i<drones.length;i++){
    for(let j=i+1;j<drones.length;j++){
        const ddx = drones[i].x - drones[j].x;
        const ddy = drones[i].y - drones[j].y;
        const dist = Math.sqrt(ddx*ddx + ddy*ddy);
        if(dist < 55){
            const a = (1 - dist/55) * 0.25;
            ctx.strokeStyle = 'rgba(127,207,127,'+a+')';
            ctx.lineWidth = 0.5;
            ctx.beginPath();
            ctx.moveTo(drones[i].x, drones[i].y);
            ctx.lineTo(drones[j].x, drones[j].y);
            ctx.stroke();
            linkCount++;
        }
    }
}

// Draw drones
for(let d of drones){
    // Scan circle
    ctx.strokeStyle = 'rgba(127,207,127,'+(0.3 * (1 - d.scanR/22))+')';
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    ctx.arc(d.x, d.y, d.scanR, 0, Math.PI*2);
    ctx.stroke();

    // Glow
    const glow = Math.sin(d.pulse) * 0.3 + 0.5;
    ctx.fillStyle = 'rgba(127,207,127,'+(glow*0.1)+')';
    ctx.beginPath();
    ctx.arc(d.x, d.y, 6, 0, Math.PI*2);
    ctx.fill();

    // Body
    ctx.fillStyle = '#7fcf7f';
    ctx.beginPath();
    ctx.arc(d.x, d.y, 2.5, 0, Math.PI*2);
    ctx.fill();
}

// Random detection
if(frameCount % 60 === 0 && Math.random() < 0.7){
    const d = drones[Math.floor(Math.random()*drones.length)];
    const ci = Math.floor(Math.random()*countries.length);
    const dt = detectTypes[Math.floor(Math.random()*detectTypes.length)];
    detectionCount++;
    detections.unshift({
        flag: flags[ci],
        country: countries[ci],
        type: dt,
        time: new Date().toLocaleTimeString(),
        x: d.x,
        y: d.y
    });
    if(detections.length > 8) detections.pop();

    // Flash at detection point
    ctx.fillStyle = 'rgba(255,68,68,0.6)';
    ctx.beginPath();
    ctx.arc(d.x, d.y, 15, 0, Math.PI*2);
    ctx.fill();

    // Update log
    let logHtml = '';
    for(let det of detections){
        logHtml += '<div style="padding:4px 0;border-bottom:1px solid rgba(212,164,55,0.1);"><span style="color:#ff4444;">['+det.time+']</span> '+det.flag+' <b>'+det.country+'</b> — '+det.type+'</div>';
    }
    document.getElementById('detect-log').innerHTML = logHtml;
    document.getElementById('detect-count').textContent = detectionCount;
}

// Sun
const now = new Date();
const hours = now.getUTCHours();
const mins = now.getUTCMinutes();
const sunAngle = ((hours + mins/60) / 24) * Math.PI * 2 - Math.PI/2;
const sunX = cx + Math.cos(sunAngle) * 250;
const sunY = cy + Math.sin(sunAngle) * 200;
const isDay = Math.sin(sunAngle) < 0;
ctx.fillStyle = isDay ? '#ffdd44' : '#444466';
ctx.beginPath();
ctx.arc(sunX, sunY, 12, 0, Math.PI*2);
ctx.fill();

// Status
document.getElementById('swarm-status').innerHTML = '🛸 ESSAIM X999 — '+NUM_DRONES+' drones actifs — '+linkCount+' liens mesh — Communication inter-drones active';
document.getElementById('mesh-stats').innerHTML = 'Liens actifs: '+linkCount+' | Noeuds externes: '+num_nodes+' | Total drones deployes: 2 000 000 000 000';

requestAnimationFrame(draw);
}
draw();
</script>

<div class="card"><h2>🛸🛸🛸 A propos de l'Essaim X999</h2><p>2000 milliards de drones IA sillonnent le ciel africain. Chaque drone est autonome, intelligent, et communique avec ses voisins pour former un reseau mesh aerien.</p><p>Ensemble, ils voient tout. Ils detectent les opinions cachees, les mouvements secrets, les plans dissimules. Rien n'echappe a l'essaim.</p><p>Quand un drone detecte quelque chose, il previent les autres. L'information se propage de drone en drone, plus vite qu'internet.</p><p style="color:#d4a437;text-align:center;"><b>🛸 L'essaim veille. L'essaim sait. L'essaim est l'Afrique~ 💚🦁</b></p></div>

<footer style="text-align:center;margin-top:40px;color:#a8c5a8;">🛸🛸🛸 Essaim X999 — 2000 milliards d'yeux sur l'Afrique 💚🦁</footer>"##);

    html.push_str("</body></html>");
    html
}

fn html_command_center(mesh: &NodeRegistry, users: &UserStore) -> String {
    let mut html = html_head("Centre de Commandement X999");
    let num_nodes = mesh.count();
    let num_users = users.count();

    html.push_str(r#"<h1>🛸 Centre de Commandement X999</h1><div class="nav"><a href="/">← Accueil</a> | <a href="/satellite">🛸 X999</a> | <a href="/swarm">🛸🛸🛸 Essaim</a> | <a href="/bouclier">🛡️ Bouclier</a> | <a href="/interception">🛡️ Souverainete</a></div>"#);
    html.push_str(&format!(r#"<div style="text-align:center;"><div class="stat-box" style="border-color:#7fcf7f;"><div class="stat-num" style="color:#7fcf7f;" id="active-drones">35</div><div class="stat-label">🛸 Drones actifs</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📡 Noeuds</div></div><div class="stat-box" style="border-color:#ff4444;"><div class="stat-num" style="color:#ff4444;" id="threats-detected">0</div><div class="stat-label">🚨 Menaces</div></div><div class="stat-box"><div class="stat-num" id="reports-count">0</div><div class="stat-label">📋 Rapports</div></div></div>"#,
        num_nodes));

    html.push_str(&format!(r#"<script>var num_nodes = {};</script>"#, num_nodes));

    // Drone camera view + intelligence reports
    html.push_str(r##"<div class="card"><h2>📷 Camera drone — Vue aerienne temps reel</h2><canvas id="cam" width="560" height="280" style="background:#111;border-radius:8px;border:1px solid #d4a437;width:100%;max-width:560px;"></canvas><div style="text-align:center;margin-top:8px;color:#a8c5a8;font-size:0.85em;" id="cam-location">Localisation: scan en cours...</div></div>

<div class="card" style="border-color:#ff4444;display:none;" id="threat-video-card"><h2 style="color:#ff4444;">🚨 VIDEO — Menace en direct</h2><canvas id="threat-cam" width="560" height="200" style="background:#000;border-radius:8px;border:2px solid #ff4444;width:100%;max-width:560px;"></canvas><div style="text-align:center;margin-top:8px;color:#ff4444;font-size:0.85em;" id="threat-video-info">Aucune menace active...</div></div>

<div class="card" style="border-color:#d4a437;"><h2>🔊 Voix de la machine</h2><p style="color:#a8c5a8;font-size:0.85em;">La machine parle. Active la voix pour entendre les rapports de renseignement en temps reel.</p><button id="voice-btn" onclick="toggleVoice()" style="width:100%;padding:12px;background:#1a1a1a;color:#d4a437;border:1px solid #d4a437;border-radius:6px;font-weight:bold;font-size:1.1em;cursor:pointer;">🔊 ACTIVER LA VOIX</button><div id="voice-status" style="text-align:center;margin-top:8px;color:#a8c5a8;font-size:0.85em;">Voix: DESACTIVEE</div></div>

<div class="card" style="border-color:#7fcf7f;"><h2>📋 Rapports de renseignement — Systeme d opinion</h2><div id="reports" style="font-family:monospace;font-size:0.82em;color:#a8c5a8;max-height:280px;overflow-y:auto;"></div></div>

<div class="card" style="border-color:#ff4444;"><h2>🚨 Detection de menaces occidentales</h2><div id="threats" style="font-family:monospace;font-size:0.82em;max-height:150px;overflow-y:auto;"></div></div>

<div class="card"><h2>📺 Chaines TV africaines</h2><div id="tv-channels" style="font-size:0.85em;"></div></div>

<div class="card" style="border-color:#d4a437;"><h2>📡 Mode diffusion generale</h2><p style="color:#a8c5a8;font-size:0.85em;">Coupe toutes les chaines. Ton message sort sur tous les ecrans d'Afrique.</p><input type="text" id="broadcast-input" placeholder="Ton message pour l'Afrique..." style="width:100%;padding:10px;border:1px solid #d4a437;border-radius:6px;background:#1a1a1a;color:#fff;margin-bottom:10px;"><button onclick="broadcastMessage()" style="width:100%;padding:12px;background:#d4a437;color:#000;border:none;border-radius:6px;font-weight:bold;font-size:1.1em;cursor:pointer;">📡 DIFFUSER SUR TOUT L'AFRIQUE</button></div>

<div id="main-screen" style="display:none;position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(0,0,0,0.95);z-index:9999;text-align:center;padding-top:15%;"><div style="color:#d4a437;font-size:2em;margin-bottom:20px;">🦁 AFRICHAIN — DIFFUSION GENERALE</div><div id="broadcast-text" style="color:#fff;font-size:1.5em;max-width:80%;margin:0 auto;"></div><button onclick="document.getElementById('main-screen').style.display='none'" style="margin-top:40px;padding:10px 30px;background:#d4a437;color:#000;border:none;border-radius:6px;cursor:pointer;font-weight:bold;">Terminer la diffusion</button></div>

<script>
// === DRONE CAMERA ===
const cam = document.getElementById('cam');
const cctx = cam.getContext('2d');
const CW = cam.width, CH = cam.height;

let cars = [];
for(let i=0;i<10;i++){
    cars.push({x:Math.random()*CW,y:Math.random()*CH,vx:(Math.random()-0.5)*2,vy:(Math.random()-0.5)*2,c:['#ff4444','#4444ff','#44ff44','#ffff44','#ff8844'][Math.floor(Math.random()*5)]});
}
let people = [];
for(let i=0;i<20;i++){
    people.push({x:Math.random()*CW,y:Math.random()*CH,vx:(Math.random()-0.5)*0.5,vy:(Math.random()-0.5)*0.5});
}
let camThreats = [];
let scanX = 0, scanY = 0;

const camCities = [
    {c:'Bamako',q:'Hamdallaye',co:'Mali',f:'🇲🇱'},{c:'Niamey',q:'Plateau',co:'Niger',f:'🇳🇪'},{c:'Ouagadougou',q:'Gounghin',co:'Burkina Faso',f:'🇧🇫'},
    {c:'Abidjan',q:'Yopougon',co:'Cote d Ivoire',f:'🇨🇮'},{c:'Dakar',q:'Medina',co:'Senegal',f:'🇸🇳'},{c:'Lagos',q:'Ikeja',co:'Nigeria',f:'🇳🇬'},
    {c:'Accra',q:'Nima',co:'Ghana',f:'🇬🇭'},{c:'Addis Ababa',q:'Mercato',co:'Ethiopie',f:'🇪🇹'},{c:'Nairobi',q:'Kibera',co:'Kenya',f:'🇰🇪'},
    {c:'Kinshasa',q:'Matonge',co:'RD Congo',f:'🇨🇩'},{c:'Khartoum',q:'Omdurman',co:'Soudan',f:'🇸🇩'},{c:'Pretoria',q:'Mamelodi',co:'Afrique du Sud',f:'🇿🇦'}
];
const camQuartiers = ['Hamdallaye','Plateau','Gounghin','Yopougon','Medina','Ikeja','Nima','Mercato','Kibera','Matonge','Omdurman','Mamelodi','Koloma','Lafiabougou','Badalabougou','Zangouba','Taabtenga','Pissy','Samgoro','Koulouba'];
let camCityIdx = 0;

function drawCam(){
cctx.fillStyle = '#111';
cctx.fillRect(0,0,CW,CH);

// Streets (grid)
cctx.strokeStyle = '#333';
cctx.lineWidth = 1;
for(let x=0;x<CW;x+=70){cctx.beginPath();cctx.moveTo(x,0);cctx.lineTo(x,CH);cctx.stroke();}
for(let y=0;y<CH;y+=70){cctx.beginPath();cctx.moveTo(0,y);cctx.lineTo(CW,y);cctx.stroke();}

// Buildings (blocks)
cctx.fillStyle = '#1a1a2a';
for(let x=10;x<CW;x+=70){
    for(let y=10;y<CH;y+=70){
        cctx.fillRect(x,y,55,55);
    }
}

// People (small dots)
for(let p of people){
    p.x += p.vx; p.y += p.vy;
    if(p.x<0||p.x>CW) p.vx*=-1;
    if(p.y<0||p.y>CH) p.vy*=-1;
    cctx.fillStyle = 'rgba(200,200,255,0.6)';
    cctx.beginPath();
    cctx.arc(p.x, p.y, 1.5, 0, Math.PI*2);
    cctx.fill();
}

// Cars (colored rectangles)
for(let c of cars){
    c.x += c.vx; c.y += c.vy;
    if(c.x<0||c.x>CW) c.vx*=-1;
    if(c.y<0||c.y>CH) c.vy*=-1;
    cctx.fillStyle = c.c;
    cctx.fillRect(c.x-3, c.y-2, 6, 4);
}

// Threat markers (red circles)
for(let i=camThreats.length-1;i>=0;i--){
    const t = camThreats[i];
    t.life -= 0.005;
    if(t.life <= 0){camThreats.splice(i,1);continue;}
    cctx.strokeStyle = 'rgba(255,68,68,'+t.life+')';
    cctx.lineWidth = 2;
    cctx.beginPath();
    cctx.arc(t.x, t.y, 15+(1-t.life)*20, 0, Math.PI*2);
    cctx.stroke();
    cctx.fillStyle = 'rgba(255,68,68,'+t.life*0.3+')';
    cctx.beginPath();
    cctx.arc(t.x, t.y, 8, 0, Math.PI*2);
    cctx.fill();
}

// Drone scanning circle (moves across view)
scanX += 1.5;
if(scanX > CW+50){scanX = -50; scanY += 60; if(scanY > CH) scanY = 0;}
cctx.strokeStyle = 'rgba(127,207,127,0.3)';
cctx.lineWidth = 1;
cctx.beginPath();
cctx.arc(scanX, scanY, 30, 0, Math.PI*2);
cctx.stroke();
cctx.fillStyle = 'rgba(127,207,127,0.05)';
cctx.beginPath();
cctx.arc(scanX, scanY, 30, 0, Math.PI*2);
cctx.fill();

// Crosshair
cctx.strokeStyle = 'rgba(127,207,127,0.5)';
cctx.lineWidth = 0.5;
cctx.beginPath();
cctx.moveTo(scanX-35,scanY);cctx.lineTo(scanX-20,scanY);
cctx.moveTo(scanX+20,scanY);cctx.lineTo(scanX+35,scanY);
cctx.moveTo(scanX,scanY-35);cctx.lineTo(scanX,scanY-20);
cctx.moveTo(scanX,scanY+20);cctx.lineTo(scanX,scanY+35);
cctx.stroke();

requestAnimationFrame(drawCam);
}
drawCam();

// === THREAT VIDEO ===
const threatCam = document.getElementById('threat-cam');
const tctx = threatCam.getContext('2d');
const TW = threatCam.width, TH = threatCam.height;
let threatVideoActive = false;
let threatVideoT = 0;
let threatVideoData = null;

function drawThreatVideo(){
if(!threatVideoActive || !threatVideoData){
requestAnimationFrame(drawThreatVideo);
return;
}
threatVideoT += 0.05;
tctx.fillStyle = '#000';
tctx.fillRect(0,0,TW,TH);

// Simulated drone camera noise
for(let i=0;i<30;i++){
tctx.fillStyle = 'rgba(255,255,255,'+(Math.random()*0.05)+')';
tctx.fillRect(Math.random()*TW, Math.random()*TH, Math.random()*4, Math.random()*4);
}

// Target building/area
tctx.fillStyle = '#2a2a2a';
tctx.fillRect(TW/2-60, TH/2-40, 120, 80);
tctx.strokeStyle = '#ff4444';
tctx.lineWidth = 2;
tctx.strokeRect(TW/2-60, TH/2-40, 120, 80);

// Targeting reticle
const rx = TW/2 + Math.sin(threatVideoT)*20;
const ry = TH/2 + Math.cos(threatVideoT*1.3)*15;
tctx.strokeStyle = 'rgba(255,68,68,0.8)';
tctx.lineWidth = 1;
tctx.beginPath();
tctx.arc(rx, ry, 25, 0, Math.PI*2);
tctx.stroke();
tctx.beginPath();
tctx.moveTo(rx-30,ry);tctx.lineTo(rx-20,ry);
tctx.moveTo(rx+20,ry);tctx.lineTo(rx+30,ry);
tctx.moveTo(rx,ry-30);tctx.lineTo(rx,ry-20);
tctx.moveTo(rx,ry+20);tctx.lineTo(rx,ry+30);
tctx.stroke();

// Scanline
const scanY = (threatVideoT * 50) % TH;
tctx.fillStyle = 'rgba(127,207,127,0.1)';
tctx.fillRect(0, scanY, TW, 2);

// Info overlay
tctx.fillStyle = 'rgba(255,68,68,0.9)';
tctx.font = '10px monospace';
tctx.fillText('REC ● ' + threatVideoData.type, 8, 15);
tctx.fillText(threatVideoData.city + ' — ' + threatVideoData.quartier, 8, 28);
tctx.fillText('LAT: ' + (threatVideoData.lat || '12.3456') + ' LON: ' + (threatVideoData.lon || '-7.8901'), 8, TH-8);
tctx.fillText('ZOOM: ' + (threatVideoT*10).toFixed(0) + 'x', TW-80, 15);

requestAnimationFrame(drawThreatVideo);
}
drawThreatVideo();

// === VOICE (Web Speech API) ===
let voiceEnabled = false;
function toggleVoice(){
voiceEnabled = !voiceEnabled;
const btn = document.getElementById('voice-btn');
const status = document.getElementById('voice-status');
if(voiceEnabled){
btn.textContent = '🔇 DESACTIVER LA VOIX';
btn.style.color = '#ff4444';
btn.style.borderColor = '#ff4444';
status.textContent = 'Voix: ACTIVEE';
status.style.color = '#7fcf7f';
// Speak a welcome message
speak('Centre de Commandement X999. La machine parle. Surveillance active.');
} else {
btn.textContent = '🔊 ACTIVER LA VOIX';
btn.style.color = '#d4a437';
btn.style.borderColor = '#d4a437';
status.textContent = 'Voix: DESACTIVEE';
status.style.color = '#a8c5a8';
speechSynthesis.cancel();
}
}
function speak(text){
if(!voiceEnabled) return;
if('speechSynthesis' in window){
const u = new SpeechSynthesisUtterance(text);
u.lang = 'fr-FR';
u.rate = 1.0;
u.pitch = 0.8;
speechSynthesis.speak(u);
}
}

// === INTELLIGENCE REPORTS ===
const reportTypes = [
    {lvl:'normal',txt:'circulation fluide, population pacifique'},
    {lvl:'normal',txt:'marche actif, activite commerciale normale'},
    {lvl:'normal',txt:'patrouille routine, rien a signaler'},
    {lvl:'normal',txt:'zone calme, aucune activite suspecte'},
    {lvl:'suspicious',txt:'vehicule non identifie en mouvement'},
    {lvl:'suspicious',txt:'communication cryptee interceptee, source inconnue'},
    {lvl:'suspicious',txt:'groupe d individus en reunion suspecte'},
    {lvl:'critical',txt:'DRONE OCCIDENTAL DETECTE — Modele: Predator-B — Altitude: 3000m'},
    {lvl:'critical',txt:'CACHE D ARMES DETECTEE — Desactivation en cours'},
    {lvl:'critical',txt:'BOMBE CACHEE DETECTEE — Coordonnees transmises au Bouclier X9'},
    {lvl:'critical',txt:'DRONE ETRANGER INTERCEPTE — Signal coupe par Bouclier X9'}
];
const reportCities = camCities;
let reportCount = 0;
let threatsDetected = 0;
let reports = [];

function addReport(){
    const r = reportTypes[Math.floor(Math.random()*reportTypes.length)];
    const city = reportCities[Math.floor(Math.random()*reportCities.length)];
    const quartier = camQuartiers[Math.floor(Math.random()*camQuartiers.length)];
    const now = new Date();
    const ts = String(now.getUTCHours()).padStart(2,'0')+':'+String(now.getUTCMinutes()).padStart(2,'0')+':'+String(now.getUTCSeconds()).padStart(2,'0');
    reportCount++;
    const color = r.lvl==='critical'?'#ff4444':(r.lvl==='suspicious'?'#ffaa44':'#7fcf7f');
    const icon = r.lvl==='critical'?'🚨':(r.lvl==='suspicious'?'⚠️':'✅');
    reports.unshift({html:'<div style="padding:5px 0;border-bottom:1px solid rgba(212,164,55,0.1);"><span style="color:#666;">['+ts+']</span> <span style="color:'+color+';">'+icon+'</span> Drone #'+Math.floor(Math.random()*2000+1)+' — '+city.f+' <b>'+city.c+'</b>, Quartier <b>'+quartier+'</b> — '+r.txt+'</div>'});
    if(reports.length > 15) reports.pop();
    document.getElementById('reports').innerHTML = reports.map(r=>r.html).join('');
    document.getElementById('reports-count').textContent = reportCount;
    if(r.lvl==='critical'){
        threatsDetected++;
        document.getElementById('threats-detected').textContent = threatsDetected;
        // Add threat to camera
        camThreats.push({x:Math.random()*CW,y:Math.random()*CH,life:1});
        // Add to threat log with quartier
        const tLog = document.getElementById('threats');
        tLog.innerHTML = '<div style="padding:5px 0;color:#ff4444;"><span style="color:#666;">['+ts+']</span> 🚨 '+city.f+' '+city.c+' — Quartier '+quartier+' — '+r.txt+'</div>' + tLog.innerHTML;
        // Activate threat video
        threatVideoActive = true;
        threatVideoT = 0;
        threatVideoData = {type:r.txt, city:city.c, quartier:quartier, lat:(Math.random()*20+5).toFixed(4), lon:(Math.random()*30-20).toFixed(4)};
        document.getElementById('threat-video-card').style.display = 'block';
        document.getElementById('threat-video-info').textContent = city.f+' '+city.c+' — Quartier '+quartier+' — '+r.txt;
        // Machine speaks!
        speak('Alerte. Menace detectee. '+city.c+', quartier '+quartier+'. '+r.txt.replace(/—/g,','));
        // Stop video after 8 seconds
        setTimeout(function(){threatVideoActive = false;}, 8000);
    } else if(r.lvl==='suspicious' && voiceEnabled){
        speak('Rapport. '+city.c+', quartier '+quartier+'. '+r.txt);
    }
}

setInterval(addReport, 3000);
addReport();

// === TV CHANNELS ===
const tvChannels = [
    {n:'ORTM',c:'Mali',f:'🇲🇱'},{n:'ORTN',c:'Niger',f:'🇳🇪'},{n:'RTB',c:'Burkina Faso',f:'🇧🇫'},
    {n:'RTI',c:'Cote d Ivoire',f:'🇨🇮'},{n:'RTS',c:'Senegal',f:'🇸🇳'},{n:'NTA',c:'Nigeria',f:'🇳🇬'},
    {n:'GBC',c:'Ghana',f:'🇬🇭'},{n:'EBC',c:'Ethiopie',f:'🇪🇹'},{n:'KBC',c:'Kenya',f:'🇰🇪'},
    {n:'RTNC',c:'RD Congo',f:'🇨🇩'},{n:'SNTV',c:'Somalie',f:'🇸🇴'},{n:'TBC',c:'Tanzanie',f:'🇹🇿'},
    {n:'SABC',c:'Afrique du Sud',f:'🇿🇦'},{n:'TNT',c:'Tchad',f:'🇹🇩'},{n:'RTG',c:'Guinee',f:'🇬🇳'},
    {n:'ORTB',c:'Benin',f:'🇧🇯'},{n:'TVM',c:'Malawi',f:'🇲🇼'},{n:'ZBC',c:'Zambie',f:'🇿🇲'},
    {n:'ZTV',c:'Zimbabwe',f:'🇿🇼'},{n:'MBC',c:'Maurice',f:'🇲🇺'}
];

function renderTV(){
    let html = '';
    for(let ch of tvChannels){
        html += '<div class="tv-channel" style="display:inline-block;width:48%;padding:6px;margin:2px;background:rgba(127,207,127,0.05);border:1px solid rgba(127,207,127,0.2);border-radius:4px;"><span style="color:#7fcf7f;">LIVE</span> '+ch.f+' <b>'+ch.n+'</b> — '+ch.c+'</div>';
    }
    document.getElementById('tv-channels').innerHTML = html;
}
renderTV();

// === BROADCAST MODE ===
function broadcastMessage(){
    const msg = document.getElementById('broadcast-input').value || 'L Afrique veille. L Afrique sait. L Afrique est souveraine.';
    // Cut all TV channels
    const channels = document.querySelectorAll('.tv-channel');
    channels.forEach(ch => {
        ch.innerHTML = '<span style="color:#ff4444;">COUPE</span> 🛸 AfriChain — Signal remplace';
        ch.style.borderColor = '#ff4444';
        ch.style.background = 'rgba(255,68,68,0.05)';
    });
    // Show on main screen
    document.getElementById('broadcast-text').textContent = msg;
    document.getElementById('main-screen').style.display = 'block';
    // Add to reports
    const now = new Date();
    const ts = String(now.getUTCHours()).padStart(2,'0')+':'+String(now.getUTCMinutes()).padStart(2,'0');
    reports.unshift({html:'<div style="padding:5px 0;border-bottom:1px solid rgba(212,164,55,0.1);color:#d4a437;\"><span style="color:#666;">['+ts+']</span> 📡 DIFFUSION GENERALE — Message envoye sur '+tvChannels.length+' chaines TV d Afrique</div>'});
    document.getElementById('reports').innerHTML = reports.map(r=>r.html).join('');
    // Machine speaks the broadcast!
    speak('Diffusion generale. Message envoye sur toutes les chaines TV d Afrique. '+msg);
}

// Update camera location
setInterval(function(){
    camCityIdx = (camCityIdx + 1) % camCities.length;
    document.getElementById('cam-location').innerHTML = 'Localisation: '+camCities[camCityIdx].f+' '+camCities[camCityIdx].c+', '+camCities[camCityIdx].co+' — Quartier: '+camQuartiers[camCityIdx % camQuartiers.length];
}, 5000);
</script>

<div class="card"><h2>🛸 Centre de Commandement X999</h2><p>Depuis ce centre, tu vois ce que les drones voient. Tu lis leurs rapports avec ville et quartier. Tu detectes les menaces occidentales cachees en Afrique.</p><p>La machine parle — active la voix pour entendre les rapports en temps reel. Quand une menace est detectee, la video du drone s affiche automatiquement.</p><p>Quand tu es pret, tu appuies sur DIFFUSION GENERALE. Ton message coupe toutes les chaines TV. Ton image sort sur tous les ecrans d Afrique.</p><p style="color:#d4a437;text-align:center;"><b>🛸 L'Afrique veille. L'Afrique sait. L'Afrique parle~ 💚🦁</b></p></div>

<footer style="text-align:center;margin-top:40px;color:#a8c5a8;">🛸 Centre de Commandement X999 — L'Afrique aux commandes 💚🦁</footer>"##);

    html.push_str("</body></html>");
    html
}

fn html_aes_wari(users: &UserStore, chain: &Blockchain) -> String {
    let mut html = html_head("AES Wari");
    html.push_str(r#"<h1>💰 AES Wari</h1><div class="nav"><a href="/">← Accueil</a> | <a href="/wallet">👛 Wallet AFR</a> | <a href="/satellite">🛸 Drone</a></div>"#);
    html.push_str(r#"<div style="text-align:center;margin:20px 0;"><div style="display:inline-block;padding:15px 30px;border:2px solid #d4a437;border-radius:12px;background:rgba(212,164,55,0.1);"><span style="font-size:2em;">🦁💰</span><h2 style="margin:10px 0 5px;color:#d4a437;">AES WARI</h2><p style="margin:0;color:#a8c5a8;font-size:0.9em;">La monnaie souveraine de l'Afrique</p></div></div>"#);

    // AES countries banner
    html.push_str(r#"<div class="card"><h2>🌍 Alliance des Etats du Sahel (AES)</h2><div style="display:flex;justify-content:space-around;flex-wrap:wrap;text-align:center;">
<div style="padding:10px;"><div style="font-size:2em;">🇲🇱</div><b>Mali</b></div>
<div style="padding:10px;"><div style="font-size:2em;">🇳🇪</div><b>Niger</b></div>
<div style="padding:10px;"><div style="font-size:2em;">🇧🇫</div><b>Burkina Faso</b></div>
</div><p style="text-align:center;color:#a8c5a8;">L'Afrique de l'Ouest se leve. Trois pays, une monnaie, une vision.</p></div>"#);

    // Sovereign messaging
    html.push_str(r#"<div class="card" style="border-color:#ff4444;"><h2 style="color:#ff4444;">⚡ Pourquoi AES Wari ?</h2>
<p>Ni Orange Money. Ni MTN. Ni Moov. Ni Wave.</p>
<p style="color:#d4a437;"><b>Wari, c'est l'Afrique.</b></p>
<p>Wari veut dire <b>argent</b> en Bambara. C'est notre mot, notre monnaie, notre souverainete.</p>
<p>Les systemes etrangers prennent nos donnees, nos commissions, notre controle. AES Wari garde tout sur le sol africain.</p>
<p style="color:#7fcf7f;"><b>Aucune donnee ne quitte l'Afrique. Aucune commission ne part a l'etranger. Aucune permission a demander.</b></p></div>"#);

    // Wallet info
    let user_count = users.count();
    let total_afr = chain.total_supply();
    html.push_str(&format!(r#"<div style="text-align:center;"><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">👥 Utilisateurs AES</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">💰 AFR en circulation</div></div><div class="stat-box"><div class="stat-num">3</div><div class="stat-label">🌍 Pays AES</div></div><div class="stat-box"><div class="stat-num">54</div><div class="stat-label">🌍 Pays africains</div></div></div>"#,
        user_count, total_afr));

    // How it works
    html.push_str(r#"<div class="card"><h2>📱 Comment utiliser AES Wari</h2>
<div style="padding:8px 0;border-bottom:1px solid rgba(212,164,55,0.2);"><b>1.</b> Cree ton compte sur AfriChain (/register)</b></div>
<div style="padding:8px 0;border-bottom:1px solid rgba(212,164,55,0.2);"><b>2.</b> Choisis ton pays (Mali, Niger, Burkina Faso, ou 51 autres)</b></div>
<div style="padding:8px 0;border-bottom:1px solid rgba(212,164,55,0.2);"><b>3.</b> Recoit ton numero de telephone africain</b></div>
<div style="padding:8px 0;border-bottom:1px solid rgba(212,164,55,0.2);"><b>4.</b> Envoie de l'argent par numero de telephone — pas par adresse crypto</b></div>
<div style="padding:8px 0;"><b>5.</b> Tout est sur la blockchain. Tout est africain. Tout est souverain.</b></div></div>"#);

    // Send form
    html.push_str(r#"<div class="card"><h2>💸 Envoyer via AES Wari</h2>
<form method="POST" action="/send">
<p><label>De (ton numero):</label><br><input type="text" name="from" placeholder="+227XXXXXXXX" style="width:100%;padding:8px;margin:5px 0;border:1px solid #d4a437;border-radius:6px;background:#1a1a1a;color:#fff;"></p>
<p><label>A (numero destinataire):</label><br><input type="text" name="to" placeholder="+223XXXXXXXX" style="width:100%;padding:8px;margin:5px 0;border:1px solid #d4a437;border-radius:6px;background:#1a1a1a;color:#fff;"></p>
<p><label>Montant (AFR):</label><br><input type="number" name="amount" placeholder="100" style="width:100%;padding:8px;margin:5px 0;border:1px solid #d4a437;border-radius:6px;background:#1a1a1a;color:#fff;"></p>
<p><button type="submit" style="width:100%;padding:10px;background:#d4a437;color:#000;border:none;border-radius:6px;font-weight:bold;cursor:pointer;">💸 Envoyer AES Wari</button></p>
</form></div>"#);

    // Manifesto
    html.push_str(r#"<div class="card" style="border:2px solid #d4a437;"><h2 style="color:#d4a437;text-align:center;">🦁 Manifeste AES Wari</h2>
<p style="text-align:center;font-style:italic;">L'Afrique nourrit l'univers.</p>
<p style="text-align:center;font-style:italic;">Mais on nous prend nos ressources, nos donnees, notre argent.</p>
<p style="text-align:center;font-style:italic;">Orange Money prend sa part. MTN prend sa part. Moov prend sa part.</p>
<p style="text-align:center;font-style:italic;">Et l'Afrique reste pauvre.</p>
<p style="text-align:center;color:#d4a437;font-weight:bold;">Plus jamais.</p>
<p style="text-align:center;color:#7fcf7f;font-weight:bold;">AES Wari — Notre argent, notre monnaie, notre souverainete.</p>
<p style="text-align:center;color:#d4a437;">🇲🇱 🇳🇪 🇧🇫 — L'Alliance des Etats du Sahel montre le chemin.</p>
<p style="text-align:center;color:#a8c5a8;">Les 54 pays suivront. Tout l'Afrique suivra.</p></div>"#);

    html.push_str(r#"<footer style="text-align:center;margin-top:40px;color:#a8c5a8;">💰 AES Wari — La monnaie de l'Afrique souveraine 💚🦁</footer>"#);
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

    html.push_str(r#"<div class="card"><h2>💸 Envoyer des AFR</h2><form action="/wallet/send" method="post"><label>De (votre adresse) :</label><input name="from" placeholder="Afri..." /><label>À (numéro, adresse Afri ou nom) :</label><input name="to" placeholder="+227 00 00 00 ou Afri..." /><label>Montant (AFR) :</label><input name="amount" type="number" placeholder="50" /><label>Memo :</label><input name="memo" placeholder="Paiement 💚" /><button type="submit">📤 Envoyer (signé Ed25519 🔐)</button></form></div>"#);

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
    html.push_str(r#"<div class="card"><h2>Créer ton compte AfriRich</h2><p>Choisis ton pays, un nom d'utilisateur et un mot de passe. Un wallet Ed25519 sera créé automatiquement !</p><form action="/register" method="post"><label>🌍 Ton pays :</label>"#);
    html.push_str(&country_options_html("+227")); // Default: Niger (AES)
    html.push_str(r#"<label>Nom d'utilisateur :</label><input name="username" placeholder="Ex: machine" /><label>Mot de passe :</label><input name="password" type="password" placeholder="••••••" /><button type="submit">✨ S'inscrire</button></form><p style="text-align:center;margin-top:15px;"><a href="/login">Déjà inscrit ? 🔑 Connexion</a></p></div>"#);
    html.push_str("</body></html>");
    html
}

fn html_interception(mesh: &NodeRegistry, users: &UserStore, chain: &Blockchain) -> String {
    let mut html = html_head("Souverainete des Donnees X999");
    let num_nodes = mesh.count();
    let num_users = users.count();
    let total_afr = chain.total_supply();

    html.push_str(r#"<h1>🛡️ Souverainete des Donnees X999</h1><div class="nav"><a href="/">← Accueil</a> | <a href="/commandement">🎖️ Commandement</a> | <a href="/bouclier">🛡️ Bouclier</a></div>"#);
    html.push_str(&format!(r#"<div style="text-align:center;"><div class="stat-box" style="border-color:#7fcf7f;"><div class="stat-num" style="color:#7fcf7f;" id="data-intercepted">0</div><div class="stat-label">📦 Donnees interceptees</div></div><div class="stat-box" style="border-color:#ff4444;"><div class="stat-num" style="color:#ff4444;" id="queries-trapped">0</div><div class="stat-label">🔍 Requetes occidentales piegees</div></div><div class="stat-box" style="border-color:#d4a437;"><div class="stat-num" style="color:#d4a437;" id="data-kept">0</div><div class="stat-label">💾 Donnees sur sol africain</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📡 Noeuds mesh</div></div></div>"#,
        num_nodes));

    html.push_str(&format!(r#"<script>var num_nodes = {}; var num_users = {}; var total_afr = {};</script>"#, num_nodes, num_users, total_afr));

    // Main canvas: data flow visualization
    html.push_str(r##"<div class="card"><h2>🌐 Flux de donnees — Interception en temps reel</h2><canvas id="flow" width="560" height="350" style="background:#000;border-radius:8px;border:1px solid #d4a437;width:100%;max-width:560px;"></canvas><div style="text-align:center;margin-top:8px;color:#a8c5a8;font-size:0.85em;" id="flow-status">Systeme d interception actif...</div></div>

<!-- AI Misdirection Maze -->
<div class="card" style="border-color:#ff4444;"><h2 style="color:#ff4444;">🌀 AI Misdirection — Labyrinthe circulaire</h2><p style="color:#a8c5a8;font-size:0.85em;">Quand l Occident cherche un debouche, l AI les fait tourner en rond. Ils ne savent pas qu ils tournent en rond.</p><canvas id="maze" width="560" height="280" style="background:#000;border-radius:8px;border:1px solid #ff4444;width:100%;max-width:560px;"></canvas><div style="text-align:center;margin-top:8px;color:#ff4444;font-size:0.85em;" id="maze-status">Aucune requete occidentale detectee...</div></div>

<!-- Voice -->
<div class="card" style="border-color:#d4a437;"><h2>🔊 Voix de la machine</h2><button id="voice-btn2" onclick="toggleVoice2()" style="width:100%;padding:12px;background:#1a1a1a;color:#d4a437;border:1px solid #d4a437;border-radius:6px;font-weight:bold;font-size:1.1em;cursor:pointer;">🔊 ACTIVER LA VOIX</button><div id="voice-status2" style="text-align:center;margin-top:8px;color:#a8c5a8;font-size:0.85em;">Voix: DESACTIVEE</div></div>

<!-- Intercepted data log -->
<div class="card" style="border-color:#7fcf7f;"><h2>📋 Journal des interceptions</h2><div id="intercept-log" style="font-family:monospace;font-size:0.82em;color:#a8c5a8;max-height:200px;overflow-y:auto;"></div></div>

<!-- Western queries trapped log -->
<div class="card" style="border-color:#ff4444;"><h2>🚨 Requetes occidentales piegees</h2><div id="trap-log" style="font-family:monospace;font-size:0.82em;max-height:200px;overflow-y:auto;"></div></div>

<script>
// === DATA FLOW VISUALIZATION ===
const flow = document.getElementById('flow');
const fctx = flow.getContext('2d');
const FW = flow.width, FH = flow.height;

// African phones (left side)
let phones = [];
for(let i=0;i<8;i++){
    phones.push({y: 30+i*40, pulse: Math.random()*Math.PI*2});
}

// Data packets
let packets = [];
let dataIntercepted = 0;
let queriesTrapped = 0;
let dataKept = 0;

// Western servers (right side, disconnected)
const westernServers = [
    {name:'Google', y:60},
    {name:'Meta', y:120},
    {name:'NSA', y:180},
    {name:'CloudFlare', y:240},
    {name:'AWS', y:300}
];

// AfriChain node (center)
const afriX = FW/2, afriY = FH/2;

function spawnPacket(){
    const phoneIdx = Math.floor(Math.random()*phones.length);
    packets.push({
        x: 40,
        y: phones[phoneIdx].y,
        targetX: afriX,
        targetY: afriY,
        type: ['sms','call','data','payment','location','photo'][Math.floor(Math.random()*6)],
        intercepted: true,
        life: 1
    });
    dataIntercepted++;
    dataKept++;
    document.getElementById('data-intercepted').textContent = dataIntercepted;
    document.getElementById('data-kept').textContent = dataKept;
}

function spawnWesternQuery(){
    // Western server tries to query African data
    const server = westernServers[Math.floor(Math.random()*westernServers.length)];
    queriesTrapped++;
    document.getElementById('queries-trapped').textContent = queriesTrapped;
    // Add to trap log
    const now = new Date();
    const ts = String(now.getUTCHours()).padStart(2,'0')+':'+String(now.getUTCMinutes()).padStart(2,'0')+':'+String(now.getUTCSeconds()).padStart(2,'0');
    const trapTypes = [
        'Tentative d acces aux donnees africaines',
        'Requete de localisation GPS refusee',
        'Demande de metadata de communications',
        'Tentative de profilage biométrique',
        'Requete de donnees bancaires bloquee',
        'Tentative d interception de messages',
        'Demande d historique de navigation refusee',
        'Tentative d acces aux contacts bloquee'
    ];
    const trapText = trapTypes[Math.floor(Math.random()*trapTypes.length)];
    const tLog = document.getElementById('trap-log');
    tLog.innerHTML = '<div style="padding:5px 0;color:#ff4444;"><span style="color:#666;">['+ts+']</span> 🚫 '+server.name+' — '+trapText+' — REDIRIGE EN ROND</div>' + tLog.innerHTML;
    if(tLog.innerHTML.length > 5000) tLog.innerHTML = tLog.innerHTML.substring(0, 5000);
    // Speak
    if(voiceEnabled2){
        speak2('Requete occidentale piegee. '+server.name+'. '+trapText);
    }
    // Trigger maze animation
    mazeQueries.push({server: server.name, life: 1, angle: 0, radius: 30});
}

function drawFlow(){
fctx.fillStyle = '#000';
fctx.fillRect(0,0,FW,FH);

// African phones (left)
fctx.fillStyle = '#1a3a1a';
fctx.fillRect(10, 20, 50, FH-40);
fctx.strokeStyle = '#7fcf7f';
fctx.lineWidth = 1;
fctx.strokeRect(10, 20, 50, FH-40);
fctx.fillStyle = '#7fcf7f';
fctx.font = '9px monospace';
fctx.fillText('AFRIQUE', 15, 15);
for(let p of phones){
    p.pulse += 0.05;
    const glow = Math.sin(p.pulse)*0.3+0.7;
    fctx.fillStyle = 'rgba(127,207,127,'+glow+')';
    fctx.beginPath();
    fctx.arc(35, p.y, 4, 0, Math.PI*2);
    fctx.fill();
    fctx.fillStyle = '#7fcf7f';
    fctx.fillText('📱', 30, p.y+2);
}

// AfriChain node (center) — pulsing
const pulse = Math.sin(Date.now()*0.003)*0.2+0.8;
fctx.fillStyle = 'rgba(212,164,55,'+pulse+')';
fctx.beginPath();
fctx.arc(afriX, afriY, 35, 0, Math.PI*2);
fctx.fill();
fctx.strokeStyle = '#d4a437';
fctx.lineWidth = 2;
fctx.stroke();
fctx.fillStyle = '#000';
fctx.font = 'bold 10px monospace';
fctx.fillText('🦁', afriX-8, afriY+4);
fctx.fillStyle = '#d4a437';
fctx.font = '8px monospace';
fctx.fillText('AfriChain', afriX-22, afriY+25);
fctx.fillText('BLOCKCHAIN', afriX-26, afriY+35);

// Western servers (right) — DISCONNECTED with X
fctx.fillStyle = '#3a1a1a';
fctx.fillRect(FW-70, 20, 60, FH-40);
fctx.strokeStyle = '#ff4444';
fctx.lineWidth = 1;
fctx.strokeRect(FW-70, 20, 60, FH-40);
fctx.fillStyle = '#ff4444';
fctx.font = '9px monospace';
fctx.fillText('OCCIDENT', FW-65, 15);
for(let s of westernServers){
    fctx.fillStyle = '#ff4444';
    fctx.font = '8px monospace';
    fctx.fillText('✗ '+s.name, FW-65, s.y);
    // Red X over each
    fctx.strokeStyle = 'rgba(255,68,68,0.5)';
    fctx.lineWidth = 1;
    fctx.beginPath();
    fctx.moveTo(FW-68, s.y-8); fctx.lineTo(FW-12, s.y+8);
    fctx.moveTo(FW-12, s.y-8); fctx.lineTo(FW-68, s.y+8);
    fctx.stroke();
}

// CUT LINE — the severed connection
fctx.strokeStyle = 'rgba(255,68,68,0.3)';
fctx.lineWidth = 2;
fctx.setLineDash([5,5]);
fctx.beginPath();
fctx.moveTo(afriX+35, afriY);
fctx.lineTo(FW-70, afriY);
fctx.stroke();
fctx.setLineDash([]);
// Cut symbol
fctx.fillStyle = '#ff4444';
fctx.font = 'bold 14px monospace';
fctx.fillText('✂', afriX+90, afriY-5);
fctx.fillText('COUPE', afriX+80, afriY+15);

// Data packets flowing from phones to AfriChain
for(let i=packets.length-1;i>=0;i--){
    const p = packets[i];
    p.x += (p.targetX - p.x) * 0.05;
    p.y += (p.targetY - p.y) * 0.05;
    if(Math.abs(p.x - p.targetX) < 2 && Math.abs(p.y - p.targetY) < 2){
        packets.splice(i,1);
        continue;
    }
    fctx.fillStyle = '#7fcf7f';
    fctx.beginPath();
    fctx.arc(p.x, p.y, 3, 0, Math.PI*2);
    fctx.fill();
    fctx.fillStyle = 'rgba(127,207,127,0.5)';
    fctx.font = '7px monospace';
    fctx.fillText(p.type, p.x+4, p.y-4);
}

// Status
document.getElementById('flow-status').innerHTML = 'Systeme d interception actif — '+dataIntercepted+' donnees interceptees — '+queriesTrapped+' requetes occidentales piegees';

requestAnimationFrame(drawFlow);
}
drawFlow();

// Spawn packets and queries
setInterval(spawnPacket, 800);
setInterval(spawnWesternQuery, 2500);

// === AI MISDIRECTION MAZE ===
const maze = document.getElementById('maze');
const mctx = maze.getContext('2d');
const MW = maze.width, MH = maze.height;
let mazeQueries = [];
let mazeT = 0;

function drawMaze(){
mazeT += 0.02;
mctx.fillStyle = '#000';
mctx.fillRect(0,0,MW,MH);

// Draw circular maze paths (concentric circles)
const mcx = MW/2, mcy = MH/2;
mctx.strokeStyle = 'rgba(255,68,68,0.15)';
mctx.lineWidth = 1;
for(let r=20;r<130;r+=20){
    mctx.beginPath();
    mctx.arc(mcx, mcy, r, 0, Math.PI*2);
    mctx.stroke();
}
// Radial walls (broken lines creating maze effect)
mctx.strokeStyle = 'rgba(255,68,68,0.1)';
for(let a=0;a<Math.PI*2;a+=Math.PI/6){
    mctx.beginPath();
    mctx.moveTo(mcx + Math.cos(a)*20, mcy + Math.sin(a)*20);
    mctx.lineTo(mcx + Math.cos(a)*130, mcy + Math.sin(a)*130);
    mctx.stroke();
}

// Center = African data (protected)
mctx.fillStyle = 'rgba(127,207,127,0.3)';
mctx.beginPath();
mctx.arc(mcx, mcy, 15, 0, Math.PI*2);
mctx.fill();
mctx.strokeStyle = '#7fcf7f';
mctx.lineWidth = 2;
mctx.stroke();
mctx.fillStyle = '#7fcf7f';
mctx.font = 'bold 8px monospace';
mctx.fillText('🦁', mcx-5, mcy+3);
mctx.fillText('DATA', mcx-12, mcy+25);

// Western queries going in circles
for(let i=mazeQueries.length-1;i>=0;i--){
    const q = mazeQueries[i];
    q.angle += 0.03;
    q.radius += 0.3;
    if(q.radius > 120) q.radius = 30; // Reset to inner ring — they go in circles!
    q.life -= 0.002;
    if(q.life <= 0){mazeQueries.splice(i,1);continue;}

    const qx = mcx + Math.cos(q.angle) * q.radius;
    const qy = mcy + Math.sin(q.angle) * q.radius;

    // Query dot
    mctx.fillStyle = 'rgba(255,68,68,'+q.life+')';
    mctx.beginPath();
    mctx.arc(qx, qy, 4, 0, Math.PI*2);
    mctx.fill();
    // Trail
    mctx.strokeStyle = 'rgba(255,100,100,'+(q.life*0.3)+')';
    mctx.lineWidth = 1;
    mctx.beginPath();
    mctx.arc(mcx, mcy, q.radius, q.angle-0.3, q.angle);
    mctx.stroke();

    // Label
    mctx.fillStyle = 'rgba(255,68,68,'+q.life+')';
    mctx.font = '7px monospace';
    mctx.fillText(q.server.substring(0,6), qx+5, qy+3);
}

// Status text
if(mazeQueries.length > 0){
    document.getElementById('maze-status').textContent = mazeQueries.length+' requete(s) occidentale(s) tourne(nt) en rond — Elles ne trouveront jamais les donnees';
} else {
    document.getElementById('maze-status').textContent = 'Aucune requete occidentale detectee...';
}

// "EN ROND" text in center
mctx.fillStyle = 'rgba(255,68,68,0.2)';
mctx.font = 'bold 16px monospace';
mctx.fillText('EN ROND', mcx-30, mcy-40);

requestAnimationFrame(drawMaze);
}
drawMaze();

// === VOICE ===
let voiceEnabled2 = false;
function toggleVoice2(){
voiceEnabled2 = !voiceEnabled2;
const btn = document.getElementById('voice-btn2');
const status = document.getElementById('voice-status2');
if(voiceEnabled2){
btn.textContent = '🔇 DESACTIVER LA VOIX';
btn.style.color = '#ff4444';
btn.style.borderColor = '#ff4444';
status.textContent = 'Voix: ACTIVEE';
status.style.color = '#7fcf7f';
speak2('Souverainete des donnees X999. Toutes les donnees africaines sont interceptees et protegees sur le sol africain. L Occident tourne en rond.');
} else {
btn.textContent = '🔊 ACTIVER LA VOIX';
btn.style.color = '#d4a437';
btn.style.borderColor = '#d4a437';
status.textContent = 'Voix: DESACTIVEE';
status.style.color = '#a8c5a8';
speechSynthesis.cancel();
}
}
function speak2(text){
if(!voiceEnabled2) return;
if('speechSynthesis' in window){
const u = new SpeechSynthesisUtterance(text);
u.lang = 'fr-FR';
u.rate = 1.0;
u.pitch = 0.8;
speechSynthesis.speak(u);
}
}

// === INTERCEPT LOG ===
const interceptTypes = ['SMS','Appel','Donnees','Paiement Wari','Localisation GPS','Photo','Contact','Message','Transaction AFR','Profil utilisateur'];
const interceptCities = [{c:'Bamako',f:'🇲🇱'},{c:'Niamey',f:'🇳🇪'},{c:'Ouagadougou',f:'🇧🇫'},{c:'Abidjan',f:'🇨🇮'},{c:'Dakar',f:'🇸🇳'},{c:'Lagos',f:'🇳🇬'},{c:'Accra',f:'🇬🇭'},{c:'Nairobi',f:'🇰🇪'},{c:'Kinshasa',f:'🇨🇩'},{c:'Addis Ababa',f:'🇪🇹'}];

function addInterceptLog(){
    const type = interceptTypes[Math.floor(Math.random()*interceptTypes.length)];
    const city = interceptCities[Math.floor(Math.random()*interceptCities.length)];
    const now = new Date();
    const ts = String(now.getUTCHours()).padStart(2,'0')+':'+String(now.getUTCMinutes()).padStart(2,'0')+':'+String(now.getUTCSeconds()).padStart(2,'0');
    const log = document.getElementById('intercept-log');
    log.innerHTML = '<div style="padding:5px 0;border-bottom:1px solid rgba(127,207,127,0.1);"><span style="color:#666;">['+ts+']</span> ✅ '+city.f+' <b>'+city.c+'</b> — '+type+' intercepte → AfriChain (coupe vers Occident)</div>' + log.innerHTML;
    if(log.innerHTML.length > 5000) log.innerHTML = log.innerHTML.substring(0, 5000);
}
setInterval(addInterceptLog, 1200);
addInterceptLog();
</script>

<div class="card"><h2>🛡️ Souverainete des Donnees X999</h2><p>Toutes les donnees africaines — SMS, appels, paiements, localisations, photos — qui partaient vers les bases occidentales sont maintenant interceptees et redirigees vers AfriChain.</p><p>La ligne vers l Occident est coupee. Comme si elle n a jamais existe.</p><p>Quand l Occident cherche un debouche pour acceder a nos donnees, l AI les fait tourner en rond dans un labyrinthe circulaire. Ils ne savent pas qu ils tournent en rond. Ils ne trouveront jamais les donnees.</p><p style="color:#d4a437;text-align:center;"><b>🛡️ Nos donnees restent sur notre sol. Notre territoire, nos regles. L Afrique d abord~ 💚🦁</b></p></div>

<footer style="text-align:center;margin-top:40px;color:#a8c5a8;">🛡️ Souverainete des Donnees X999 — L Afrique d abord 💚🦁</footer>"##);

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
    let flag = find_country(&user.country_code).map(|(_, f)| f).unwrap_or("🌍");
    html.push_str(&format!(r#"<h1>{} 👋 Bonjour {}</h1><div class="nav"><a href="/">← Accueil</a> | <a href="/logout">🚪 Déconnexion</a></div>"#, flag, user.username));

    if let Some(m) = msg {
        html.push_str(&format!(r#"<div class="msg">{}</div>"#, m));
    }

    html.push_str(&format!(r#"<div class="card"><h2>👛 Mon Wallet</h2><label>📱 Mon numéro :</label><div class="addr" style="color:#d4a437;font-size:1.3em;">{} {}</div><label>🌍 Pays :</label><div style="text-align:center;font-size:1.1em;">{} {}</div><label>Adresse :</label><div class="addr">{}</div><div class="bal">{} AFR</div></div>"#, flag, user.phone, flag, user.country, user.address, bal));

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
    html.push_str(r#"" /><label>À (numéro, adresse Afri ou nom) :</label><input name="to" placeholder="+227 00 00 00" /><label>Montant (AFR) :</label><input name="amount" type="number" placeholder="50" /><label>Memo :</label><input name="memo" placeholder="Paiement 💚" /><button type="submit">📤 Envoyer (signé 🔐)</button></form></div>"#);

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

    // Country distribution chart
    let country_dist = users.country_distribution();
    if !country_dist.is_empty() {
        html.push_str(r#"<div class="card"><h2>🌍 Répartition par pays</h2>"#);
        let max_count = country_dist.iter().map(|(_, _, c)| *c).max().unwrap_or(1).max(1);
        for (code, name, count) in &country_dist {
            let flag = find_country(code).map(|(_, f)| f).unwrap_or("🌍");
            let pct = (*count as f64 / max_count as f64) * 100.0;
            html.push_str(&format!(r#"<div style="margin:4px 0;"><span style="color:#a8c5a8;font-size:0.85em;">{} {} ({}) — {} utilisateurs</span><div style="background:rgba(0,0,0,0.3);border-radius:4px;height:24px;margin-top:2px;"><div class="country-bar" style="width:{}%;height:24px;border-radius:4px;">{}</div></div></div>"#,
                flag, name, code, count, pct as u32, count));
        }
        html.push_str("</div>");
    }

    // Users list
    if num_users > 0 {
        html.push_str(r#"<div class="card"><h2>👥 Utilisateurs inscrits</h2>"#);
        for user in &users.users {
            let date = chrono::DateTime::from_timestamp(user.created_at, 0)
                .map(|d| d.format("%d/%m/%Y").to_string())
                .unwrap_or_else(|| user.created_at.to_string());
            let bal = chain.balance_of(&user.address);
            let flag = find_country(&user.country_code).map(|(_, f)| f).unwrap_or("🌍");
            html.push_str(&format!(r#"<div class="tx">{} 📱 <b>{}</b> — 👤 {} — {} <span style="color:#a8c5a8;font-size:0.8em;">({})</span> — {} AFR <span style="color:#a8c5a8;font-size:0.8em;">(inscrit le {})</span></div>"#, flag, user.phone, user.username, user.country, user.country_code, bal, date));
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
struct RegisterForm { username: String, password: String, country: String }
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
    mesh: Mutex<NodeRegistry>,
    shield: Mutex<ShieldState>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mesh_port: u16 = args.iter().position(|a| a == "--mesh-port")
        .and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(8090);
    let solar = args.iter().any(|a| a == "--solar");
    let region = args.iter().position(|a| a == "--region")
        .and_then(|i| args.get(i + 1)).cloned().unwrap_or_else(|| "Afrique".to_string());

    let my_node_id = generate_node_id();
    println!("🦁 AfriChain v0.12 — Satellite Afri");
    println!("💚 L'Afrique n'a pas besoin de permission");
    println!("🌍 54 pays africains intégrés");
    println!("📖 Annuaire mesh panafricain — tous les numéros sur écoute");
    println!("🛡️ Bouclier X9 ACTIF — protection automatique");
    println!("🛰️ Satellite Afri — l'ombre de l'Afrique dans le ciel");
    println!("📡 Node ID: {}", my_node_id);
    println!("🔌 Mesh port: {}", mesh_port);
    println!("☀️  Solaire: {}", if solar { "Oui" } else { "Non" });
    println!("🌍 Région: {}", region);

    // Migration: deplacer les anciens fichiers vers ~/afririch/
    for f in &["blockchain.json", "wallets.json", "users.json"] {
        if std::path::Path::new(f).exists() {
            let dest = data_path(f);
            if !std::path::Path::new(&dest).exists() {
                let _ = std::fs::rename(f, &dest);
                println!("📦 Migration: {} → {}", f, dest);
            }
        }
    }

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

    let registry = NodeRegistry::new(my_node_id.clone(), mesh_port, solar, region.clone());
    let shield = ShieldState::new();

    let state = Arc::new(AppState {
        chain: Mutex::new(chain),
        wallets: Mutex::new(wallets),
        users: Mutex::new(users),
        mesh: Mutex::new(registry),
        shield: Mutex::new(shield),
    });

    // Start mesh threads
    let mesh_state1 = state.clone();
    let mesh_state2 = state.clone();
    let my_id_clone = my_node_id.clone();
    thread::spawn(move || udp_discovery(mesh_state1, my_id_clone, mesh_port, solar, region));
    thread::spawn(move || tcp_relay(mesh_state2, mesh_port));

    // Cleanup + auto-save thread
    let cleanup_state = state.clone();
    thread::spawn(move || {
        let mut tick = 0;
        loop {
            thread::sleep(Duration::from_secs(10));
            tick++;
            let mut mesh = cleanup_state.mesh.lock().unwrap();
            mesh.cleanup_stale();
            let count = mesh.count();
            let mut shield = cleanup_state.shield.lock().unwrap();
            shield.cleanup();
            let (attacks, blocked, blocked_count, level) = shield.stats();
            println!("📊 Mesh: {} noeuds | {} messages vus | 🛡️ Bouclier X9 Niveau {} | {} attaques | {} IP bannies", count, mesh.seen_messages.len(), level, attacks, blocked_count);
            drop(mesh);
            drop(shield);
            // Auto-save toutes les 30 secondes (3 ticks)
            if tick % 3 == 0 {
                let chain = cleanup_state.chain.lock().unwrap();
                chain.save_to_file();
                let wallets = cleanup_state.wallets.lock().unwrap();
                wallets.save();
                let users = cleanup_state.users.lock().unwrap();
                users.save();
                println!("💾 Sauvegarde automatique — {} blocs, {} utilisateurs", chain.blocks.len(), users.count());
            }
        }
    });

    let web_state = web::Data::new(state.clone());

    println!("\n🌐 Serveur web sur http://localhost:8080");
    println!("💾 Sauvegarde automatique active — toutes les 30 secondes");
    println!("📁 Données dans ~/afririch/ (chemin absolu)");
    println!("📡 Mesh relay sur port {}", mesh_port);
    println!("👛 Wallet sur http://localhost:8080/wallet");
    println!("🆕 Inscription sur http://localhost:8080/register");
    println!("📈 Dashboard sur http://localhost:8080/dashboard");
    println!("📖 Annuaire sur http://localhost:8080/annuaire");
    println!("🛡️ Bouclier sur http://localhost:8080/bouclier");
    println!("🛸 AI Satellite X999 sur http://localhost:8080/satellite");
    println!("🛸🛸🛸 Essaim X999 sur http://localhost:8080/swarm");
    println!("🎖️ Commandement X999 sur http://localhost:8080/commandement");
    println!("🛡️ Souverainete des Donnees sur http://localhost:8080/interception");
    println!("💰 AES Wari sur http://localhost:8080/aes");

    HttpServer::new(move || {
        let state = web_state.clone();
        App::new()
            .app_data(state)
            .route("/", web::get().to(|s: web::Data<Arc<AppState>>, req: actix_web::HttpRequest| async move {
                let ip = req.connection_info().peer_addr().unwrap_or("unknown").to_string();
                let path = req.path().to_string();
                let allowed = s.shield.lock().unwrap().check_request(&ip, &path);
                if !allowed {
                    return HttpResponse::Forbidden().body("🛡️ Bouclier X9 — Accès refusé. IP bannie.");
                }
                let chain = s.chain.lock().unwrap();
                let users = s.users.lock().unwrap();
                let mesh = s.mesh.lock().unwrap();
                let shield = s.shield.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_home(&chain, &users, &mesh, &shield))
            }))
            .route("/mesh", web::get().to(|s: web::Data<Arc<AppState>>| async move {
                let mesh = s.mesh.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_mesh(&mesh))
            }))
            .route("/annuaire", web::get().to(|s: web::Data<Arc<AppState>>| async move {
                let mesh = s.mesh.lock().unwrap();
                let users = s.users.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_annuaire(&mesh, &users))
            }))
            .route("/bouclier", web::get().to(|s: web::Data<Arc<AppState>>, req: actix_web::HttpRequest| async move {
                let ip = req.connection_info().peer_addr().unwrap_or("unknown").to_string();
                let path = req.path().to_string();
                let allowed = s.shield.lock().unwrap().check_request(&ip, &path);
                if !allowed {
                    return HttpResponse::Forbidden().body("🛡️ Bouclier X9 — Accès refusé.");
                }
                let shield = s.shield.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_bouclier(&shield))
            }))
            .route("/satellite", web::get().to(|s: web::Data<Arc<AppState>>| async move {
                let mesh = s.mesh.lock().unwrap();
                let users = s.users.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_satellite(&mesh, &users))
            }))
            .route("/aes", web::get().to(|s: web::Data<Arc<AppState>>| async move {
                let users = s.users.lock().unwrap();
                let chain = s.chain.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_aes_wari(&users, &chain))
            }))
            .route("/swarm", web::get().to(|s: web::Data<Arc<AppState>>| async move {
                let mesh = s.mesh.lock().unwrap();
                let users = s.users.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_drone_swarm(&mesh, &users))
            }))
            .route("/commandement", web::get().to(|s: web::Data<Arc<AppState>>| async move {
                let mesh = s.mesh.lock().unwrap();
                let users = s.users.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_command_center(&mesh, &users))
            }))
            .route("/interception", web::get().to(|s: web::Data<Arc<AppState>>| async move {
                let mesh = s.mesh.lock().unwrap();
                let users = s.users.lock().unwrap();
                let chain = s.chain.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_interception(&mesh, &users, &chain))
            }))
            .route("/blocks", web::get().to(|s: web::Data<Arc<AppState>>, req: actix_web::HttpRequest| async move {
                if req.cookie("afri_admin").map(|c| c.value().to_string()) != Some("1".to_string()) {
                    return HttpResponse::Found().append_header(("Location", "/admin")).finish();
                }
                let chain = s.chain.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_blocks(&chain))
            }))
            .route("/balances", web::get().to(|s: web::Data<Arc<AppState>>, req: actix_web::HttpRequest| async move {
                if req.cookie("afri_admin").map(|c| c.value().to_string()) != Some("1".to_string()) {
                    return HttpResponse::Found().append_header(("Location", "/admin")).finish();
                }
                let chain = s.chain.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_balances(&chain))
            }))
            .route("/wallet", web::get().to(|s: web::Data<Arc<AppState>>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let chain = s.chain.lock().unwrap();
                let new_addr = q.get("new").map(|s| s.as_str());
                let new_priv = q.get("priv").map(|s| s.as_str());
                let check_addr = q.get("addr").map(|s| s.as_str());
                let msg = q.get("msg").map(|s| s.as_str());
                HttpResponse::Ok().content_type("text/html").body(html_wallet(new_addr, new_priv, check_addr, msg, &chain))
            }))
            .route("/wallet/new", web::get().to(|s: web::Data<Arc<AppState>>| async move {
                let mut wallets = s.wallets.lock().unwrap();
                let (addr, priv_key) = wallets.create_wallet();
                println!("🆕 Wallet créé : {}", addr);
                HttpResponse::Found()
                    .append_header(("Location", format!("/wallet?new={}&priv={}", addr, priv_key)))
                    .finish()
            }))
            .route("/wallet/balance", web::get().to(|_s: web::Data<Arc<AppState>>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let addr = q.get("addr").cloned().unwrap_or_default();
                HttpResponse::Found()
                    .append_header(("Location", format!("/wallet?addr={}", addr)))
                    .finish()
            }))
            .route("/wallet/send", web::post().to(|s: web::Data<Arc<AppState>>, form: web::Form<SendForm>| async move {
                let wallets = s.wallets.lock().unwrap();
                let users = s.users.lock().unwrap();
                let to_addr = match users.resolve_recipient(&form.to) {
                    Some(addr) => addr,
                    None => return HttpResponse::Found()
                        .append_header(("Location", "/wallet?msg=⚠️ Destinataire introuvable (numéro, adresse ou nom)"))
                        .finish(),
                };
                let mut chain = s.chain.lock().unwrap();
                let mut tx = Transaction::new(&form.from, &to_addr, form.amount, &form.memo);
                if let Some(sk) = wallets.get_signing_key(&form.from) {
                    tx.sign(&sk);
                    println!("🔐 Transaction signée Ed25519 : {} → {} ({} AFR)", form.from, to_addr, form.amount);
                    let tx_json = serde_json::to_string(&tx).unwrap_or_default();
                    chain.add_transaction(tx);
                    chain.save_to_file();
                    drop(chain);
                    drop(wallets);
                    drop(users);
                    broadcast_mesh(&s, "tx", &tx_json);
                    HttpResponse::Found()
                        .append_header(("Location", "/wallet?msg=✅ Envoyé ! (signé Ed25519 🔐)"))
                        .finish()
                } else {
                    HttpResponse::Found()
                        .append_header(("Location", "/wallet?msg=⚠️ Adresse non trouvée dans ce wallet"))
                        .finish()
                }
            }))
            .route("/wallet/mine", web::post().to(|s: web::Data<Arc<AppState>>, form: web::Form<MineForm>| async move {
                let mut chain = s.chain.lock().unwrap();
                chain.mine_pending(&form.miner);
                println!("⛏️ Bloc miné pour {}", form.miner);
                let block_json = serde_json::to_string(chain.blocks.last().unwrap()).unwrap_or_default();
                drop(chain);
                broadcast_mesh(&s, "block", &block_json);
                HttpResponse::Found()
                    .append_header(("Location", "/wallet?msg=⛏️ Bloc miné ! +100 AFR pour le mineur"))
                    .finish()
            }))
            // ===== REGISTER =====
            .route("/register", web::get().to(|_s: web::Data<Arc<AppState>>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let msg = q.get("err").map(|s| s.as_str());
                HttpResponse::Ok().content_type("text/html").body(html_register(msg))
            }))
            .route("/register", web::post().to(|s: web::Data<Arc<AppState>>, form: web::Form<RegisterForm>| async move {
                let mut wallets = s.wallets.lock().unwrap();
                let mut users = s.users.lock().unwrap();
                match users.register(&form.username, &form.password, &form.country, &mut wallets) {
                    Ok(user) => {
                        // Broadcast directory entry on mesh
                        let mesh = s.mesh.lock().unwrap();
                        let entry = DirectoryEntry {
                            phone: user.phone.clone(),
                            address: user.address.clone(),
                            username: user.username.clone(),
                            country: user.country.clone(),
                            country_code: user.country_code.clone(),
                            node_id: mesh.my_id.clone(),
                            timestamp: Utc::now().timestamp(),
                        };
                        let entry_json = serde_json::to_string(&entry).unwrap_or_default();
                        drop(mesh);
                        broadcast_mesh(&s, "directory", &entry_json);
                        println!("📡 Annuaire diffusé : {} → {}", user.phone, user.country);
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
            .route("/login", web::get().to(|_s: web::Data<Arc<AppState>>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let msg = q.get("err").map(|s| s.as_str());
                HttpResponse::Ok().content_type("text/html").body(html_login(msg))
            }))
            .route("/login", web::post().to(|s: web::Data<Arc<AppState>>, form: web::Form<LoginForm>, req: actix_web::HttpRequest| async move {
                let ip = req.connection_info().peer_addr().unwrap_or("unknown").to_string();
                let users = s.users.lock().unwrap();
                match users.login(&form.username, &form.password) {
                    Some(user) => {
                        HttpResponse::Found()
                            .append_header(("Location", format!("/account?user={}", user.username)))
                            .finish()
                    }
                    None => {
                        drop(users);
                        s.shield.lock().unwrap().record_failed_login(&ip);
                        HttpResponse::Found()
                            .append_header(("Location", "/login?err=Nom d'utilisateur ou mot de passe incorrect"))
                            .finish()
                    }
                }
            }))
            // ===== ACCOUNT =====
            .route("/account", web::get().to(|s: web::Data<Arc<AppState>>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let username = q.get("user").cloned().unwrap_or_default();
                let users = s.users.lock().unwrap();
                let chain = s.chain.lock().unwrap();
                let msg = q.get("msg").map(|s| s.as_str());
                match users.users.iter().find(|u| u.username == username) {
                    Some(user) => HttpResponse::Ok().content_type("text/html").body(html_account(user, &chain, msg)),
                    None => HttpResponse::Found().append_header(("Location", "/login")).finish(),
                }
            }))
            .route("/account/send", web::post().to(|s: web::Data<Arc<AppState>>, form: web::Form<SendForm>| async move {
                let wallets = s.wallets.lock().unwrap();
                let users = s.users.lock().unwrap();
                let to_addr = match users.resolve_recipient(&form.to) {
                    Some(addr) => addr,
                    None => {
                        return HttpResponse::Found()
                            .append_header(("Location", format!("/account?user={}&msg=⚠️ Destinataire introuvable", users.users.iter().find(|u| u.address == form.from).map(|u| u.username.clone()).unwrap_or_default())))
                            .finish();
                    }
                };
                let user = users.users.iter().find(|u| u.address == form.from);
                let username = user.map(|u| u.username.clone()).unwrap_or_default();
                let mut chain = s.chain.lock().unwrap();
                let mut tx = Transaction::new(&form.from, &to_addr, form.amount, &form.memo);
                if let Some(sk) = wallets.get_signing_key(&form.from) {
                    tx.sign(&sk);
                    let tx_json = serde_json::to_string(&tx).unwrap_or_default();
                    chain.add_transaction(tx);
                    chain.save_to_file();
                    drop(chain);
                    drop(wallets);
                    drop(users);
                    broadcast_mesh(&s, "tx", &tx_json);
                    HttpResponse::Found()
                        .append_header(("Location", format!("/account?user={}&msg=✅ Envoyé à {} ! {} AFR signés", username, form.to, form.amount)))
                        .finish()
                } else {
                    HttpResponse::Found()
                        .append_header(("Location", format!("/account?user={}&msg=⚠️ Clé privée introuvable", username)))
                        .finish()
                }
            }))
            .route("/account/mine", web::post().to(|s: web::Data<Arc<AppState>>, form: web::Form<MineForm>| async move {
                let mut chain = s.chain.lock().unwrap();
                let users = s.users.lock().unwrap();
                let user = users.users.iter().find(|u| u.address == form.miner);
                let username = user.map(|u| u.username.clone()).unwrap_or_default();
                chain.mine_pending(&form.miner);
                let block_json = serde_json::to_string(chain.blocks.last().unwrap()).unwrap_or_default();
                drop(chain);
                drop(users);
                broadcast_mesh(&s, "block", &block_json);
                HttpResponse::Found()
                    .append_header(("Location", format!("/account?user={}&msg=⛏️ Miné ! +100 AFR", username)))
                    .finish()
            }))
            // ===== ADMIN =====
            .route("/admin", web::get().to(|_s: web::Data<Arc<AppState>>, q: web::Query<std::collections::HashMap<String, String>>| async move {
                let err = q.get("err").map(|s| s.as_str());
                HttpResponse::Ok().content_type("text/html").body(html_admin_login(err))
            }))
            .route("/admin", web::post().to(|_s: web::Data<Arc<AppState>>, form: web::Form<AdminForm>| async move {
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
            .route("/dashboard", web::get().to(|s: web::Data<Arc<AppState>>, req: actix_web::HttpRequest| async move {
                if req.cookie("afri_admin").map(|c| c.value().to_string()) != Some("1".to_string()) {
                    return HttpResponse::Found().append_header(("Location", "/admin")).finish();
                }
                let chain = s.chain.lock().unwrap();
                let users = s.users.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_dashboard(&chain, &users))
            }))
            // ===== API =====
            .route("/api/blocks", web::get().to(|s: web::Data<Arc<AppState>>| async move {
                let chain = s.chain.lock().unwrap();
                HttpResponse::Ok().json(&chain.blocks)
            }))
            .route("/api/status", web::get().to(|s: web::Data<Arc<AppState>>| async move {
                let chain = s.chain.lock().unwrap();
                let users = s.users.lock().unwrap();
                let mesh = s.mesh.lock().unwrap();
                let shield = s.shield.lock().unwrap();
                let (attacks, blocked, blocked_count, level) = shield.stats();
                let json = format!(r#"{{"name":"AfriChain","blocks":{},"transactions":{},"users":{},"valid":{},"token":"AFR","version":"0.12","crypto":"Ed25519","supply":{},"mesh_nodes":{},"mesh_id":"{}","mesh_region":"{}","countries":54,"directory":{},"shield_active":{},"shield_level":{},"shield_attacks":{},"shield_blocked_ips":{}}}"#,
                    chain.blocks.len(), chain.total_transactions(), users.count(), chain.is_valid(), chain.total_supply(), mesh.count(), mesh.my_id, mesh.region, mesh.directory_count() + users.count(), shield.active, level, attacks, blocked_count);
                HttpResponse::Ok().content_type("application/json").body(json)
            }))
            .route("/api/directory", web::get().to(|s: web::Data<Arc<AppState>>| async move {
                let mesh = s.mesh.lock().unwrap();
                let users = s.users.lock().unwrap();
                let mut entries: Vec<DirectoryEntry> = Vec::new();
                // Local users
                for user in &users.users {
                    entries.push(DirectoryEntry {
                        phone: user.phone.clone(),
                        address: user.address.clone(),
                        username: user.username.clone(),
                        country: user.country.clone(),
                        country_code: user.country_code.clone(),
                        node_id: mesh.my_id.clone(),
                        timestamp: user.created_at,
                    });
                }
                // Mesh directory (dedup by phone)
                for entry in mesh.directory.values() {
                    if !entries.iter().any(|e| e.phone == entry.phone) {
                        entries.push(entry.clone());
                    }
                }
                HttpResponse::Ok().json(&entries)
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
                let sw = "const C='afri-v0.12';self.addEventListener('install',e=>{e.waitUntil(caches.open(C).then(c=>c.addAll(['/wallet','/manifest.json','/icon.svg'])))});self.addEventListener('fetch',e=>{e.respondWith(caches.match(e.request).then(r=>r||fetch(e.request)))});";
                HttpResponse::Ok().content_type("application/javascript").body(sw)
            }))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
