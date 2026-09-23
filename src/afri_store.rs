// ===== AFRI STORE — v1.90: LE PLAY STORE DE L'AFRIQUE 🏪📱 =====
// Pas de Google. Pas de commission 30%. Pas de compte bancaire à Mountain View.
// Le développeur africain publie son app, le continent l'installe, les AFR vont
// DIRECTEMENT dans sa poche. 0% de commission — l'Afrique nourrit l'Afrique. 💚

use std::collections::HashMap;
use crate::afri_json::{JsonValue, to_string, from_str};
use crate::afri_cerveau::normaliser;

/// Une application du store — l'œuvre d'un développeur africain 📱
#[derive(Debug, Clone)]
pub struct AppAfri {
    pub id: String,             // "APP-1"
    pub nom: String,            // "Le Lion du Sahel"
    pub description: String,    // courte description
    pub auteur: String,         // username du développeur
    pub emoji: String,          // icône emoji
    pub prix: i64,              // 0 = gratuit
    pub categorie: String,     // Jeux, Réseaux, Outils, Éducation, Banque, Santé...
    pub version: String,        // "1.0"
    pub installs: u64,          // nombre d'installations
    pub note_total: i64,        // somme des étoiles (1-5)
    pub note_count: i64,        // nombre de notes
    pub media: String,          // icône/screenshot (fichier dans store_media/)
    pub media_ext: String,      // extension du média
    pub heure: i64,             // date de publication
}

/// Une note d'un utilisateur ⭐
#[derive(Debug, Clone)]
pub struct NoteApp {
    pub app: String,            // id de l'app
    pub de: String,             // username
    pub etoiles: i64,           // 1 à 5
}

/// Une installation : qui a installé quelle app (v2.28 — MES APPS)
#[derive(Debug, Clone)]
pub struct InstallAfri {
    pub app: String,        // id de l'app
    pub de: String,         // username
    pub heure: i64,
}

/// Le Play Store africain — persisté dans store.json
#[derive(Debug, Clone)]
pub struct StoreAfri {
    pub apps: Vec<AppAfri>,
    pub notes: Vec<NoteApp>,
    pub installs_de: Vec<InstallAfri>,  // v2.28 : qui a installé quoi
    pub prochain_id: u64,
    pub chemin: String,
}

/// Les catégories du store africain 🗂️
pub const CATEGORIES: [(&str, &str); 8] = [
    ("Jeux", "🎮"),
    ("Réseaux", "💬"),
    ("Outils", "🔧"),
    ("Éducation", "📚"),
    ("Banque", "🏦"),
    ("Santé", "🏥"),
    ("Culture", "🎨"),
    ("Autre", "📦"),
];

impl StoreAfri {
    pub fn nouveau() -> Self {
        let chemin = crate::data_path("store.json");
        match std::fs::read_to_string(&chemin) {
            Ok(data) => {
                let v = from_str(&data).unwrap_or(JsonValue::Object(HashMap::new()));
                StoreAfri::from_json(&v)
            }
            Err(_) => StoreAfri { apps: Vec::new(), notes: Vec::new(), installs_de: Vec::new(), prochain_id: 1, chemin },
        }
    }

    pub fn sauvegarder(&self) {
        let _ = std::fs::write(&self.chemin, to_string(&self.to_json()));
    }

    /// Publier une app — le développeur africain entre dans le store 🚀
    pub fn publier(&mut self, nom: &str, description: &str, auteur: &str, emoji: &str, prix: i64, categorie: &str, media: &str, media_ext: &str, heure: i64) -> String {
        let id = format!("APP-{}", self.prochain_id);
        self.prochain_id += 1;
        let app = AppAfri {
            id: id.clone(),
            nom: nom.to_string(),
            description: description.to_string(),
            auteur: auteur.to_string(),
            emoji: emoji.to_string(),
            prix,
            categorie: categorie.to_string(),
            version: "1.0".to_string(),
            installs: 0,
            note_total: 0,
            note_count: 0,
            media: media.to_string(),
            media_ext: media_ext.to_string(),
            heure,
        };
        self.apps.push(app);
        id
    }

    /// Trouver une app par id 🔍
    pub fn trouver(&self, id: &str) -> Option<&AppAfri> {
        self.apps.iter().find(|a| a.id == id)
    }

    /// Une installation de plus 📲
    /// v2.28 — les apps installées par un utilisateur (pour sa page MES APPS)
    pub fn apps_installees(&self, username: &str) -> Vec<&AppAfri> {
        self.installs_de.iter()
            .filter(|i| i.de == username)
            .filter_map(|i| self.apps.iter().find(|a| a.id == i.app))
            .collect()
    }

    /// v2.28 — installer ET enregistrer qui installe
    pub fn installer_de(&mut self, id: &str, username: &str) -> bool {
        let deja = self.installs_de.iter().any(|i| i.app == id && i.de == username);
        if deja { return true; } // déjà installée — pas de doublon
        self.installs_de.push(InstallAfri { app: id.to_string(), de: username.to_string(), heure: crate::now_timestamp() });
        self.installer(id)
    }

