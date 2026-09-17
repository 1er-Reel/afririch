// ===== AFRI NOTIFS — v1.86: les notifications de l'Afrique 🔔 =====
// Comme Facebook : chaque like, chaque commentaire, chaque message t'arrive
// dans ta cloche. L'Afrique ne laisse personne dans le silence. 💚

use std::collections::HashMap;
use crate::afri_json::{JsonValue, to_string, from_str};

/// Une notification — un petit coup de cloche 🔔
#[derive(Debug, Clone)]
pub struct Notif {
    pub id: u64,
    pub pour: String,     // username destinataire
    pub texte: String,    // "aisha a aimé ton post"
    pub lien: String,    // "/plante" ou "/noires?contact=aisha"
    pub heure: i64,
    pub lu: bool,
}

/// Le magasin de notifications — persisté dans notifs.json
#[derive(Debug, Clone)]
pub struct NotifStore {
    pub notifs: Vec<Notif>,
    pub prochain_id: u64,
    pub chemin: String,
}

impl NotifStore {
    pub fn nouveau() -> Self {
        let chemin = crate::data_path("notifs.json");
        match std::fs::read_to_string(&chemin) {
            Ok(data) => {
                let v = from_str(&data).unwrap_or(JsonValue::Object(HashMap::new()));
                NotifStore::from_json(&v)
            }
            Err(_) => NotifStore { notifs: Vec::new(), prochain_id: 1, chemin },
        }
    }

    pub fn sauvegarder(&self) {
        let _ = std::fs::write(&self.chemin, to_string(&self.to_json()));
    }

    /// Envoyer une notification à un utilisateur 📬
    /// (silencieux si l'utilisateur s'envoie une notif à lui-même)
    pub fn notifier(&mut self, pour: &str, texte: &str, lien: &str, heure: i64) {
        if pour.is_empty() {
            return;
        }
        let n = Notif {
            id: self.prochain_id,
            pour: pour.to_string(),
            texte: texte.to_string(),
            lien: lien.to_string(),
            heure,
            lu: false,
        };
        self.prochain_id += 1;
        self.notifs.push(n);
        // Garde les 300 dernières notifications par personne
        let mut par_user: HashMap<String, usize> = HashMap::new();
        self.notifs.retain(|n| {
            let c = par_user.entry(n.pour.clone()).or_insert(0);
            *c += 1;
            *c <= 300
        });
        self.sauvegarder();
    }

    /// Les notifications d'un utilisateur, les plus récentes d'abord
    pub fn notifs_de(&self, username: &str) -> Vec<&Notif> {
        self.notifs.iter().filter(|n| n.pour == username).collect()
    }

    /// Combien de non-lues pour un utilisateur (pour le badge 🔔)
    pub fn non_lus(&self, username: &str) -> usize {
        self.notifs.iter().filter(|n| n.pour == username && !n.lu).count()
    }

    /// Marquer toutes les notifications d'un utilisateur comme lues
    pub fn marquer_lus(&mut self, username: &str) {
        for n in self.notifs.iter_mut() {
            if n.pour == username {
                n.lu = true;
            }
        }
        self.sauvegarder();
    }

    pub fn to_json(&self) -> JsonValue {
        let mut arr = Vec::new();
        for n in &self.notifs {
            let mut o = HashMap::new();
            o.insert("id".to_string(), JsonValue::Int(n.id as i64));
            o.insert("pour".to_string(), JsonValue::Str(n.pour.clone()));
            o.insert("texte".to_string(), JsonValue::Str(n.texte.clone()));
            o.insert("lien".to_string(), JsonValue::Str(n.lien.clone()));
            o.insert("heure".to_string(), JsonValue::Int(n.heure));
            o.insert("lu".to_string(), JsonValue::Bool(n.lu));
            arr.push(JsonValue::Object(o));
        }
        let mut root = HashMap::new();
        root.insert("notifs".to_string(), JsonValue::Array(arr));
        root.insert("prochain_id".to_string(), JsonValue::Int(self.prochain_id as i64));
        JsonValue::Object(root)
    }

    pub fn from_json(v: &JsonValue) -> Self {
        let mut notifs = Vec::new();
        let mut prochain_id = 1u64;
        if let Some(obj) = v.as_object() {
            prochain_id = obj.get("prochain_id").and_then(|x| x.as_i64()).unwrap_or(1) as u64;
            if let Some(JsonValue::Array(arr)) = obj.get("notifs") {
                for item in arr {
                    if let Some(o) = item.as_object() {
                        notifs.push(Notif {
                            id: o.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
                            pour: o.get("pour").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            texte: o.get("texte").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            lien: o.get("lien").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            heure: o.get("heure").and_then(|v| v.as_i64()).unwrap_or(0),
                            lu: o.get("lu").and_then(|v| v.as_bool()).unwrap_or(false),
                        });
                    }
                }
            }
        }
        NotifStore { notifs, prochain_id, chemin: crate::data_path("notifs.json") }
    }
}
