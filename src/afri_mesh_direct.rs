// AfriMesh Direct — Protocole mesh direct pour telephones africains
// 100% from scratch, std uniquement
// Connecte les telephones SANS operateurs (Orange, MTN, Moov)
// Fonctionne sur Bluetooth, WiFi Direct, ou TCP

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

use crate::afri_json::{JsonValue, to_string, from_str};
use crate::afri_hex::{encode as hex_encode, decode as hex_decode};
use crate::afri_time::now_timestamp;

// Types de messages mesh direct
pub const MSG_TX: &str = "tx";           // Transaction
pub const MSG_BLOCK: &str = "block";      // Bloc
pub const MSG_CHAT: &str = "chat";        // Message chat
pub const MSG_DIR: &str = "directory";    // Entree annuaire
pub const MSG_PING: &str = "ping";        // Decouverte
pub const MSG_PONG: &str = "pong";        // Reponse decouverte
pub const MSG_SYNC: &str = "sync";        // Sync blockchain (light)
pub const MSG_STORED: &str = "stored";     // Message stocke (store-and-forward)

// Nombre maximum de sauts (hops) avant qu'un message meure
pub const MAX_HOPS: u8 = 50;

// Nombre maximum de messages stockes (store-and-forward)
pub const MAX_STORED: usize = 10000;

// Taille max d'un message (bytes)
pub const MAX_MSG_SIZE: usize = 65536;

/// Message direct — format pour la communication telephone-à-telephone
#[derive(Clone)]
pub struct DirectMessage {
    pub msg_type: String,      // tx, block, chat, directory, ping, pong, sync, stored
    pub msg_id: String,        // identifiant unique (hash du contenu)
    pub sender: String,        // node_id de l'envoyeur
    pub recipient: String,     // node_id du destinataire ("*" = broadcast)
    pub payload: String,       // contenu (JSON, texte, etc.)
    pub hops: u8,               // nombre de sauts effectués
    pub timestamp: i64,        // heure d'envoi
    pub signature: String,     // signature Ed25519 de l'envoyeur
}

impl DirectMessage {
    /// Crée un nouveau message direct
    pub fn new(msg_type: &str, sender: &str, recipient: &str, payload: &str) -> Self {
        let mut rng_data = format!("{}{}{}{}", msg_type, sender, payload, now_timestamp());
        let hash = crate::afri_hash::afrihash_256(rng_data.as_bytes());
        DirectMessage {
            msg_type: msg_type.to_string(),
            msg_id: hex_encode(&hash),
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            payload: payload.to_string(),
            hops: 0,
            timestamp: now_timestamp(),
            signature: String::new(),
        }
    }

    /// Signe le message avec Ed25519
    pub fn sign(&mut self, sk: &crate::afri_ed25519::AfriSecretKey) {
        let data = self.sign_data();
        let sig = sk.sign(&data);
        self.signature = hex_encode(&sig.to_bytes());
    }

    /// Vérifie la signature Ed25519
    pub fn verify(&self) -> bool {
        if self.signature.is_empty() { return true; }
        let sig_bytes = match hex_decode(&self.signature) {
            Some(b) if b.len() == 64 => b,
            _ => return false,
        };
        let sig_arr: [u8; 64] = sig_bytes.try_into().unwrap();
        let signature = crate::afri_ed25519::AfriSignature::from_bytes(&sig_arr);
        // Pour la verification, on a besoin de la cle publique de l'envoyeur
        // Le sender est un node_id qui contient la cle publique
        let pub_hex = match self.sender.strip_prefix("AFR-") {
            Some(h) => h,
            None => return true, // pas de prefix, on accepte
        };
        let pub_bytes = match hex_decode(pub_hex) {
            Some(b) if b.len() == 32 => b,
            _ => return true,
        };
        let pub_arr: [u8; 32] = pub_bytes.try_into().unwrap();
        let vk = match crate::afri_ed25519::AfriPublicKey::from_bytes(&pub_arr) {
            Some(k) => k,
            None => return false,
        };
        vk.verify(&self.sign_data(), &signature)
    }