    pub fn installer(&mut self, id: &str) -> bool {
        match self.apps.iter_mut().find(|a| a.id == id) {
            Some(a) => { a.installs += 1; true }
            None => false,
        }
    }

    /// Noter une app — une seule note par personne et par app ⭐
    /// Retourne Some(ancienne_note) si mise à jour, None si nouvelle note.
    pub fn noter(&mut self, app: &str, de: &str, etoiles: i64) -> Option<i64> {
        if let Some(n) = self.notes.iter_mut().find(|n| n.app == app && n.de == de) {
            let ancienne = n.etoiles;
            n.etoiles = etoiles;
            if let Some(a) = self.apps.iter_mut().find(|a| a.id == app) {
                a.note_total += etoiles - ancienne;
            }
            Some(ancienne)
        } else {
            self.notes.push(NoteApp { app: app.to_string(), de: de.to_string(), etoiles });
            if let Some(a) = self.apps.iter_mut().find(|a| a.id == app) {
                a.note_total += etoiles;
                a.note_count += 1;
            }
            None
        }
    }

    /// La note d'un utilisateur pour une app
    pub fn ma_note(&self, app: &str, de: &str) -> Option<i64> {
        self.notes.iter().find(|n| n.app == app && n.de == de).map(|n| n.etoiles)
    }

    /// L'auteur d'une app peut-il la modifier ? (seul l'auteur)
    pub fn est_auteur(&self, id: &str, username: &str) -> bool {
        self.apps.iter().any(|a| a.id == id && a.auteur == username)
    }

    /// Supprimer une app (l'auteur uniquement)
    pub fn supprimer(&mut self, id: &str) -> bool {
        let avant = self.apps.len();
        self.apps.retain(|a| a.id != id);
        self.notes.retain(|n| n.app != id);
        self.apps.len() < avant
    }

    /// Les apps d'un développeur 🧑🏾‍💻
    pub fn apps_de(&self, username: &str) -> Vec<&AppAfri> {
        self.apps.iter().filter(|a| a.auteur == username).collect()
    }

    /// v1.92: Mettre à jour son app — version, description, emoji, prix (l'auteur uniquement) 🔄
    /// Retourne false si l'app n'existe pas.
    pub fn maj(&mut self, id: &str, auteur: &str, description: &str, emoji: &str, prix: i64, version: &str) -> bool {
        match self.apps.iter_mut().find(|a| a.id == id && a.auteur == auteur) {
            Some(a) => {
                if !description.is_empty() { a.description = description.to_string(); }
                if !emoji.is_empty() { a.emoji = emoji.to_string(); }
                if prix >= 0 { a.prix = prix; }
                if !version.is_empty() { a.version = version.to_string(); }
                true
            }
            None => false,
        }
    }

    /// v1.92: Rechercher des apps 🔍 — nom + description + auteur, sans accents ni casse
    pub fn rechercher(&self, q: &str) -> Vec<&AppAfri> {
        let nq = normaliser(q);
        if nq.is_empty() { return Vec::new(); }
        self.apps.iter().filter(|a| {
            normaliser(&a.nom).contains(&nq)
                || normaliser(&a.description).contains(&nq)
                || normaliser(&a.auteur).contains(&nq)
                || normaliser(&a.categorie).contains(&nq)
        }).collect()
    }

