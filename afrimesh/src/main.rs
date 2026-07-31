use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;
use std::net::{UdpSocket, TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{Read, Write};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ===== NODE ID =====
fn generate_node_id() -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("afrimesh-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)).as_bytes());
    let hash = hex::encode(hasher.finalize());
    format!("AFR-{}", &hash[..16])
}

// ===== MESH MESSAGE =====
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeshMessage {
    msg_type: String,    // "discovery", "relay", "block", "tx", "ping", "ack"
    node_id: String,     // who sent this
    payload: String,     // the actual data (JSON string)
    timestamp: i64,
    ttl: u32,            // time to live (hops remaining)
    msg_id: String,     // unique message ID (for dedup)
}

impl MeshMessage {
    fn new(msg_type: &str, node_id: &str, payload: &str, ttl: u32) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}-{}-{}", node_id, msg_type, Utc::now().timestamp_nanos_opt().unwrap_or(0)).as_bytes());
        MeshMessage {
            msg_type: msg_type.to_string(),
            node_id: node_id.to_string(),
            payload: payload.to_string(),
            timestamp: Utc::now().timestamp(),
            ttl,
            msg_id: hex::encode(&hasher.finalize()[..16]),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    fn from_bytes(data: &[u8]) -> Option<Self> {
        serde_json::from_slice(data).ok()
    }
}

// ===== NODE REGISTRY =====
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeInfo {
    node_id: String,
    address: String,
    last_seen: i64,
    solar_powered: bool,
    region: String,
}

struct NodeRegistry {
    nodes: HashMap<String, NodeInfo>,
    seen_messages: HashMap<String, Instant>,
    my_id: String,
    my_port: u16,
    solar: bool,
    region: String,
}

impl NodeRegistry {
    fn new(my_id: String, my_port: u16, solar: bool, region: String) -> Self {
        NodeRegistry {
            nodes: HashMap::new(),
            seen_messages: HashMap::new(),
            my_id,
            my_port,
            solar,
            region,
        }
    }

    fn add_or_update(&mut self, node_id: String, address: String, solar: bool, region: String) {
        let info = NodeInfo {
            node_id: node_id.clone(),
            address,
            last_seen: Utc::now().timestamp(),
            solar_powered: solar,
            region,
        };
        self.nodes.insert(node_id, info);
    }

    fn cleanup_stale(&mut self) {
        let now = Utc::now().timestamp();
        let timeout = 60; // 60 seconds
        self.nodes.retain(|_, info| now - info.last_seen < timeout);
    }

    fn has_seen(&self, msg_id: &str) -> bool {
        self.seen_messages.contains_key(msg_id)
    }

    fn mark_seen(&mut self, msg_id: String) {
        self.seen_messages.insert(msg_id, Instant::now());
        // Cleanup old seen messages (keep 1000 max)
        if self.seen_messages.len() > 1000 {
            let oldest: Vec<String> = self.seen_messages.iter()
                .min_by_key(|(_, t)| *t)
                .map(|(k, _)| vec![k.clone()])
                .unwrap_or_default();
            for k in oldest {
                self.seen_messages.remove(&k);
            }
        }
    }

    fn count(&self) -> usize {
        self.nodes.len()
    }

    fn to_json(&self) -> String {
        let nodes: Vec<&NodeInfo> = self.nodes.values().collect();
        serde_json::to_string_pretty(&nodes).unwrap_or_default()
    }
}

// ===== MESH NODE =====
struct MeshNode {
    registry: Arc<Mutex<NodeRegistry>>,
    port: u16,
    my_id: String,
    solar: bool,
    region: String,
}

impl MeshNode {
    fn new(port: u16, solar: bool, region: String) -> Self {
        let my_id = generate_node_id();
        println!("🦁 AfriMesh v0.2 — Réseau Mesh Africain");
        println!("📡 Node ID: {}", my_id);
        println!("🔌 Port: {}", port);
        println!("☀️  Solaire: {}", if solar { "Oui ☀️" } else { "Non" });
        println!("🌍 Région: {}", region);
        println!("💚 L'Afrique n'a pas besoin de permission\n");

        let registry = Arc::new(Mutex::new(NodeRegistry::new(my_id.clone(), port, solar, region.clone())));
        MeshNode { registry, port, my_id, solar, region }
    }

    fn start(&self) {
        let registry = self.registry.clone();
        let my_id = self.my_id.clone();
        let port = self.port;
        let solar = self.solar;
        let region = self.region.clone();

        // Thread 1: UDP Discovery (broadcast)
        let reg1 = registry.clone();
        let my_id1 = my_id.clone();
        let solar1 = solar;
        let region1 = region.clone();
        thread::spawn(move || {
            udp_discovery(reg1, my_id1, port, solar1, region1);
        });

        // Thread 2: TCP Server (relay)
        let reg2 = registry.clone();
        let my_id2 = my_id.clone();
        thread::spawn(move || {
            tcp_server(reg2, my_id2, port);
        });

        // Thread 3: Cleanup + Status
        let reg3 = registry.clone();
        let _my_id3 = my_id.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(10));
                {
                    let mut reg = reg3.lock().unwrap();
                    reg.cleanup_stale();
                    let count = reg.count();
                    println!("📊 Noeuds actifs: {} | Messages vus: {}", count, reg.seen_messages.len());
                }
            }
        });

        // Main: CLI
        let reg4 = registry.clone();
        let my_id4 = my_id.clone();
        cli_loop(reg4, my_id4);
    }
}

