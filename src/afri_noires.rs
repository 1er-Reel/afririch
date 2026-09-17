// ===== AFRI NOIRES — v1.83: notre WhatsApp africain 💬 =====
// LES NOIRES : la messagerie souveraine de la Planète Verte.
// Chaque message passe de puce à puce sur AfriChain — zéro serveur occidental.
// + Les codes de compte : comme Orange Money, chaque opération importante
//   envoie un code de vérification dans la messagerie de l'utilisateur.

use std::collections::HashMap;
use crate::afri_json::{JsonValue, to_string, from_str};
use crate::afri_time::now_timestamp;

/// Un message LES NOIRES — un pli plié qui voyage de main en main 📨
#[derive(Debug, Clone)]
pub struct MsgNoires {
    pub id: String,
    pub de: String,       // username expéditeur
    pub a: String,        // username destinataire
    pub texte: String,
    pub heure: i64,
    pub lu: bool,
}

/// Un code de compte — la notification bancaire de l'Afrique 🔢
/// Comme Orange Money : "Code 4821 — Vous avez envoyé 50 AFR à aisha"
#[derive(Debug, Clone)]
pub struct CodeCompte {
    pub id: String,
    pub username: String,  // destinataire du code
    pub code: String,      // les 6 chiffres
    pub texte: String,     // le message qui accompagne
    pub heure: i64,
    pub lu: bool,
}

/// Le magasin LES NOIRES — messages + codes, persisté dans noires.json
#[derive(Debug, Clone)]
pub struct NoiresStore {
    pub messages: Vec<MsgNoires>,
    pub codes: Vec<CodeCompte>,
    pub chemin: String,
}

impl NoiresStore {
    pub fn nouveau() -> Self {
        let chemin = crate::data_path("noires.json");
        let s = match std::fs::read_to_string(&chemin) {
            Ok(data) => {
                let v = from_str(&data).unwrap_or(JsonValue::Object(HashMap::new()));
                NoiresStore::from_json(&v)
            }
            Err(_) => NoiresStore { messages: Vec::new(), codes: Vec::new(), chemin },
        };
        s
    }

    pub fn sauvegarder(&self) {
        let _ = std::fs::write(&self.chemin, to_string(&self.to_json()));
    }

    /// Envoyer un message entre deux utilisateurs
    pub fn envoyer(&mut self, de: &str, a: &str, texte: &str, heure: i64) -> MsgNoires {
        let id = format!("NQ{}", self.messages.len() + 1);
        let msg = MsgNoires {
            id: id.clone(),
            de: de.to_string(),
            a: a.to_string(),
            texte: texte.to_string(),
            heure,
            lu: false,
        };
        self.messages.push(msg.clone());
        self.sauvegarder();
        msg
    }

    /// Conversation entre deux utilisateurs, triée par heure
    pub fn conversation(&self, u1: &str, u2: &str) -> Vec<&MsgNoires> {
        let mut conv: Vec<&MsgNoires> = self.messages.iter()
            .filter(|m| (m.de == u1 && m.a == u2) || (m.de == u2 && m.a == u1))
            .collect();
        conv.sort_by_key(|m| m.heure);
        conv
    }

    /// Boîte de réception : dernier message par contact (données clonées, sans borrow)
    pub fn boite_de(&self, username: &str) -> Vec<(String, MsgNoires)> {
        let mut derniers: HashMap<String, &MsgNoires> = HashMap::new();
        for m in &self.messages {
            if m.de == username {
                derniers.insert(m.a.clone(), m);
            } else if m.a == username {
                derniers.insert(m.de.clone(), m);
            }
        }
        let mut liste: Vec<(String, MsgNoires)> = derniers
            .into_iter()
            .map(|(k, m)| (k, m.clone()))
            .collect();
        liste.sort_by(|a, b| b.1.heure.cmp(&a.1.heure));
        liste
    }

    /// Marquer toute une conversation comme lue
    pub fn marquer_lu(&mut self, username: &str, autre: &str) {
        let mut change = false;
        for m in self.messages.iter_mut() {
            if m.a == username && m.de == autre && !m.lu {
                m.lu = true;
                change = true;
            }
        }
        if change { self.sauvegarder(); }
    }

    /// Total des messages non lus
    pub fn non_lus(&self, username: &str) -> usize {
        self.messages.iter().filter(|m| m.a == username && !m.lu).count()
    }

    /// Non lus d'un contact précis
    pub fn non_lus_de(&self, username: &str, autre: &str) -> usize {
        self.messages.iter().filter(|m| m.a == username && m.de == autre && !m.lu).count()
    }