    fn sign_data(&self) -> Vec<u8> {
        let mut arr = Vec::new();
        arr.push(JsonValue::Str(self.msg_type.clone()));
        arr.push(JsonValue::Str(self.msg_id.clone()));
        arr.push(JsonValue::Str(self.sender.clone()));
        arr.push(JsonValue::Str(self.recipient.clone()));
        arr.push(JsonValue::Str(self.payload.clone()));
        arr.push(JsonValue::UInt(self.hops as u64));
        arr.push(JsonValue::Int(self.timestamp));
        to_string(&JsonValue::Array(arr)).into_bytes()
    }

    /// Sérialise en JSON pour transmission
    pub fn to_json(&self) -> JsonValue {
        let mut map = HashMap::new();
        map.insert("type".to_string(), JsonValue::Str(self.msg_type.clone()));
        map.insert("id".to_string(), JsonValue::Str(self.msg_id.clone()));
        map.insert("sender".to_string(), JsonValue::Str(self.sender.clone()));
        map.insert("recipient".to_string(), JsonValue::Str(self.recipient.clone()));
        map.insert("payload".to_string(), JsonValue::Str(self.payload.clone()));
        map.insert("hops".to_string(), JsonValue::UInt(self.hops as u64));
        map.insert("timestamp".to_string(), JsonValue::Int(self.timestamp));
        map.insert("signature".to_string(), JsonValue::Str(self.signature.clone()));
        JsonValue::Object(map)
    }

    /// Désérialise depuis JSON
    pub fn from_json(j: &JsonValue) -> Option<Self> {
        let obj = match j { JsonValue::Object(o) => o, _ => return None };
        Some(DirectMessage {
            msg_type: obj.get("type")?.as_str()?.to_string(),
            msg_id: obj.get("id")?.as_str()?.to_string(),
            sender: obj.get("sender")?.as_str()?.to_string(),
            recipient: obj.get("recipient")?.as_str()?.to_string(),
            payload: obj.get("payload")?.as_str()?.to_string(),
            hops: obj.get("hops")?.as_u64()? as u8,
            timestamp: obj.get("timestamp")?.as_i64()?,
            signature: obj.get("signature").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        })
    }

    /// Sérialise en bytes pour transmission réseau
    pub fn to_bytes(&self) -> Vec<u8> {
        to_string(&self.to_json()).into_bytes()
    }

    /// Désérialise depuis bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        let s = std::str::from_utf8(data).ok()?;
        let j = from_str(s).ok()?;
        Self::from_json(&j)
    }

    /// Incrémente les sauts (pour le relay)
    pub fn hop(&mut self) -> bool {
        if self.hops >= MAX_HOPS {
            return false; // message mort
        }
        self.hops += 1;
        true
    }

    /// Est-ce que ce message est pour moi?
    pub fn is_for(&self, my_id: &str) -> bool {
        self.recipient == "*" || self.recipient == my_id
    }

    /// Est-ce un broadcast?
    pub fn is_broadcast(&self) -> bool {
        self.recipient == "*"
    }
}

/// Entree d'annuaire direct — un telephone africain sur le mesh
#[derive(Clone)]
pub struct DirectNode {
    pub node_id: String,
    pub phone: String,
    pub username: String,
    pub country: String,
    pub country_code: String,
    pub address: String,       // adresse blockchain (Afri...)
    pub last_seen: i64,         // derniere fois vu
    pub hop_distance: u8,      // distance en sauts
}

impl DirectNode {
    pub fn to_json(&self) -> JsonValue {
        let mut map = HashMap::new();
        map.insert("node_id".to_string(), JsonValue::Str(self.node_id.clone()));
        map.insert("phone".to_string(), JsonValue::Str(self.phone.clone()));
        map.insert("username".to_string(), JsonValue::Str(self.username.clone()));
        map.insert("country".to_string(), JsonValue::Str(self.country.clone()));
        map.insert("country_code".to_string(), JsonValue::Str(self.country_code.clone()));
        map.insert("address".to_string(), JsonValue::Str(self.address.clone()));
        map.insert("last_seen".to_string(), JsonValue::Int(self.last_seen));
        map.insert("hop_distance".to_string(), JsonValue::UInt(self.hop_distance as u64));
        JsonValue::Object(map)
    }