// ===== UDP DISCOVERY =====
fn udp_discovery(registry: Arc<Mutex<NodeRegistry>>, my_id: String, port: u16, solar: bool, region: String) {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Cannot bind UDP");
    socket.set_broadcast(true).expect("Cannot set broadcast");

    let discovery_port = 7946; // Standard mesh discovery port
    let bind_addr = format!("0.0.0.0:{}", discovery_port);

    // Try to bind to discovery port for listening
    let listener = UdpSocket::bind(&bind_addr);
    let listener = match listener {
        Ok(s) => { s }
        Err(_) => {
            // Port already in use, bind to any port
            UdpSocket::bind("0.0.0.0:0").unwrap()
        }
    };
    listener.set_read_timeout(Some(Duration::from_secs(2))).ok();

    let discovery_msg = MeshMessage::new("discovery", &my_id, &format!("{}|{}|{}", port, solar, region), 5);

    loop {
        // Broadcast discovery
        let broadcast_addr = "255.255.255.255:7946";
        let msg_bytes = discovery_msg.to_bytes();
        if socket.send_to(&msg_bytes, broadcast_addr).is_ok() {
            // Silent - don't spam console
        }

        // Listen for other nodes' discovery messages
        let mut buf = [0u8; 4096];
        match listener.recv_from(&mut buf) {
            Ok((len, src)) => {
                if let Some(msg) = MeshMessage::from_bytes(&buf[..len]) {
                    if msg.node_id != my_id && msg.msg_type == "discovery" {
                        let parts: Vec<&str> = msg.payload.split('|').collect();
                        if parts.len() >= 3 {
                            let other_port: u16 = parts[0].parse().unwrap_or(port);
                            let other_solar: bool = parts[1] == "true";
                            let other_region = parts[2].to_string();
                            let node_addr = format!("{}:{}", src.ip(), other_port);

                            let mut reg = registry.lock().unwrap();
                            reg.add_or_update(msg.node_id.clone(), node_addr.clone(), other_solar, other_region.clone());

                            // Send direct discovery back so they find us
                            let reply = MeshMessage::new("discovery", &my_id, &format!("{}|{}|{}", port, solar, region), 5);
                            if let Ok(reply_socket) = UdpSocket::bind("0.0.0.0:0") {
                                let _ = reply_socket.send_to(&reply.to_bytes(), src);
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }

        thread::sleep(Duration::from_secs(3));
    }
}

// ===== TCP SERVER (RELAY + WEB) =====
fn tcp_server(registry: Arc<Mutex<NodeRegistry>>, my_id: String, port: u16) {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).expect("Cannot bind TCP");
    listener.set_nonblocking(true).expect("Cannot set nonblocking");

    println!("🌐 Serveur mesh sur {}", addr);
    println!("🌍 Interface web sur http://localhost:{}", port);

    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                let reg = registry.clone();
                let my_id = my_id.clone();
                let port = port;
                thread::spawn(move || {
                    handle_tcp_connection(stream, addr, reg, my_id, port);
                });
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

fn handle_tcp_connection(mut stream: TcpStream, addr: SocketAddr, registry: Arc<Mutex<NodeRegistry>>, my_id: String, port: u16) {
    let mut buf = [0u8; 65536];
    match stream.read(&mut buf) {
        Ok(len) if len > 0 => {
            let data = &buf[..len];

            // Check if it's an HTTP request (from Chrome)
            if data.starts_with(b"GET ") || data.starts_with(b"POST ") {
                let reg = registry.lock().unwrap();
                let html = html_mesh_page(&reg, &my_id, port);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                    html.len(), html
                );
                let _ = stream.write_all(response.as_bytes());
                return;
            }

            // Otherwise it's a mesh message
            if let Some(msg) = MeshMessage::from_bytes(data) {
                let mut reg = registry.lock().unwrap();

                if reg.has_seen(&msg.msg_id) {
                    return; // Already seen, don't relay
                }
                reg.mark_seen(msg.msg_id.clone());

                match msg.msg_type.as_str() {
                    "block" => {
                        println!("📦 Bloc reçu de {} (TTL={})", msg.node_id, msg.ttl);
                        println!("   Payload: {}", &msg.payload[..msg.payload.len().min(80)]);
                    }
                    "tx" => {
                        println!("💸 Transaction reçue de {} (TTL={})", msg.node_id, msg.ttl);
                        println!("   Payload: {}", &msg.payload[..msg.payload.len().min(80)]);
                    }
                    "ping" => {
                        println!("📡 Ping de {}", msg.node_id);
                        let ack = MeshMessage::new("ack", &my_id, &format!("pong from {}", my_id), 1);
                        let _ = stream.write_all(&ack.to_bytes());
                    }
                    "ack" => {
                        // Silent
                    }
                    _ => {}
                }

                // Relay to other nodes if TTL > 0
                if msg.ttl > 0 {
                    let mut relay_msg = msg.clone();
                    relay_msg.ttl -= 1;
                    relay_msg.node_id = my_id.clone();
                    let relay_bytes = relay_msg.to_bytes();

                    let nodes: Vec<String> = reg.nodes.keys().cloned().collect();
                    drop(reg);

                    for node_id in nodes {
                        let reg = registry.lock().unwrap();
                        if let Some(info) = reg.nodes.get(&node_id) {
                            if let Ok(mut peer) = TcpStream::connect_timeout(&info.address.parse().unwrap_or(addr), Duration::from_secs(2)) {
                                let _ = peer.write_all(&relay_bytes);
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

// ===== CLI LOOP =====
fn cli_loop(registry: Arc<Mutex<NodeRegistry>>, my_id: String) {
    println!("\n📋 Commandes disponibles:");
    println!("  nodes    — Liste des noeuds connectés");
    println!("  ping     — Ping tous les noeuds");
    println!("  send X   — Envoyer un message à tous");
    println!("  status   — Statut du réseau");
    println!("  help     — Aide");
    println!("  quit     — Quitter\n");

    loop {
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            thread::sleep(Duration::from_secs(1));
            continue;
        }
        let input = input.trim();

        match input {
            "nodes" => {
                let reg = registry.lock().unwrap();
                if reg.nodes.is_empty() {
                    println!("Aucun noeud connecté. En attente...");
                } else {
                    println!("\n📡 Noeuds connectés ({}):", reg.nodes.len());
                    for (id, info) in &reg.nodes {
                        let solar_icon = if info.solar_powered { "☀️" } else { "🔌" };
                        println!("  {} {} — {} | {} | {}",
                            solar_icon, id, info.address, info.region,
                            chrono::DateTime::from_timestamp(info.last_seen, 0)
                                .map(|d| d.format("%H:%M:%S").to_string())
                                .unwrap_or_else(|| "?".to_string()));
                    }
                }
                println!();
            }
            "ping" => {
                let reg = registry.lock().unwrap();
                let nodes: Vec<(String, String)> = reg.nodes.iter()
                    .map(|(k, v)| (k.clone(), v.address.clone()))
                    .collect();
                drop(reg);

                if nodes.is_empty() {
                    println!("Aucun noeud à pinger.");
                } else {
                    for (node_id, addr) in nodes {
                        let ping = MeshMessage::new("ping", &my_id, "ping", 1);
                        match TcpStream::connect_timeout(&addr.parse().unwrap_or("127.0.0.1:8080".parse().unwrap()), Duration::from_secs(2)) {
                            Ok(mut stream) => {
                                let _ = stream.write_all(&ping.to_bytes());
                                println!("📡 Ping envoyé à {} ({})", node_id, addr);
                            }
                            Err(_) => {
                                println!("❌ Impossible de joindre {} ({})", node_id, addr);
                            }
                        }
                    }
                }
            }
            "status" => {
                let reg = registry.lock().unwrap();
                println!("\n🦁 AfriMesh — Statut du réseau");
                println!("  Mon ID: {}", my_id);
                println!("  Noeuds connectés: {}", reg.count());
                println!("  Messages relayés: {}", reg.seen_messages.len());
                println!("  Solaire: {}", if reg.solar { "Oui ☀️" } else { "Non 🔌" });
                println!("  Région: {}", reg.region);
                println!("  Port: {}", reg.my_port);
                println!();
            }
            cmd if cmd.starts_with("send ") => {
                let payload = &cmd[5..];
                let msg = MeshMessage::new("relay", &my_id, payload, 5);
                let reg = registry.lock().unwrap();
                let nodes: Vec<(String, String)> = reg.nodes.iter()
                    .map(|(k, v)| (k.clone(), v.address.clone()))
                    .collect();
                drop(reg);

                let bytes = msg.to_bytes();
                for (node_id, addr) in &nodes {
                    if let Ok(mut stream) = TcpStream::connect_timeout(&addr.parse().unwrap_or("127.0.0.1:8080".parse().unwrap()), Duration::from_secs(2)) {
                        let _ = stream.write_all(&bytes);
                        println!("📤 Envoyé à {} ({})", node_id, addr);
                    }
                }
                if nodes.is_empty() {
                    println!("Aucun noeud connecté. Message en attente.");
                }
            }
            "help" => {
                println!("\n📋 Commandes:");
                println!("  nodes    — Liste des noeuds");
                println!("  ping     — Ping tous les noeuds");
                println!("  send X   — Envoyer un message");
                println!("  status   — Statut du réseau");
                println!("  quit     — Quitter\n");
            }
            "quit" | "exit" => {
                println!("🦁 Au revoir! Le réseau AfriMesh continue de rugir~ 💚");
                std::process::exit(0);
            }
            "" => {}
            _ => {
                println!("Commande inconnue: {}. Tape 'help' pour l'aide.", input);
            }
        }
    }
}

fn html_mesh_page(reg: &NodeRegistry, my_id: &str, port: u16) -> String {
    let node_count = reg.count();
    let msg_count = reg.seen_messages.len();
    let solar = reg.solar;
    let region = &reg.region;

    let mut nodes_html = String::new();
    for (id, info) in &reg.nodes {
        let icon = if info.solar_powered { "☀️" } else { "🔌" };
        let last_seen = chrono::DateTime::from_timestamp(info.last_seen, 0)
            .map(|d| d.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "?".to_string());
        nodes_html.push_str(&format!(
            r#"<div class="node"><span class="icon">{}</span><span class="id">{}</span><span class="addr">{}</span><span class="region">{}</span><span class="time">{}</span></div>"#,
            icon, id, info.address, info.region, last_seen
        ));
    }
    if nodes_html.is_empty() {
        nodes_html = r#"<div class="empty">Aucun noeud connecté. En attente... ⏳</div>"#.to_string();
    }

    format!(r##"<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>🦁 AfriMesh — Réseau Mesh Africain</title><meta http-equiv="refresh" content="5"><style>body{{font-family:sans-serif;background:linear-gradient(135deg,#1a3d2e,#0d1f17);color:#f5e9d4;padding:20px;margin:0;}}h1{{color:#d4a437;text-align:center;}}h2{{color:#d4a437;}}.stat-box{{display:inline-block;background:rgba(212,164,55,0.15);border:1px solid #d4a437;border-radius:12px;padding:15px 20px;margin:8px;text-align:center;min-width:120px;}}.stat-num{{font-size:2em;color:#d4a437;font-weight:bold;}}.stat-label{{color:#a8c5a8;font-size:0.85em;}}.card{{background:rgba(212,164,55,0.1);border:1px solid #d4a437;border-radius:12px;padding:20px;margin:15px auto;max-width:800px;}}.node{{display:flex;justify-content:space-between;align-items:center;background:rgba(0,0,0,0.3);padding:12px;margin:6px 0;border-radius:8px;font-size:0.95em;}}.icon{{font-size:1.3em;}}.id{{color:#7fcf7f;font-family:monospace;font-weight:bold;}}.addr{{color:#a8c5a8;font-family:monospace;font-size:0.85em;}}.region{{color:#d4a437;font-size:0.85em;}}.time{{color:#a8c5a8;font-size:0.85em;}}.empty{{text-align:center;color:#a8c5a8;padding:20px;}}.my-id{{text-align:center;font-family:monospace;color:#7fcf7f;font-size:1.1em;margin:10px 0;}}.solar-badge{{display:inline-block;background:#d4a437;color:#1a3d2e;padding:3px 10px;border-radius:12px;font-size:0.8em;font-weight:bold;}}footer{{text-align:center;margin-top:40px;color:#a8c5a8;}}</style></head><body><h1>🦁 AfriMesh</h1><p style="text-align:center;">Le réseau mesh 100% africain 💚</p><div style="text-align:center;"><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📡 Noeuds</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">📦 Messages</div></div><div class="stat-box"><div class="stat-num">{}</div><div class="stat-label">🌍 Région</div></div></div><div class="card"><h2>📡 Mon Noeud</h2><div class="my-id">{}</div><p style="text-align:center;">Port mesh: {} | Port web: {} | Solaire: {}</p></div><div class="card"><h2>🌐 Noeuds connectés</h2>{}</div><footer>🦁 AfriMesh — Un seul réseau pour l'Afrique 💚<br><small>Page auto-actualisée toutes les 5 secondes</small></footer></body></html>"##,
        node_count,
        msg_count,
        if solar { "☀️ Oui" } else { "🔌 Non" },
        my_id,
        reg.my_port,
        port,
        if solar { "<span class=\"solar-badge\">☀️ Solaire</span>" } else { "🔌 Secteur" },
        nodes_html,
    )
}

// ===== MAIN =====
fn main() {
    let args: Vec<String> = std::env::args().collect();

    let port: u16 = args.iter().position(|a| a == "--port")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(8090);

    let solar = args.iter().any(|a| a == "--solar");

    let region = args.iter().position(|a| a == "--region")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "Afrique".to_string());

    let node = MeshNode::new(port, solar, region);
    node.start();
}
