// ===== AFRI PUCE — PLANÈTE VERTE: la puce africaine + notre USSD =====
// v1.70 — La puce verte : APN, activation, réseau unique 54 pays.
// Chaque puce est gravée sur la blockchain à son activation.
// Notre USSD : #144# (Afri.Wari), #100# (solde), #145# (info puce).
// Orange Money disparaît. La puce EST le portefeuille.

use std::collections::HashMap;
use crate::afri_json::{JsonValue, to_string, from_str};
use crate::afri_time::now_timestamp;

/// L'APN de la Planète Verte — le point d'entrée du réseau africain.
/// C'est LE paramètre que tape l'utilisateur dans son téléphone :
/// Réglages → Réseaux mobiles → Nom des points d'accès → Nouvel APN.
pub const APN_NAME: &str = "planete-verte";
pub const APN_APN: &str = "afri.planete.verte";
pub const APN_MMSC: &str = "http://afri.wari/mms";
pub const APN_MCC: &str = "600"; // MCC 600 = Afrique (plage panafricaine)
pub const APN_MNC: &str = "54"; // MNC 54 = les 54 pays, un seul réseau
pub const APN_TYPE: &str = "default,mms,supl";
pub const APN_PROTOCOLE: &str = "IPv4/IPv6";

/// Les codes USSD de la Planète Verte — notre propre téléphonie.
/// L'utilisateur tape ces codes dans le clavier téléphone.
pub fn ussd_codes() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("#144#", "Afri.Wari — Envoi, retrait, solde, épargne", "🏦"),
        ("#100#", "Solde AFR + graines + FCFA", "💰"),
        ("#145#", "Info puce : numéro vert, pays, état du réseau", "🌿"),
        ("#146#", "Recharger son compte par code (graines)", "🪙"),
        ("#111#", "Urgence SOS — contacter le nœud mesh le plus proche", "🆘"),
        ("*#06#", "Identité de la puce (IMSI vert gravé sur blockchain)", "🔢"),
    ]
}

/// Une puce Planète Verte — chaque puce activée vit dans puce.json
#[derive(Debug, Clone)]
pub struct Puce {
    pub id: String,           // ex: PV-223-000001
    pub numero_vert: String,  // ex: +223 76 00 00 01 (numéro du réseau vert)
    pub pays: String,         // ex: Mali
    pub code_pays: String,    // ex: +223
    pub username: String,     // propriétaire (compte AfriChain)
    pub apn: String,          // afri.planete.verte
    pub etat: String,         // ACTIVE / SUSPENDUE / PERDUE
    pub active_le: i64,       // timestamp activation
    pub bloc_activation: u64, // bloc blockchain où la puce est gravée
    pub pin_hash: String,     // code PIN puce (hashé AfriHash-256)
    pub puk_hash: String,     // code PUK puce (hashé)
    pub tentative_pin: u32,   // tentatives PIN échouées (3 max → PUK)
}

#[derive(Debug, Clone)]
pub struct PuceStore {
    pub puces: Vec<Puce>,
}

impl PuceStore {
    pub fn load() -> Self {
        match std::fs::read_to_string(crate::data_path("puce.json")) {
            Ok(data) => {
                match from_str(&data) {
                    Ok(JsonValue::Object(map)) => {
                        let puces = match map.get("puces") {
                            Some(JsonValue::Array(arr)) => arr.iter()
                                .filter_map(|v| Puce::from_json(v))
                                .collect(),
                            _ => Vec::new(),
                        };
                        PuceStore { puces }
                    }
                    _ => PuceStore { puces: Vec::new() },
                }
            }
            _ => PuceStore { puces: Vec::new() },
        }
    }

    pub fn save(&self) {
        let arr: Vec<JsonValue> = self.puces.iter().map(|p| p.to_json()).collect();
        let mut map = HashMap::new();
        map.insert("puces".to_string(), JsonValue::Array(arr));
        let _ = std::fs::write(crate::data_path("puce.json"), to_string(&JsonValue::Object(map)));
    }

    /// Hacher un PIN puce avec AfriHash-256 (le sel de la Planète Verte)
    pub fn hash_pin(pin: &str) -> String {
        let h = crate::afri_hash::afrihash_256(format!("planete_verte_salt_{}", pin).as_bytes());
        crate::afri_hex::encode(&h)
    }