    pub fn from_json(j: &JsonValue) -> Option<Self> {
        let obj = match j { JsonValue::Object(o) => o, _ => return None };
        Some(DirectNode {
            node_id: obj.get("node_id")?.as_str()?.to_string(),
            phone: obj.get("phone")?.as_str()?.to_string(),
            username: obj.get("username")?.as_str()?.to_string(),
            country: obj.get("country")?.as_str()?.to_string(),
            country_code: obj.get("country_code")?.as_str()?.to_string(),
            address: obj.get("address")?.as_str()?.to_string(),
            last_seen: obj.get("last_seen")?.as_i64()?,
            hop_distance: obj.get("hop_distance").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
        })
    }

    /// Est-ce que ce nœud est actif? (vu dans les 5 dernieres minutes)
    pub fn is_active(&self) -> bool {
        now_timestamp() - self.last_seen < 300
    }
}

/// Message stocké (store-and-forward) — pour livraison quand le destinataire se connecte
pub struct StoredMessage {
    pub message: DirectMessage,
    pub stored_at: i64,
    pub delivery_attempts: u32,
}

/// En-tête de bloc léger (light sync) — pour téléphones avec peu de mémoire
pub struct LightBlock {
    pub index: u64,
    pub hash: String,
    pub prev_hash: String,
    pub timestamp: i64,
    pub tx_count: usize,
    pub miner: String,
    pub country: String,
}

impl LightBlock {
    pub fn to_json(&self) -> JsonValue {
        let mut map = HashMap::new();
        map.insert("index".to_string(), JsonValue::UInt(self.index));
        map.insert("hash".to_string(), JsonValue::Str(self.hash.clone()));
        map.insert("prev_hash".to_string(), JsonValue::Str(self.prev_hash.clone()));
        map.insert("timestamp".to_string(), JsonValue::Int(self.timestamp));
        map.insert("tx_count".to_string(), JsonValue::UInt(self.tx_count as u64));
        map.insert("miner".to_string(), JsonValue::Str(self.miner.clone()));
        map.insert("country".to_string(), JsonValue::Str(self.country.clone()));
        JsonValue::Object(map)
    }
}

/// AfriMesh Direct — le cœur du mesh panafricain
pub struct AfriMeshDirect {
    pub my_id: String,
    pub my_phone: String,
    pub my_username: String,
    pub my_country: String,
    pub my_country_code: String,
    pub my_address: String,
    
    // Nœuds découverts sur le mesh
    pub nodes: HashMap<String, DirectNode>,
    
    // Messages déjà vus (anti-boucle)
    pub seen: HashMap<String, i64>,  // msg_id -> timestamp
    
    // Messages stockés (store-and-forward)
    pub stored: Vec<StoredMessage>,
    
    // Port d'écoute directe
    pub listen_port: u16,
    
    // Statistiques
    pub messages_relayed: u64,
    pub messages_delivered: u64,
    pub messages_stored: u64,
    pub nodes_discovered: u64,
}

impl AfriMeshDirect {
    pub fn new(
        my_id: &str,
        phone: &str,
        username: &str,
        country: &str,
        country_code: &str,
        address: &str,
        listen_port: u16,
    ) -> Self {
        AfriMeshDirect {
            my_id: my_id.to_string(),
            my_phone: phone.to_string(),
            my_username: username.to_string(),
            my_country: country.to_string(),
            my_country_code: country_code.to_string(),
            my_address: address.to_string(),
            nodes: HashMap::new(),
            seen: HashMap::new(),
            stored: Vec::new(),
            listen_port,
            messages_relayed: 0,
            messages_delivered: 0,
            messages_stored: 0,
            nodes_discovered: 0,
        }
    }

