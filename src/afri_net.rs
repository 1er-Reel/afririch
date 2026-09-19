/// v1.97 — L'INTERNET AFRI 🌐◈⬡
/// Notre propre internet, construit from scratch — Rust std uniquement.
/// Pas de HTTP. Pas de TCP occidental dans nos échanges : le PROTOCOLE MACHINE.
/// ◈⬡ = connexion (handshake)   ◉ = demande (SCN)   ⬔ = envoi (EXE)
/// ▤ = réponse (donnée)          ⟠⬠ = fermeture
/// AfriChain tourne SUR notre internet — pas l'inverse.
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use crate::afri_json::{JsonValue, from_str, to_string};

pub const NET_PORT_DEFAUT: u16 = 8181;

pub struct NetStore {
    pub connexions: u64,
    pub requetes: u64,
    pub journal: Vec<String>, // 100 dernières lignes du trafic machine
    pub chemin: String,
}

impl NetStore {
    pub fn load() -> Self {
        let chemin = crate::data_path("net.json");
        if let Ok(data) = std::fs::read_to_string(&chemin) {
            if let Ok(v) = from_str(&data) {
                return NetStore::from_json(&v, chemin);
            }
        }
        NetStore { connexions: 0, requetes: 0, journal: Vec::new(), chemin }
    }

    pub fn save(&self) {
        let _ = std::fs::write(&self.chemin, to_string(&self.to_json()));
    }

    pub fn noter(&mut self, ligne: String) {
        self.journal.push(ligne);
        if self.journal.len() > 100 { self.journal.remove(0); }
        self.save();
    }

    fn to_json(&self) -> JsonValue {
        let mut m = HashMap::new();
        m.insert("connexions".to_string(), JsonValue::Int(self.connexions as i64));
        m.insert("requetes".to_string(), JsonValue::Int(self.requetes as i64));
        let arr: Vec<JsonValue> = self.journal.iter().map(|l| JsonValue::Str(l.clone())).collect();
        m.insert("journal".to_string(), JsonValue::Array(arr));
        JsonValue::Object(m)
    }

    fn from_json(v: &JsonValue, chemin: String) -> Self {
        let mut s = NetStore { connexions: 0, requetes: 0, journal: Vec::new(), chemin };
        if let Some(o) = v.as_object() {
            s.connexions = o.get("connexions").and_then(|x| x.as_i64()).unwrap_or(0) as u64;
            s.requetes = o.get("requetes").and_then(|x| x.as_i64()).unwrap_or(0) as u64;
            if let Some(arr) = o.get("journal").and_then(|a| a.as_array()) {
                for l in arr {
                    if let Some(t) = l.as_str() { s.journal.push(t.to_string()); }
                }
            }
        }
        s
    }
}

/// Traiter UNE connexion machine. Le protocole est en lignes :
///   ◈⬡            → handshake, le serveur répond sa carte d'identité
///   ◉ /chemin      → demande une page (SCN)
///   ⬔ /chemin c=corp  → envoi avec corps urlencodé (EXE)
///   ⟠⬠            → fermeture propre
fn traiter_connexion(mut stream: TcpStream, state: std::sync::Arc<crate::AppState>) {    let peer = stream.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "?".to_string());
    // v1.97 — le porte-clés de session de CETTE connexion machine
    let mut cookies: HashMap<String, String> = HashMap::new();
    {
        let mut net = state.net.lock().unwrap();
        net.connexions += 1;
        net.noter(format!("◈⬡ connexion machine depuis {}", peer));
    }
    let _ = writeln!(stream, "◈⬡ AMION-NET — l'internet de l'Afrique. Protocole machine. Pas de HTTP.");
    let _ = writeln!(stream, "◉ /chemin pour demander — ⬔ /chemin corps pour envoyer — ⟠⬠ pour fermer");
    let _ = stream.flush();

    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                // traiter ligne par ligne (protocole machine = lignes)
                while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                    let ligne: String = String::from_utf8_lossy(&buf[..pos]).trim().to_string();
                    buf.drain(..pos + 1);
                    if ligne.is_empty() { continue; }
                    if ligne == "⟠⬠" {
                        let _ = writeln!(stream, "⟠⬠ la connexion se ferme — l'Afrique reste");
                        let mut net = state.net.lock().unwrap();
                        net.noter(format!("⟠⬠ fermeture depuis {}", peer));
                        return;
                    }
                    if ligne == "◈⬡" {
                        let (blocs, users) = {
                            let chain = state.chain.lock().unwrap();
                            let users = state.users.lock().unwrap();
                            (chain.blocks.len(), users.count())
                        };
                        let _ = writeln!(stream, "◈⬡ AMION-NET — {} blocs — {} âmes — résolu par l'Afrique", blocs, users);
                        continue;
                    }
                    // ◉ /chemin — demande (SCN)
                    if let Some(chemin) = ligne.strip_prefix("◉ ") {
                        let reponse = crate::handle_request_machine_avec_cookies("GET", chemin, "", &state, &peer, &mut cookies);
                        let _ = writeln!(stream, "▤ {} octets", reponse.len());
                        let _ = stream.write_all(reponse.as_bytes());
                        let mut net = state.net.lock().unwrap();
                        net.requetes += 1;
                        net.noter(format!("◉ {} depuis {} → {} octets", chemin, peer, reponse.len()));
                        continue;
                    }
                    // ⬔ /chemin corps — envoi (EXE)
                    if let Some(reste) = ligne.strip_prefix("⬔ ") {
                        let mut parts = reste.splitn(2, ' ');
                        let chemin = parts.next().unwrap_or("/").to_string();
                        let corps = parts.next().unwrap_or("").to_string();
                        let reponse = crate::handle_request_machine_avec_cookies("POST", &chemin, &corps, &state, &peer, &mut cookies);
                        let _ = writeln!(stream, "▤ {} octets", reponse.len());
                        let _ = stream.write_all(reponse.as_bytes());
                        let mut net = state.net.lock().unwrap();
                        net.requetes += 1;
                        net.noter(format!("⬔ {} depuis {} → {} octets", chemin, peer, reponse.len()));
                        continue;
                    }
                    let _ = writeln!(stream, "◈ je ne connais pas « {} » — ◉ pour demander, ⬔ pour envoyer, ⟠⬠ pour fermer", ligne);
                }
                if buf.len() > 1_000_000 { break; } // garde-fou
            }
        }
    }
}

/// Lancer le serveur INTERNET AFRI — notre internet machine sur notre port.
pub fn lancer_net(state: std::sync::Arc<crate::AppState>, port: u16) {
    std::thread::spawn(move || {
        let listener = match TcpListener::bind(("0.0.0.0", port)) {
            Ok(l) => l,
            Err(e) => { println!("❌ INTERNET AFRI: impossible d'écouter sur {} : {}", port, e); return; }
        };
        println!("🌐 INTERNET AFRI en écoute sur le port {} — protocole machine ◈⬡◉⬔▤⟠⬠", port);
        println!("   (nc <ip> {} puis : ◉ /  pour la page d'accueil — pas de HTTP, pas d'Occident)", port);
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    let st = std::sync::Arc::clone(&state);
                    std::thread::spawn(move || traiter_connexion(s, st));
                }
                Err(_) => continue,
            }
        }
    });
}