    /// Créer une nouvelle puce pour un utilisateur — numéro vert unique par pays
    pub fn creer_puce(&mut self, username: &str, pays: &str, code_pays: &str) -> Puce {
        let count = self.puces.iter()
            .filter(|p| p.code_pays == code_pays)
            .count() as u64;
        let id = format!("PV{}{:06}", code_pays.trim_start_matches('+'), count + 1);
        // Numéro vert : code pays + 76 (préfixe vert) + 6 chiffres
        let numero_vert = format!("{}76{:06}", code_pays, count + 1);
        let puce = Puce {
            id,
            numero_vert,
            pays: pays.to_string(),
            code_pays: code_pays.to_string(),
            username: username.to_string(),
            apn: APN_APN.to_string(),
            etat: "ACTIVE".to_string(),
            active_le: now_timestamp(),
            bloc_activation: 0, // rempli après mine
            pin_hash: String::new(), // PIN choisi à la première utilisation
            puk_hash: String::new(),
            tentative_pin: 0,
        };
        self.puces.push(puce.clone());
        self.save();
        puce
    }

    pub fn par_username(&self, username: &str) -> Option<&Puce> {
        self.puces.iter().find(|p| p.username == username)
    }

    pub fn par_username_mut(&mut self, username: &str) -> Option<&mut Puce> {
        self.puces.iter_mut().find(|p| p.username == username)
    }

    /// Vérifier le PIN puce — 3 tentatives max, ensuite PUK obligatoire
    pub fn verifier_pin(&mut self, username: &str, pin: &str) -> Result<(), String> {
        let puce = self.par_username_mut(username)
            .ok_or("Aucune puce Planète Verte pour cet utilisateur")?;
        if puce.etat == "SUSPENDUE" {
            return Err("Puce suspendue. Contacte l'administrateur du réseau vert.".to_string());
        }
        if puce.tentative_pin >= 3 {
            return Err("Trop de tentatives. Entre ton code PUK.".to_string());
        }
        if PuceStore::hash_pin(pin) == puce.pin_hash {
            puce.tentative_pin = 0;
            Ok(())
        } else {
            puce.tentative_pin += 1;
            Err(format!("Code PIN incorrect (tentative {}/3)", puce.tentative_pin))
        }
    }

    /// Définir le PIN puce (première utilisation ou changement)
    pub fn definir_pin(&mut self, username: &str, pin: &str) -> Result<(), String> {
        if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
            return Err("Le code PIN doit faire exactement 4 chiffres".to_string());
        }
        let puce = self.par_username_mut(username)
            .ok_or("Aucune puce Planète Verte pour cet utilisateur")?;
        puce.pin_hash = PuceStore::hash_pin(pin);
        puce.tentative_pin = 0;
        self.save();
        Ok(())
    }
}

impl Puce {
    pub fn to_json(&self) -> JsonValue {
        let mut map = HashMap::new();
        map.insert("id".to_string(), JsonValue::Str(self.id.clone()));
        map.insert("numero_vert".to_string(), JsonValue::Str(self.numero_vert.clone()));
        map.insert("pays".to_string(), JsonValue::Str(self.pays.clone()));
        map.insert("code_pays".to_string(), JsonValue::Str(self.code_pays.clone()));
        map.insert("username".to_string(), JsonValue::Str(self.username.clone()));
        map.insert("apn".to_string(), JsonValue::Str(self.apn.clone()));
        map.insert("etat".to_string(), JsonValue::Str(self.etat.clone()));
        map.insert("active_le".to_string(), JsonValue::Int(self.active_le));
        map.insert("bloc_activation".to_string(), JsonValue::Int(self.bloc_activation as i64));
        map.insert("pin_hash".to_string(), JsonValue::Str(self.pin_hash.clone()));
        map.insert("puk_hash".to_string(), JsonValue::Str(self.puk_hash.clone()));
        map.insert("tentative_pin".to_string(), JsonValue::Int(self.tentative_pin as i64));
        JsonValue::Object(map)
    }

    pub fn from_json(v: &JsonValue) -> Option<Self> {
        let map = v.as_object()?;
        Some(Puce {
            id: map.get("id")?.as_str()?.to_string(),
            numero_vert: map.get("numero_vert")?.as_str()?.to_string(),
            pays: map.get("pays")?.as_str()?.to_string(),
            code_pays: map.get("code_pays")?.as_str()?.to_string(),
            username: map.get("username")?.as_str()?.to_string(),
            apn: map.get("apn")?.as_str()?.to_string(),
            etat: map.get("etat")?.as_str()?.to_string(),
            active_le: map.get("active_le")?.as_i64()?,
            bloc_activation: map.get("bloc_activation").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
            pin_hash: map.get("pin_hash").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            puk_hash: map.get("puk_hash").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            tentative_pin: map.get("tentative_pin").and_then(|v| v.as_i64()).unwrap_or(0) as u32,
        })
    }
}