    // ===== LES CODES DE COMPTE 🔢 =====

    /// Envoyer un code de compte à un utilisateur (6 chiffres, comme Orange Money)
    pub fn envoyer_code(&mut self, username: &str, texte: &str, heure: i64) -> CodeCompte {
        let code = format!("{:06}", crate::afri_rng::random_u64() % 1_000_000);
        let id = format!("CD{}", self.codes.len() + 1);
        let c = CodeCompte {
            id,
            username: username.to_string(),
            code: code.clone(),
            texte: texte.to_string(),
            heure,
            lu: false,
        };
        self.codes.push(c.clone());
        self.sauvegarder();
        c
    }

    /// Les codes d'un utilisateur (les 30 derniers)
    pub fn codes_de(&self, username: &str) -> Vec<CodeCompte> {
        let mut v: Vec<CodeCompte> = self.codes.iter().filter(|c| c.username == username).cloned().collect();
        v.sort_by_key(|c| c.heure);
        if v.len() > 30 { v.drain(0..v.len() - 30); }
        v
    }

    /// Codes non lus
    pub fn codes_non_lus(&self, username: &str) -> usize {
        self.codes.iter().filter(|c| c.username == username && !c.lu).count()
    }

    /// Marquer les codes comme lus
    pub fn codes_marquer_lus(&mut self, username: &str) {
        let mut change = false;
        for c in self.codes.iter_mut() {
            if c.username == username && !c.lu {
                c.lu = true;
                change = true;
            }
        }
        if change { self.sauvegarder(); }
    }

    pub fn to_json(&self) -> JsonValue {
        let mut msgs = Vec::new();
        for m in &self.messages {
            let mut o = HashMap::new();
            o.insert("id".to_string(), JsonValue::Str(m.id.clone()));
            o.insert("de".to_string(), JsonValue::Str(m.de.clone()));
            o.insert("a".to_string(), JsonValue::Str(m.a.clone()));
            o.insert("texte".to_string(), JsonValue::Str(m.texte.clone()));
            o.insert("heure".to_string(), JsonValue::Int(m.heure));
            o.insert("lu".to_string(), JsonValue::Bool(m.lu));
            msgs.push(JsonValue::Object(o));
        }
        let mut cds = Vec::new();
        for c in &self.codes {
            let mut o = HashMap::new();
            o.insert("id".to_string(), JsonValue::Str(c.id.clone()));
            o.insert("username".to_string(), JsonValue::Str(c.username.clone()));
            o.insert("code".to_string(), JsonValue::Str(c.code.clone()));
            o.insert("texte".to_string(), JsonValue::Str(c.texte.clone()));
            o.insert("heure".to_string(), JsonValue::Int(c.heure));
            o.insert("lu".to_string(), JsonValue::Bool(c.lu));
            cds.push(JsonValue::Object(o));
        }
        let mut root = HashMap::new();
        root.insert("messages".to_string(), JsonValue::Array(msgs));
        root.insert("codes".to_string(), JsonValue::Array(cds));
        JsonValue::Object(root)
    }

    pub fn from_json(v: &JsonValue) -> Self {
        let mut messages = Vec::new();
        let mut codes = Vec::new();
        if let Some(obj) = v.as_object() {
            if let Some(JsonValue::Array(arr)) = obj.get("messages") {
                for item in arr {
                    if let Some(m) = item.as_object() {
                        let g = |k: &str| m.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let gi = |k: &str| m.get(k).and_then(|v| v.as_i64()).unwrap_or(0);
                        let gb = |k: &str| m.get(k).and_then(|v| v.as_bool()).unwrap_or(false);
                        messages.push(MsgNoires {
                            id: g("id"), de: g("de"), a: g("a"),
                            texte: g("texte"), heure: gi("heure"), lu: gb("lu"),
                        });
                    }
                }
            }
            if let Some(JsonValue::Array(arr)) = obj.get("codes") {
                for item in arr {
                    if let Some(m) = item.as_object() {
                        let g = |k: &str| m.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let gi = |k: &str| m.get(k).and_then(|v| v.as_i64()).unwrap_or(0);
                        let gb = |k: &str| m.get(k).and_then(|v| v.as_bool()).unwrap_or(false);
                        codes.push(CodeCompte {
                            id: g("id"), username: g("username"), code: g("code"),
                            texte: g("texte"), heure: gi("heure"), lu: gb("lu"),
                        });
                    }
                }
            }
        }
        NoiresStore { messages, codes, chemin: crate::data_path("noires.json") }
    }
}
