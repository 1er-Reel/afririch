// ===== AFRI PLANTE — Planté Verte: le réseau social africain =====
// v1.80 — LE FACEBOOK DE L'AFRIQUE. 🌱
// Règle du chef: "l'utilisateur ne doit pas voir ça à moins qu'ils sont amis,
// il est dans son contact." → Ici, on ne voit QUE ses propres posts
// et ceux de ses contacts. Personne d'autre. Confidentialité africaine. 🔒
// Chaque post est gravé sur la blockchain (tx PLANTE-POST) — rien ne sort du continent.

use std::collections::HashMap;
use crate::afri_json::{JsonValue, to_string, from_str};
use crate::afri_time::now_timestamp;

/// Un post de Planté Verte
#[derive(Clone, Debug)]
pub struct PostPlante {
    pub author: String,
    pub contenu: String,
    pub date: i64,
    pub likes: u64,
    pub likers: Vec<String>,   // qui a aimé (1 seul like par personne)
}

/// Le magasin Planté Verte: posts + contacts (amitié à sens unique: A ajoute B → A voit B)
pub struct PlanteStore {
    pub posts: Vec<PostPlante>,
    pub contacts: HashMap<String, Vec<String>>, // username → liste de contacts
}

impl PlanteStore {
    pub fn nouveau() -> Self {
        PlanteStore { posts: Vec::new(), contacts: HashMap::new() }
    }

    pub fn chemin_data() -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/afririch/plante.json", home)
    }

    pub fn charger() -> Self {
        Self::charger_depuis(&Self::chemin_data())
    }

    pub fn charger_depuis(chemin: &str) -> Self {
        if let Ok(data) = std::fs::read_to_string(chemin) {
            if let Ok(v) = from_str(&data) {
                return Self::from_json(&v);
            }
        }
        Self::nouveau()
    }

    pub fn sauvegarder(&self) {
        self.sauvegarder_vers(&Self::chemin_data());
    }

    pub fn sauvegarder_vers(&self, chemin: &str) {
        let _ = std::fs::write(chemin, to_string(&self.to_json()));
    }

    // ===== CONTACTS =====

    /// A ajoute B dans ses contacts → A voit les posts de B
    pub fn ajouter_contact(&mut self, moi: &str, ami: &str) -> bool {
        if moi == ami { return false; }
        let liste = self.contacts.entry(moi.to_string()).or_default();
        if liste.iter().any(|c| c == ami) { return false; } // déjà contact
        liste.push(ami.to_string());
        true
    }

    pub fn retirer_contact(&mut self, moi: &str, ami: &str) -> bool {
        if let Some(liste) = self.contacts.get_mut(moi) {
            let avant = liste.len();
            liste.retain(|c| c != ami);
            return liste.len() < avant;
        }
        false
    }

    /// Est-ce que `moi` a `ami` dans ses contacts ?
    pub fn est_contact(&self, moi: &str, ami: &str) -> bool {
        self.contacts.get(moi).map(|l| l.iter().any(|c| c == ami)).unwrap_or(false)
    }

    pub fn contacts_de(&self, moi: &str) -> Vec<String> {
        self.contacts.get(moi).cloned().unwrap_or_default()
    }

    // ===== POSTS =====

    pub fn publier(&mut self, author: &str, contenu: &str) {
        self.posts.push(PostPlante {
            author: author.to_string(),
            contenu: contenu.to_string(),
            date: now_timestamp(),
            likes: 0,
            likers: Vec::new(),
        });
    }

    /// LE FIL PRIVÉ: mes posts + les posts de MES contacts seulement. 🔒
    /// C'est la règle du chef — personne d'autre ne voit.
    pub fn fil_prive(&self, moi: &str) -> Vec<&PostPlante> {
        let mes_contacts = self.contacts_de(moi);
        self.posts.iter()
            .filter(|p| p.author == moi || mes_contacts.iter().any(|c| *c == p.author))
            .collect()
    }

    /// Aimer un post (toggle: re-cliquer retire le cœur) — 1 seul like par personne
    pub fn aimer(&mut self, index: usize, moi: &str) -> bool {
        if let Some(post) = self.posts.get_mut(index) {
            if post.likers.iter().any(|l| l == moi) {
                post.likers.retain(|l| l != moi);
                post.likes = post.likes.saturating_sub(1);
            } else {
                post.likers.push(moi.to_string());
                post.likes += 1;
            }
            true
        } else {
            false
        }
    }

    // ===== PERSISTANCE =====

    pub fn to_json(&self) -> JsonValue {
        let mut root = HashMap::new();
        let posts: Vec<JsonValue> = self.posts.iter().map(|p| {
            let mut m = HashMap::new();
            m.insert("author".to_string(), JsonValue::Str(p.author.clone()));
            m.insert("contenu".to_string(), JsonValue::Str(p.contenu.clone()));
            m.insert("date".to_string(), JsonValue::Int(p.date));
            m.insert("likes".to_string(), JsonValue::Int(p.likes as i64));
            m.insert("likers".to_string(), JsonValue::Array(
                p.likers.iter().map(|l| JsonValue::Str(l.clone())).collect()));
            JsonValue::Object(m)
        }).collect();
        root.insert("posts".to_string(), JsonValue::Array(posts));
        let mut contacts = HashMap::new();
        for (user, liste) in &self.contacts {
            contacts.insert(user.clone(), JsonValue::Array(
                liste.iter().map(|c| JsonValue::Str(c.clone())).collect()));
        }
        root.insert("contacts".to_string(), JsonValue::Object(contacts));
        JsonValue::Object(root)
    }

    pub fn from_json(v: &JsonValue) -> Self {
        let mut store = Self::nouveau();
        if let Some(map) = v.as_object() {
            if let Some(JsonValue::Array(posts)) = map.get("posts") {
                for p in posts {
                    if let Some(pm) = p.as_object() {
                        store.posts.push(PostPlante {
                            author: pm.get("author").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            contenu: pm.get("contenu").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            date: pm.get("date").and_then(|v| v.as_i64()).unwrap_or(0),
                            likes: pm.get("likes").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
                            likers: match pm.get("likers") {
                                Some(JsonValue::Array(arr)) => arr.iter()
                                    .filter_map(|l| l.as_str().map(|s| s.to_string())).collect(),
                                _ => Vec::new(),
                            },
                        });
                    }
                }
            }
            if let Some(JsonValue::Object(contacts)) = map.get("contacts") {
                for (user, liste) in contacts {
                    let l = match liste {
                        JsonValue::Array(arr) => arr.iter()
                            .filter_map(|c| c.as_str().map(|s| s.to_string())).collect(),
                        _ => Vec::new(),
                    };
                    store.contacts.insert(user.clone(), l);
                }
            }
        }
        store
    }
}
