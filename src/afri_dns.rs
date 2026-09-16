/// v1.72 — AFRIDNS : Le Serveur de Noms de l'Afrique 🌐
/// Notre propre DNS, construit from scratch — Rust std uniquement.
/// Plus jamais le DNS de l'Occident : afri.wari, planete.verte, blockchain.africa
/// sont résolus par NOUS, gravés sur NOTRE blockchain.
use std::collections::HashMap;
use std::net::UdpSocket;
use crate::afri_json::{JsonValue, from_str, to_string};
use crate::afri_time::now_timestamp;

pub struct EnregistrementDns {
    pub domaine: String,   // ex: "afri.wari"
    pub ip: String,        // ex: "127.0.0.1"
    pub proprietaire: String,
    pub bloc: u64,
    pub enregistre_le: i64,
}

pub struct DnsStore {
    pub registre: Vec<EnregistrementDns>,
    pub chemin: String,
}

impl DnsStore {
    pub fn load() -> Self {
        let chemin = crate::data_path("dns.json");
        if let Ok(data) = std::fs::read_to_string(&chemin) {
            if let Ok(v) = from_str(&data) {
                return DnsStore::from_json(&v, chemin);
            }
        }
        DnsStore { registre: Vec::new(), chemin }
    }

    pub fn save(&self) {
        let s = to_string(&self.to_json());
        let _ = std::fs::write(&self.chemin, s);
    }

    pub fn enregistrer(&mut self, domaine: &str, ip: &str, proprietaire: &str, bloc: u64) {
        if let Some(e) = self.registre.iter_mut().find(|e| e.domaine == domaine) {
            e.ip = ip.to_string();
            e.bloc = bloc;
            self.save();
            return;
        }
        self.registre.push(EnregistrementDns {
            domaine: domaine.to_lowercase(),
            ip: ip.to_string(),
            proprietaire: proprietaire.to_string(),
            bloc,
            enregistre_le: now_timestamp(),
        });
        self.save();
    }

    pub fn resoudre(&self, domaine: &str) -> Option<String> {
        let d = domaine.trim_end_matches('.').to_lowercase();
        self.registre.iter().find(|e| e.domaine == d).map(|e| e.ip.clone())
    }

    fn to_json(&self) -> JsonValue {
        let arr: Vec<JsonValue> = self.registre.iter().map(|e| {
            let mut m = HashMap::new();
            m.insert("domaine".to_string(), JsonValue::Str(e.domaine.clone()));
            m.insert("ip".to_string(), JsonValue::Str(e.ip.clone()));
            m.insert("proprietaire".to_string(), JsonValue::Str(e.proprietaire.clone()));
            m.insert("bloc".to_string(), JsonValue::Int(e.bloc as i64));
            m.insert("enregistre_le".to_string(), JsonValue::Int(e.enregistre_le));
            JsonValue::Object(m)
        }).collect();
        let mut m = HashMap::new();
        m.insert("registre".to_string(), JsonValue::Array(arr));
        JsonValue::Object(m)
    }

    fn from_json(v: &JsonValue, chemin: String) -> Self {
        let mut s = DnsStore { registre: Vec::new(), chemin };
        if let Some(arr) = v.as_object().and_then(|m| m.get("registre")).and_then(|a| a.as_array()) {
            for e in arr {
                let o = match e.as_object() { Some(o) => o, None => continue };
                s.registre.push(EnregistrementDns {
                    domaine: o.get("domaine").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                    ip: o.get("ip").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                    proprietaire: o.get("proprietaire").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                    bloc: o.get("bloc").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                    enregistre_le: o.get("enregistre_le").and_then(|x| x.as_i64()).unwrap_or(0),
                });
            }
        }
        s
    }
}

/// Extraire le nom de domaine d'une question DNS (format labels)
fn extraire_domaine(paquet: &[u8]) -> Option<String> {
    if paquet.len() < 12 { return None; }
    let mut i = 12usize;
    let mut labels: Vec<String> = Vec::new();
    loop {
        if i >= paquet.len() { return None; }
        let len = paquet[i] as usize;
        if len == 0 { break; }
        if len & 0xC0 != 0 { return None; } // compression — pas dans une question
        i += 1;
        if i + len > paquet.len() { return None; }
        labels.push(String::from_utf8_lossy(&paquet[i..i + len]).to_string());
        i += len;
    }
    Some(labels.join("."))
}

