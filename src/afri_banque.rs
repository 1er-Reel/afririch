// ===== AFRI BANQUE — La Banque Internationale des 54 Pays 🏦🌍 =====
// v1.79 — La vision du chef: le soleil mine un bloc pour un pays →
// la valeur (100 AFR par bloc) va à la Grande Banque de ce pays →
// le gouverneur verse l'argent dans les sous-banques des villes →
// le peuple ne se déplace plus. L'argent vient à lui.
//
// PAS DE TRICHE: la valeur de chaque pays est CALCULÉE depuis la vraie
// blockchain (blocs minés pour ce pays × 100 AFR). Les versements sont
// de vraies transactions SYSTEM → VILLE-{ville}, gravées et minées.
// Le reste = valeur minée − valeur déjà versée. Impossible de verser
// plus que le soleil n'a donné. C'est le soleil qui valide. ☀️

use std::collections::HashMap;
use crate::afri_json::{JsonValue, to_string, from_str};
use crate::afri_time::now_timestamp;

/// 1 AES = $3M = 3 AFR (l'or, le fer, l'eau, le pétrole, le diamant)
pub const AFR_PAR_AES: u64 = 3;

/// La récompense de minage par bloc (doit rester synchronisée avec Blockchain::reward)
pub const RECOMPENSE_BLOC_AFR: u64 = 100;

/// Les 3 grandes villes de chaque pays — la sous-banque de chaque ville.
/// Le gouverneur verse ici pour que la population ne se déplace plus.
pub const VILLES_PAYS: &[(&str, &[&str])] = &[
    ("NE", &["Niamey", "Zinder", "Maradi"]),
    ("ML", &["Bamako", "Sikasso", "Ségou"]),
    ("SD", &["Khartoum", "Omdurman", "Port-Soudan"]),
    ("TD", &["N'Djamena", "Moundou", "Sarh"]),
    ("LY", &["Tripoli", "Benghazi", "Misrata"]),
    ("EG", &["Le Caire", "Alexandrie", "Louxor"]),
    ("DZ", &["Alger", "Oran", "Constantine"]),
    ("ER", &["Asmara", "Keren", "Massaoua"]),
    ("DJ", &["Djibouti", "Ali-Sabieh", "Tadjourah"]),
    ("MR", &["Nouakchott", "Nouadhibou", "Kiffa"]),
    ("SO", &["Mogadiscio", "Hargeisa", "Kismaayo"]),
    ("SS", &["Djouba", "Wau", "Malakal"]),
    ("ET", &["Addis-Abeba", "Dire Dawa", "Mekelélé"]),
    ("BF", &["Ouagadougou", "Bobo-Dioulasso", "Koudougou"]),
    ("NA", &["Windhoek", "Walvis Bay", "Swakopmund"]),
    ("BW", &["Gaborone", "Francistown", "Maun"]),
    ("SN", &["Dakar", "Thiès", "Touba"]),
    ("NG", &["Lagos", "Abuja", "Kano"]),
    ("TN", &["Tunis", "Sfax", "Sousse"]),
    ("CV", &["Praia", "Mindelo", "Santa Maria"]),
    ("ZM", &["Lusaka", "Kitwe", "Livingstone"]),
    ("KE", &["Nairobi", "Mombasa", "Kisumu"]),
    ("ZW", &["Harare", "Bulawayo", "Mutare"]),
    ("LS", &["Maseru", "Teyateyaneng", "Mafeteng"]),
    ("MA", &["Casablanca", "Rabat", "Marrakech"]),
    ("AO", &["Luanda", "Huambo", "Lobito"]),
    ("ZA", &["Johannesburg", "Le Cap", "Durban"]),
    ("TZ", &["Dar es Salaam", "Dodoma", "Arusha"]),
    ("MW", &["Lilongwe", "Blantyre", "Mzuzu"]),
    ("MZ", &["Maputo", "Beira", "Nampula"]),
    ("MG", &["Antananarivo", "Toamasina", "Mahajanga"]),
    ("SC", &["Victoria", "Anse Boileau", "Beau Vallon"]),
    ("GM", &["Banjul", "Serekunda", "Brikama"]),
    ("SZ", &["Mbabane", "Manzini", "Nhlangano"]),
    ("CF", &["Bangui", "Bimbo", "Berbérati"]),
    ("UG", &["Kampala", "Gulu", "Entebbe"]),
    ("RW", &["Kigali", "Butare", "Musanze"]),
    ("KM", &["Moroni", "Mutsamudu", "Fomboni"]),
    ("GW", &["Bissau", "Bafatá", "Gabú"]),
    ("CI", &["Abidjan", "Yamoussoukro", "Bouaké"]),
    ("GH", &["Accra", "Kumasi", "Tamale"]),
    ("GN", &["Conakry", "Kankan", "Nzérékoré"]),
    ("TG", &["Lomé", "Sokodé", "Kara"]),
    ("BJ", &["Cotonou", "Porto-Novo", "Parakou"]),
    ("CM", &["Douala", "Yaoundé", "Garoua"]),
    ("CD", &["Kinshasa", "Lubumbashi", "Goma"]),
    ("CG", &["Brazzaville", "Pointe-Noire", "Dolisie"]),
    ("BI", &["Gitega", "Bujumbura", "Ngozi"]),
    ("MU", &["Port-Louis", "Vacoas", "Curepipe"]),
    ("LR", &["Monrovia", "Gbarnga", "Buchanan"]),
    ("SL", &["Freetown", "Bo", "Kenema"]),
    ("GA", &["Libreville", "Port-Gentil", "Franceville"]),
    ("ST", &["São Tomé", "Trindade", "Santana"]),
    ("GQ", &["Malabo", "Bata", "Ebebiyín"]),
];