    /// Ajoute un nœud découvert
    pub fn add_node(&mut self, node: DirectNode) {
        if node.node_id == self.my_id { return; }
        let is_new = !self.nodes.contains_key(&node.node_id);
        self.nodes.insert(node.node_id.clone(), node);
        if is_new {
            self.nodes_discovered += 1;
        }
    }

    /// Nettoie les nœuds inactifs (plus vus depuis 5 min)
    pub fn cleanup(&mut self) {
        let now = now_timestamp();
        self.nodes.retain(|_, n| now - n.last_seen < 300);
        self.seen.retain(|_, t| now - *t < 3600); // garde 1h d'historique
    }

    /// Compte les nœuds actifs
    pub fn count(&self) -> usize {
        self.nodes.values().filter(|n| n.is_active()).count() + 1 // +1 pour moi
    }

    /// Compte tous les nœuds (actifs + inactifs récents)
    pub fn count_all(&self) -> usize {
        self.nodes.len() + 1
    }

    /// Vérifie si on a déjà vu ce message (anti-boucle)
    pub fn already_seen(&self, msg_id: &str) -> bool {
        self.seen.contains_key(msg_id)
    }

    /// Marque un message comme vu
    pub fn mark_seen(&mut self, msg_id: &str) {
        self.seen.insert(msg_id.to_string(), now_timestamp());
        // Nettoie si trop plein
        if self.seen.len() > 50000 {
            let now = now_timestamp();
            self.seen.retain(|_, t| now - *t < 3600);
        }
    }

    /// Traite un message reçu du mesh
    /// Retourne true si on doit le relayer, false sinon
    pub fn receive(&mut self, msg: &DirectMessage) -> bool {
        // Anti-boucle: déjà vu?
        if self.already_seen(&msg.msg_id) {
            return false;
        }
        self.mark_seen(&msg.msg_id);

        // Le message est pour moi?
        if msg.is_for(&self.my_id) {
            self.messages_delivered += 1;
            return false; // pas besoin de relayer
        }

        // Message broadcast ou pour quelqu'un d'autre — relaye
        // Mais d'abord, stocke si le destinataire n'est pas connecté
        if !msg.is_broadcast() && !self.nodes.contains_key(&msg.recipient) {
            // Destinataire pas connecté — stocke pour livraison ultérieure
            self.store_message(msg.clone());
        }

        self.messages_relayed += 1;
        true
    }

    /// Stocke un message pour livraison ultérieure (store-and-forward)
    pub fn store_message(&mut self, msg: DirectMessage) {
        if self.stored.len() >= MAX_STORED {
            self.stored.remove(0); // FIFO — enlève le plus ancien
        }
        self.stored.push(StoredMessage {
            message: msg,
            stored_at: now_timestamp(),
            delivery_attempts: 0,
        });
        self.messages_stored += 1;
    }

    /// Vérifie les messages stockés pour un nœud qui vient de se connecter
    pub fn check_stored_for(&mut self, node_id: &str) -> Vec<DirectMessage> {
        let mut delivered = Vec::new();
        self.stored.retain(|s| {
            if s.message.recipient == node_id {
                delivered.push(s.message.clone());
                false // enlève de la liste
            } else {
                true
            }
        });
        self.messages_delivered += delivered.len() as u64;
        delivered
    }

    /// Crée un message ping pour découverte
    pub fn create_ping(&self) -> DirectMessage {
        let payload = format!("{}|{}|{}|{}|{}", 
            self.my_phone, self.my_username, 
            self.my_country, self.my_country_code, self.my_address);
        DirectMessage::new(MSG_PING, &self.my_id, "*", &payload)
    }

    /// Crée un message pong pour répondre à un ping
    pub fn create_pong(&self, sender: &str) -> DirectMessage {
        let payload = format!("{}|{}|{}|{}|{}", 
            self.my_phone, self.my_username, 
            self.my_country, self.my_country_code, self.my_address);
        DirectMessage::new(MSG_PONG, &self.my_id, sender, &payload)
    }