/// Construire une réponse DNS A valide, from scratch
fn construire_reponse(paquet: &[u8], ip: Option<&str>) -> Vec<u8> {
    let mut rep = Vec::new();
    // Header : ID + flags (réponse, récursif)
    rep.extend_from_slice(&paquet[0..2]);          // ID
    rep.extend_from_slice(&[0x85, 0x80]);          // flags: QR=1, AA=1, RD=1, RA=1
    // QD=1, AN=1 si trouvé sinon 0 (NXDOMAIN: RCODE=3)
    let an: u16 = if ip.is_some() { 1 } else { 0 };
    let rcode: u8 = if ip.is_some() { 0x80 } else { 0x83 }; // 0x80 normal | 0x83 = NXDOMAIN
    rep[2] = 0x85; rep[3] = rcode;
    rep.extend_from_slice(&[0x00, 0x01]);
    rep.extend_from_slice(&an.to_be_bytes());
    rep.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    // Question recopiée telle quelle (jusqu'à la fin de QNAME + QTYPE + QCLASS)
    let mut i = 12usize;
    while i < paquet.len() && paquet[i] != 0 { i += 1 + paquet[i] as usize; }
    i += 1; // le 0 final
    i += 4; // QTYPE + QCLASS
    rep.extend_from_slice(&paquet[12..i]);
    if let Some(ip) = ip {
        // Réponse : pointeur vers la question (0xC00C), TYPE A, CLASS IN, TTL 60s
        rep.extend_from_slice(&[0xC0, 0x0C]);
        rep.extend_from_slice(&[0x00, 0x01]);          // TYPE = A
        rep.extend_from_slice(&[0x00, 0x01]);          // CLASS = IN
        rep.extend_from_slice(&[0x00, 0x00, 0x00, 0x3C]); // TTL 60
        rep.extend_from_slice(&[0x00, 0x04]);          // RDLENGTH 4
        let octets: Vec<u8> = ip.split('.').filter_map(|p| p.parse::<u8>().ok()).collect();
        rep.extend_from_slice(&octets);
    }
    rep
}

/// Lancer le serveur DNS souverain. Port 53 si possible, sinon 5353.
/// Chaque requête est résolue par NOTRE registre — jamais un DNS occidental.
pub fn lancer_dns(store: std::sync::Arc<std::sync::Mutex<DnsStore>>) {
    std::thread::spawn(move || {
        let port = match UdpSocket::bind("0.0.0.0:53") {
            Ok(_) => 53u16,
            Err(_) => 5353u16, // Android sans root : port élevé
        };
        let sock = match UdpSocket::bind(format!("0.0.0.0:{}", port)) {
            Ok(s) => s,
            Err(e) => { println!("❌ AfriDNS: impossible d'écouter ({}): {}", port, e); return; }
        };
        println!("🌐 AfriDNS souverain en écoute sur le port {} — .afri .wari .verte .africa", port);
        if port == 5353 {
            println!("   (Android sans root → port 5353. Sur PC/routeur: mets le DNS sur ce port ou redirige le 53)");
        }
        let mut buf = [0u8; 512];
        loop {
            match sock.recv_from(&mut buf) {
                Ok((n, src)) => {
                    let domaine = match extraire_domaine(&buf[..n]) {
                        Some(d) => d,
                        None => continue,
                    };
                    let ip = {
                        let s = store.lock().unwrap();
                        s.resoudre(&domaine)
                    };
                    let rep = construire_reponse(&buf[..n], ip.as_deref());
                    let _ = sock.send_to(&rep, src);
                    match ip {
                        Some(ip) => println!("🌐 AfriDNS: {} → {} (résolu par l'Afrique)", domaine, ip),
                        None => println!("🌐 AfriDNS: {} → inconnu dans le registre africain", domaine),
                    }
                }
                Err(_) => continue,
            }
        }
    });
}
