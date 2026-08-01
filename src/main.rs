use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use chrono::Utc;
use std::sync::{Arc, Mutex};

use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Verifier, Signature};
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
        std::fs::write("blockchain.json", data).ok();
    }

    fn load_from_file() -> Option<Self> {
        match std::fs::read_to_string("blockchain.json") {
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
        match std::fs::read_to_string("wallets.json") {
            Ok(data) => serde_json::from_str(&data).unwrap_or(WalletStore { wallets: HashMap::new() }),
            Err(_) => WalletStore { wallets: HashMap::new() },
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
        match std::fs::read_to_string("users.json") {
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
        std::fs::write("users.json", data).ok();
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
        self.nodes.len()
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
        if let Ok(bytes) = announce.to_bytes().as_slice().try_into().map(|_: Vec<u8>| announce.to_bytes()) {
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
                                println!("💸 Transaction reçue via mesh");
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
fn html_home(chain: &Blockchain, users: &UserStore, mesh: &NodeRegistry) -> String {
    let mut html = html_head("🦁 AfriChain");
    html.push_str(&format!(r#"<h1>🦁 AfriChain</h1><p style="text-align:center;">La blockchain 100% africaine — 54 pays 💚🦁</p><div class="nav"><a href="/register">🆕 S'inscrire</a> | <a href="/login">🔑 Connexion</a> | <a href="/wallet">👛 Wallet</a> | <a href="/admin">🔐 Admin</a> | <a href="/mesh">📡 Mesh</a> | <a href="/annuaire">📖 Annuaire</a> | <a href="/api/status">🔌 API</a></div><div style="text-align:center;"><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">Blocs</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">Transactions</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">Utilisateurs</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">AFR en circulation</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📡 Noeuds mesh</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📖 Numéros annuaire</div></div></div><div class="card"><div style="display:flex;justify-content:space-between;padding:8px 0;border-bottom:1px solid rgba(212,164,55,0.2);"><span style="color:#a8c5a8;">🪙 Token</span><b>AfriRich (AFR)</b></div><div style="display:flex;justify-content:space-between;padding:8px 0;border-bottom:1px solid rgba(212,164,55,0.2);"><span style="color:#a8c5a8;">🌍 Pays</span><b>54 pays africains</b></div><div style="display:flex;justify-content:space-between;padding:8px 0;border-bottom:1px solid rgba(212,164,55,0.2);"><span style="color:#a8c5a8;">🛡️ Statut</span><b>Souveraine 💚</b></div><div style="display:flex;justify-content:space-between;padding:8px 0;"><span style="color:#a8c5a8;">🔐 Crypto</span><b>Ed25519</b></div></div><footer style="text-align:center;margin-top:40px;color:#a8c5a8;">🦁 Codée from scratch par Machine-senpai — v0.10 Annuaire Mesh</footer>"#,
        chain.blocks.len(),
        chain.total_transactions(),
        users.count(),
        chain.total_supply(),
        mesh.count(),
        mesh.directory_count() + users.count(),
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
    println!("🦁 AfriChain v0.10 — Annuaire Mesh");
    println!("💚 L'Afrique n'a pas besoin de permission");
    println!("🌍 54 pays africains intégrés");
    println!("📖 Annuaire mesh panafricain — tous les numéros sur écoute");
    println!("📡 Node ID: {}", my_node_id);
    println!("🔌 Mesh port: {}", mesh_port);
    println!("☀️  Solaire: {}", if solar { "Oui" } else { "Non" });
    println!("🌍 Région: {}", region);

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

    let state = Arc::new(AppState {
        chain: Mutex::new(chain),
        wallets: Mutex::new(wallets),
        users: Mutex::new(users),
        mesh: Mutex::new(registry),
    });

    // Start mesh threads
    let mesh_state1 = state.clone();
    let mesh_state2 = state.clone();
    let my_id_clone = my_node_id.clone();
    thread::spawn(move || udp_discovery(mesh_state1, my_id_clone, mesh_port, solar, region));
    thread::spawn(move || tcp_relay(mesh_state2, mesh_port));

    // Cleanup thread
    let cleanup_state = state.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10));
            let mut mesh = cleanup_state.mesh.lock().unwrap();
            mesh.cleanup_stale();
            let count = mesh.count();
            println!("📊 Mesh: {} noeuds | {} messages vus", count, mesh.seen_messages.len());
        }
    });

    let web_state = web::Data::new(state.clone());

    println!("\n🌐 Serveur web sur http://localhost:8080");
    println!("📡 Mesh relay sur port {}", mesh_port);
    println!("👛 Wallet sur http://localhost:8080/wallet");
    println!("🆕 Inscription sur http://localhost:8080/register");
    println!("📈 Dashboard sur http://localhost:8080/dashboard");
    println!("📖 Annuaire sur http://localhost:8080/annuaire");

    HttpServer::new(move || {
        let state = web_state.clone();
        App::new()
            .app_data(state)
            .route("/", web::get().to(|s: web::Data<Arc<AppState>>| async move {
                let chain = s.chain.lock().unwrap();
                let users = s.users.lock().unwrap();
                let mesh = s.mesh.lock().unwrap();
                HttpResponse::Ok().content_type("text/html").body(html_home(&chain, &users, &mesh))
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
            .route("/login", web::post().to(|s: web::Data<Arc<AppState>>, form: web::Form<LoginForm>| async move {
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
                let json = format!(r#"{{"name":"AfriChain","blocks":{},"transactions":{},"users":{},"valid":{},"token":"AFR","version":"0.10","crypto":"Ed25519","supply":{},"mesh_nodes":{},"mesh_id":"{}","mesh_region":"{}","countries":54,"directory":{}}}"#,
                    chain.blocks.len(), chain.total_transactions(), users.count(), chain.is_valid(), chain.total_supply(), mesh.count(), mesh.my_id, mesh.region, mesh.directory_count() + users.count());
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
                let sw = "const C='afri-v0.10';self.addEventListener('install',e=>{e.waitUntil(caches.open(C).then(c=>c.addAll(['/wallet','/manifest.json','/icon.svg'])))});self.addEventListener('fetch',e=>{e.respondWith(caches.match(e.request).then(r=>r||fetch(e.request)))});";
                HttpResponse::Ok().content_type("application/javascript").body(sw)
            }))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