    /// Où vivent les médias du store 📸
    pub fn chemin_media() -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/afririch/store_media", home)
    }

    /// Sauver l'icône d'une app (base64 + extension) — max 1 Mo 🖼️
    pub fn sauver_media(b64: &str, ext: &str) -> Result<String, String> {
        let ext = ext.to_lowercase();
        if !["jpg", "jpeg", "png", "gif", "webp"].contains(&ext.as_str()) {
            return Err(format!("format .{} non autorisé (jpg, png, gif, webp)", ext));
        }
        let bytes = crate::afri_plante::PlanteStore::base64_decode(b64).ok_or("média illisible".to_string())?;
        if bytes.is_empty() { return Err("média vide".to_string()); }
        if bytes.len() > 1024 * 1024 { return Err("trop lourd (max 1 Mo)".to_string()); }
        let dir = Self::chemin_media();
        std::fs::create_dir_all(&dir).map_err(|_| "erreur disque".to_string())?;
        let h = crate::afri_hash::afrihash_256(&bytes);
        let nom = format!("{}_{}.{}", crate::afri_time::now_timestamp_millis(), &crate::afri_hex::encode(&h[..4]), ext);
        std::fs::write(format!("{}/{}", dir, nom), &bytes).map_err(|_| "erreur écriture".to_string())?;
        Ok(nom)
    }

    // ===== PERSISTANCE =====

    pub fn to_json(&self) -> JsonValue {
        let mut obj = HashMap::new();
        obj.insert("prochain_id".to_string(), JsonValue::Int(self.prochain_id as i64));
        obj.insert("apps".to_string(), JsonValue::Array(self.apps.iter().map(|a| {
            let mut o = HashMap::new();
            o.insert("id".to_string(), JsonValue::Str(a.id.clone()));
            o.insert("nom".to_string(), JsonValue::Str(a.nom.clone()));
            o.insert("description".to_string(), JsonValue::Str(a.description.clone()));
            o.insert("auteur".to_string(), JsonValue::Str(a.auteur.clone()));
            o.insert("emoji".to_string(), JsonValue::Str(a.emoji.clone()));
            o.insert("prix".to_string(), JsonValue::Int(a.prix));
            o.insert("categorie".to_string(), JsonValue::Str(a.categorie.clone()));
            o.insert("version".to_string(), JsonValue::Str(a.version.clone()));
            o.insert("installs".to_string(), JsonValue::Int(a.installs as i64));
            o.insert("note_total".to_string(), JsonValue::Int(a.note_total));
            o.insert("note_count".to_string(), JsonValue::Int(a.note_count));
            o.insert("media".to_string(), JsonValue::Str(a.media.clone()));
            o.insert("media_ext".to_string(), JsonValue::Str(a.media_ext.clone()));
            o.insert("heure".to_string(), JsonValue::Int(a.heure));
            JsonValue::Object(o)
        }).collect()));
        obj.insert("notes".to_string(), JsonValue::Array(self.notes.iter().map(|n| {
            let mut o = HashMap::new();
            o.insert("app".to_string(), JsonValue::Str(n.app.clone()));
            o.insert("de".to_string(), JsonValue::Str(n.de.clone()));
            o.insert("etoiles".to_string(), JsonValue::Int(n.etoiles));
            JsonValue::Object(o)
        }).collect()));
        JsonValue::Object(obj)
    }

    pub fn from_json(v: &JsonValue) -> Self {
        let mut store = StoreAfri { apps: Vec::new(), notes: Vec::new(), installs_de: Vec::new(), prochain_id: 1, chemin: crate::data_path("store.json") };
        if let JsonValue::Object(obj) = v {
            if let Some(JsonValue::Int(i)) = obj.get("prochain_id") { store.prochain_id = *i as u64; }
            if let Some(JsonValue::Array(arr)) = obj.get("apps") {
                for a in arr {
                    // Anti-collision: prochain_id doit toujours dépasser le plus grand ID existant
                    if let JsonValue::Object(o) = a {
                        if let Some(JsonValue::Str(sid)) = o.get("id") {
                            if let Some(n) = sid.trim_start_matches("APP-").parse::<u64>().ok() {
                                if n >= store.prochain_id { store.prochain_id = n + 1; }
                            }
                        }
                    }
                    if let JsonValue::Object(o) = a {
                        let g = |k: &str| o.get(k).and_then(|x| match x { JsonValue::Str(s) => Some(s.clone()), _ => None }).unwrap_or_default();
                        let gi = |k: &str| o.get(k).and_then(|x| x.as_i64()).unwrap_or(0);
                        store.apps.push(AppAfri {
                            id: g("id"),
                            nom: g("nom"),
                            description: g("description"),
                            auteur: g("auteur"),
                            emoji: g("emoji"),
                            prix: gi("prix"),
                            categorie: g("categorie"),
                            version: g("version"),
                            installs: gi("installs") as u64,
                            note_total: gi("note_total"),
                            note_count: gi("note_count"),
                            media: g("media"),
                            media_ext: g("media_ext"),
                            heure: gi("heure"),
                        });
                    }
                }
            }
            if let Some(JsonValue::Array(arr)) = obj.get("notes") {
                for n in arr {
                    if let JsonValue::Object(o) = n {
                        let g = |k: &str| o.get(k).and_then(|x| match x { JsonValue::Str(s) => Some(s.clone()), _ => None }).unwrap_or_default();
                        let gi = |k: &str| o.get(k).and_then(|x| x.as_i64()).unwrap_or(0);
                        store.notes.push(NoteApp { app: g("app"), de: g("de"), etoiles: gi("etoiles") });
                    }
                }
            }
            // v2.28 — les installations : qui a installé quoi
            if let Some(JsonValue::Array(arr)) = obj.get("installs_de") {
                for n in arr {
                    if let JsonValue::Object(o) = n {
                        let g = |k: &str| o.get(k).and_then(|x| match x { JsonValue::Str(s) => Some(s.clone()), _ => None }).unwrap_or_default();
                        let gi = |k: &str| o.get(k).and_then(|x| x.as_i64()).unwrap_or(0);
                        store.installs_de.push(InstallAfri { app: g("app"), de: g("de"), heure: gi("heure") });
                    }
                }
            }
        }
        store
    }
}