    /// Parse un ping/pong pour extraire les infos du nœud
    pub fn parse_node_info(payload: &str, sender: &str, hop_distance: u8) -> Option<DirectNode> {
        let parts: Vec<&str> = payload.split('|').collect();
        if parts.len() < 5 { return None; }
        Some(DirectNode {
            node_id: sender.to_string(),
            phone: parts[0].to_string(),
            username: parts[1].to_string(),
            country: parts[2].to_string(),
            country_code: parts[3].to_string(),
            address: parts[4].to_string(),
            last_seen: now_timestamp(),
            hop_distance,
        })
    }

    /// Liste les nœuds par pays
    pub fn nodes_by_country(&self) -> HashMap<String, Vec<&DirectNode>> {
        let mut by_country: HashMap<String, Vec<&DirectNode>> = HashMap::new();
        for node in self.nodes.values() {
            by_country.entry(node.country.clone())
                .or_insert_with(Vec::new)
                .push(node);
        }
        by_country
    }

    /// Statistiques du mesh
    pub fn stats(&self) -> (usize, u64, u64, u64, u64) {
        (self.count(), self.messages_relayed, self.messages_delivered, 
         self.messages_stored, self.nodes_discovered)
    }

    /// Sérialise l'état pour persistance
    pub fn to_json(&self) -> JsonValue {
        let mut map = HashMap::new();
        map.insert("my_id".to_string(), JsonValue::Str(self.my_id.clone()));
        map.insert("listen_port".to_string(), JsonValue::UInt(self.listen_port as u64));
        map.insert("messages_relayed".to_string(), JsonValue::UInt(self.messages_relayed));
        map.insert("messages_delivered".to_string(), JsonValue::UInt(self.messages_delivered));
        map.insert("messages_stored".to_string(), JsonValue::UInt(self.messages_stored));
        map.insert("nodes_discovered".to_string(), JsonValue::UInt(self.nodes_discovered));
        let nodes_arr: Vec<JsonValue> = self.nodes.values()
            .map(|n| n.to_json())
            .collect();
        map.insert("nodes".to_string(), JsonValue::Array(nodes_arr));
        JsonValue::Object(map)
    }
}

/// Démarre l'écoute TCP pour connexions directes
pub fn listen_direct(mesh_port: u16, my_info: String) {
    thread::spawn(move || {
        let addr = format!("0.0.0.0:{}", mesh_port);
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Mesh Direct: impossible d'écouter sur {}: {}", addr, e);
                return;
            }
        };
        println!("📡 AfriMesh Direct en écoute sur le port {}", mesh_port);
        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    let info = my_info.clone();
                    thread::spawn(move || {
                        handle_direct_connection(&mut s, &info);
                    });
                }
                Err(_) => continue,
            }
        }
    });
}

fn handle_direct_connection(stream: &mut TcpStream, my_info: &str) {
    let mut buf = [0u8; MAX_MSG_SIZE];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    
    // Tente de parser le message
    if let Some(msg) = DirectMessage::from_bytes(&buf[..n]) {
        // Répond avec nos infos si c'est un ping
        if msg.msg_type == MSG_PING {
            let pong = DirectMessage::new(MSG_PONG, my_info, &msg.sender, my_info);
            let _ = stream.write_all(&pong.to_bytes());
        }
    }
}

/// Découverte active — scanne les ports pour trouver d'autres nœuds
pub fn discover_nodes(mesh_port: u16, my_info: String, known_ips: Vec<String>) {
    thread::spawn(move || {
        loop {
            for ip in &known_ips {
                let addr = format!("{}:{}", ip, mesh_port);
                if let Ok(mut stream) = TcpStream::connect_timeout(
                    &addr.parse::<SocketAddr>().unwrap_or_else(|_| {
                        format!("0.0.0.0:80").parse().unwrap()
                    }),
                    Duration::from_secs(2)
                ) {
                    let ping = DirectMessage::new(MSG_PING, &my_info, "*", &my_info);
                    let _ = stream.write_all(&ping.to_bytes());
                }
            }
            thread::sleep(Duration::from_secs(30)); // scanne toutes les 30s
        }
    });
}