/// Villes d'un pays (pour la page du pays). Retourne un Vec statique.
pub fn villes_de(code: &str) -> &'static [&'static str] {
    for (c, villes) in VILLES_PAYS {
        if *c == code { return villes; }
    }
    &["Capitale"]
}

/// Un message envoyé au gouverneur d'une Grande Banque.
#[derive(Clone)]
pub struct MessageBanque {
    pub de: String,
    pub texte: String,
    pub date: i64,
}

/// L'état persistant d'une Grande Banque de pays.
#[derive(Clone)]
pub struct BanquePays {
    pub code: String,        // "ML", "NE"...
    pub gouverneur: String,  // username du gouverneur ("" = pas encore nommé)
    pub messages: Vec<MessageBanque>,
}

impl BanquePays {
    pub fn to_json(&self) -> JsonValue {
        let mut m = HashMap::new();
        m.insert("code".to_string(), JsonValue::Str(self.code.clone()));
        m.insert("gouverneur".to_string(), JsonValue::Str(self.gouverneur.clone()));
        m.insert("messages".to_string(), JsonValue::Array(
            self.messages.iter().map(|msg| {
                let mut mm = HashMap::new();
                mm.insert("de".to_string(), JsonValue::Str(msg.de.clone()));
                mm.insert("texte".to_string(), JsonValue::Str(msg.texte.clone()));
                mm.insert("date".to_string(), JsonValue::UInt(msg.date as u64));
                JsonValue::Object(mm)
            }).collect()
        ));
        JsonValue::Object(m)
    }

    pub fn from_json(v: &JsonValue) -> Option<BanquePays> {
        let map = v.as_object()?;
        Some(BanquePays {
            code: map.get("code")?.as_str()?.to_string(),
            gouverneur: map.get("gouverneur").and_then(|g| g.as_str()).unwrap_or("").to_string(),
            messages: match map.get("messages") {
                Some(JsonValue::Array(arr)) => arr.iter().filter_map(|mv| {
                    let mm = mv.as_object()?;
                    Some(MessageBanque {
                        de: mm.get("de").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                        texte: mm.get("texte").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                        date: mm.get("date").and_then(|x| x.as_u64()).unwrap_or(0) as i64,
                    })
                }).collect(),
                _ => Vec::new(),
            },
        })
    }
}

/// Le magasin de toutes les grandes banques — persisté dans banque.json.
pub struct BanqueStore {
    pub pays: HashMap<String, BanquePays>, // code → banque
}

impl BanqueStore {
    fn chemin_data() -> String {
        crate::data_path("banque.json")
    }

    pub fn charger() -> Self {
        let mut store = BanqueStore { pays: HashMap::new() };
        if let Ok(s) = std::fs::read_to_string(Self::chemin_data()) {
            if let Ok(JsonValue::Object(map)) = from_str(&s) {
                if let Some(JsonValue::Array(arr)) = map.get("pays") {
                    for v in arr {
                        if let Some(b) = BanquePays::from_json(v) {
                            store.pays.insert(b.code.clone(), b);
                        }
                    }
                }
            }
        }
        store
    }

    pub fn sauvegarder(&self) {
        let mut m = HashMap::new();
        m.insert("pays".to_string(), JsonValue::Array(
            self.pays.values().map(|b| b.to_json()).collect()
        ));
        let _ = std::fs::write(Self::chemin_data(), to_string(&JsonValue::Object(m)));
    }

    /// La Grande Banque d'un pays — créée vide si elle n'existe pas encore.
    pub fn banque(&mut self, code: &str) -> &mut BanquePays {
        self.pays.entry(code.to_string()).or_insert_with(|| BanquePays {
            code: code.to_string(),
            gouverneur: String::new(),
            messages: Vec::new(),
        })
    }

    /// Nommer un gouverneur (admin ou gouverneur sortant).
    pub fn nommer_gouverneur(&mut self, code: &str, username: &str) {
        self.banque(code).gouverneur = username.to_string();
        self.sauvegarder();
    }

    /// Envoyer un message au gouverneur d'un pays.
    pub fn envoyer_message(&mut self, code: &str, de: &str, texte: &str) {
        let b = self.banque(code);
        b.messages.push(MessageBanque {
            de: de.to_string(),
            texte: texte.to_string(),
            date: now_timestamp() as i64,
        });
        // Garde les 30 derniers messages
        if b.messages.len() > 30 {
            let drop = b.messages.len() - 30;
            b.messages.drain(0..drop);
        }
        self.sauvegarder();
    }
}

/// Formatage: 1234567 → "1 234 567"
pub fn format_nombre(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    let bytes = s.as_bytes();
    for (i, c) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 { out.push(' '); }
        out.push(*c as char);
    }
    out
}
