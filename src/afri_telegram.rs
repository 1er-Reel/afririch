// ===== AFRI TÉLÉGRAM — v1.85: le Telegram africain, RÉEL 📢 =====
// Des vrais canaux : n'importe qui peut créer son canal, les Africains s'abonnent,
// le créateur publie des annonces — et chaque annonce est GRAVÉE sur la blockchain.
// Pas de serveurs à Dubaï. Pas de fondateur russe. Le broadcast de l'Afrique. 💚

use std::collections::HashMap;
use crate::afri_json::{JsonValue, to_string, from_str};

/// Une annonce de canal — un message que tout le canal entend 📢
#[derive(Debug, Clone)]
pub struct MsgTelegram {
    pub de: String,       // username du créateur qui publie
    pub texte: String,
    pub heure: i64,
    pub commentaires: Vec<CommentTelegram>, // v1.86 : les abonnés peuvent répondre 🗣️
}

/// Un commentaire sous une annonce — la voix des abonnés 🗣️
#[derive(Debug, Clone)]
pub struct CommentTelegram {
    pub de: String,
    pub texte: String,
    pub heure: i64,
    pub likers: Vec<String>,
    pub reponses: Vec<RepTelegram>, // réponses imbriquées (1 niveau)
}

/// Une réponse à un commentaire ↩️
#[derive(Debug, Clone)]
pub struct RepTelegram {
    pub de: String,
    pub texte: String,
    pub heure: i64,
    pub likers: Vec<String>,
}

/// Un canal — la voix d'un Africain que des milliers peuvent suivre 📡
#[derive(Debug, Clone)]
pub struct CanalTelegram {
    pub id: u64,
    pub nom: String,
    pub description: String,
    pub createur: String,          // seul le créateur peut publier
    pub abonnes: Vec<String>,    // usernames abonnés
    pub messages: Vec<MsgTelegram>,
}

/// Le magasin Afri Télégram — persisté dans telegram.json
#[derive(Debug, Clone)]
pub struct TelegramStore {
    pub canaux: Vec<CanalTelegram>,
    pub prochain_id: u64,
    pub chemin: String,
}

impl TelegramStore {
    pub fn nouveau() -> Self {
        let chemin = crate::data_path("telegram.json");
        match std::fs::read_to_string(&chemin) {
            Ok(data) => {
                let v = from_str(&data).unwrap_or(JsonValue::Object(HashMap::new()));
                TelegramStore::from_json(&v)
            }
            Err(_) => TelegramStore { canaux: Vec::new(), prochain_id: 1, chemin },
        }
    }

    pub fn sauvegarder(&self) {
        let _ = std::fs::write(&self.chemin, to_string(&self.to_json()));
    }

    /// Créer un canal — la voix naît 📡
    pub fn creer_canal(&mut self, nom: &str, description: &str, createur: &str) -> Result<CanalTelegram, String> {
        let nom = nom.trim();
        if nom.is_empty() || nom.len() > 60 {
            return Err("Nom du canal vide ou trop long (max 60)".into());
        }
        if description.len() > 200 {
            return Err("Description trop longue (max 200)".into());
        }
        if self.canaux.iter().any(|c| c.nom.eq_ignore_ascii_case(nom)) {
            return Err("Ce nom de canal existe déjà".into());
        }
        // Un Africain ne crée pas 100 canaux — max 5 par créateur
        let nb = self.canaux.iter().filter(|c| c.createur == createur).count();
        if nb >= 5 {
            return Err("Maximum 5 canaux par créateur".into());
        }
        let canal = CanalTelegram {
            id: self.prochain_id,
            nom: nom.to_string(),
            description: description.trim().to_string(),
            createur: createur.to_string(),
            abonnes: vec![createur.to_string()], // le créateur est abonné d'office
            messages: Vec::new(),
        };
        self.prochain_id += 1;
        self.canaux.push(canal.clone());
        self.sauvegarder();
        Ok(canal)
    }

    /// Trouver un canal par id
    pub fn canal(&self, id: u64) -> Option<&CanalTelegram> {
        self.canaux.iter().find(|c| c.id == id)
    }

    /// S'abonner à un canal
    pub fn abonner(&mut self, id: u64, username: &str) -> Result<(), String> {
        let canal = self.canaux.iter_mut().find(|c| c.id == id)
            .ok_or_else(|| "Canal introuvable".to_string())?;
        if canal.abonnes.iter().any(|a| a == username) {
            return Err("Tu es déjà abonné".into());
        }
        canal.abonnes.push(username.to_string());
        self.sauvegarder();
        Ok(())
    }

    /// Se désabonner
    pub fn desabonner(&mut self, id: u64, username: &str) -> Result<(), String> {
        let canal = self.canaux.iter_mut().find(|c| c.id == id)
            .ok_or_else(|| "Canal introuvable".to_string())?;
        if canal.createur == username {
            return Err("Le créateur ne peut pas quitter son canal".into());
        }
        canal.abonnes.retain(|a| a != username);
        self.sauvegarder();
        Ok(())
    }

    /// Publier une annonce — SEUL le créateur peut publier, comme un vrai canal Telegram
    pub fn publier_annonce(&mut self, id: u64, createur: &str, texte: &str, heure: i64) -> Result<MsgTelegram, String> {
        let texte = texte.trim();
        if texte.is_empty() || texte.len() > 500 {
            return Err("Annonce vide ou trop longue (max 500)".into());
        }
        let canal = self.canaux.iter_mut().find(|c| c.id == id)
            .ok_or_else(|| "Canal introuvable".to_string())?;
        if canal.createur != createur {
            return Err("Seul le créateur du canal peut publier des annonces".into());
        }
        let msg = MsgTelegram {
            de: createur.to_string(),
            texte: texte.to_string(),
            heure,
            commentaires: Vec::new(),
        };
        canal.messages.push(msg.clone());
        // Garde les 200 dernières annonces par canal
        if canal.messages.len() > 200 {
            let excédent = canal.messages.len() - 200;
            canal.messages.drain(0..excédent);
        }
        self.sauvegarder();
        Ok(msg)
    }

    /// Les canaux auxquels un utilisateur est abonné
    pub fn abonnements_de(&self, username: &str) -> Vec<&CanalTelegram> {
        self.canaux.iter().filter(|c| c.abonnes.iter().any(|a| a == username)).collect()
    }

    /// v1.86 : Commenter une annonce — TOUT abonné peut commenter 🗣️
    pub fn commenter_annonce(&mut self, id: u64, msg_index: usize, username: &str, texte: &str, heure: i64) -> Result<(), String> {
        let texte = texte.trim();
        if texte.is_empty() || texte.len() > 500 {
            return Err("Commentaire vide ou trop long (max 500)".into());
        }
        let canal = self.canaux.iter_mut().find(|c| c.id == id)
            .ok_or_else(|| "Canal introuvable".to_string())?;
        if !canal.abonnes.iter().any(|a| a == username) {
            return Err("Abonne-toi pour commenter".into());
        }
        let msg = canal.messages.get_mut(msg_index)
            .ok_or_else(|| "Annonce introuvable".to_string())?;
        let c = CommentTelegram {
            de: username.to_string(),
            texte: texte.to_string(),
            heure,
            likers: Vec::new(),
            reponses: Vec::new(),
        };
        msg.commentaires.push(c);
        self.sauvegarder();
        Ok(())
    }

    /// v1.86 : Répondre à un commentaire ↩️
    pub fn repondre_comment(&mut self, id: u64, msg_index: usize, c_index: usize, username: &str, texte: &str, heure: i64) -> Result<(), String> {
        let texte = texte.trim();
        if texte.is_empty() || texte.len() > 500 {
            return Err("Réponse vide ou trop longue (max 500)".into());
        }
        let canal = self.canaux.iter_mut().find(|c| c.id == id)
            .ok_or_else(|| "Canal introuvable".to_string())?;
        if !canal.abonnes.iter().any(|a| a == username) {
            return Err("Abonne-toi pour répondre".into());
        }
        let msg = canal.messages.get_mut(msg_index)
            .ok_or_else(|| "Annonce introuvable".to_string())?;
        let com = msg.commentaires.get_mut(c_index)
            .ok_or_else(|| "Commentaire introuvable".to_string())?;
        com.reponses.push(RepTelegram {
            de: username.to_string(),
            texte: texte.to_string(),
            heure,
            likers: Vec::new(),
        });
        self.sauvegarder();
        Ok(())
    }

    /// v1.86 : Aimer un commentaire ❤️ (toggle, 1 like par personne)
    pub fn aimer_comment(&mut self, id: u64, msg_index: usize, c_index: usize, username: &str) -> Result<bool, String> {
        let canal = self.canaux.iter_mut().find(|c| c.id == id)
            .ok_or_else(|| "Canal introuvable".to_string())?;
        if !canal.abonnes.iter().any(|a| a == username) {
            return Err("Abonne-toi pour aimer".into());
        }
        let msg = canal.messages.get_mut(msg_index)
            .ok_or_else(|| "Annonce introuvable".to_string())?;
        let com = msg.commentaires.get_mut(c_index)
            .ok_or_else(|| "Commentaire introuvable".to_string())?;
        if let Some(pos) = com.likers.iter().position(|u| u == username) {
            com.likers.remove(pos);
            self.sauvegarder();
            Ok(false)
        } else {
            com.likers.push(username.to_string());
            self.sauvegarder();
            Ok(true)
        }
    }

    /// v1.86 : Aimer une réponse ❤️ (toggle)
    pub fn aimer_rep(&mut self, id: u64, msg_index: usize, c_index: usize, r_index: usize, username: &str) -> Result<bool, String> {
        let canal = self.canaux.iter_mut().find(|c| c.id == id)
            .ok_or_else(|| "Canal introuvable".to_string())?;
        if !canal.abonnes.iter().any(|a| a == username) {
            return Err("Abonne-toi pour aimer".into());
        }
        let msg = canal.messages.get_mut(msg_index)
            .ok_or_else(|| "Annonce introuvable".to_string())?;
        let com = msg.commentaires.get_mut(c_index)
            .ok_or_else(|| "Commentaire introuvable".to_string())?;
        let rep = com.reponses.get_mut(r_index)
            .ok_or_else(|| "Réponse introuvable".to_string())?;
        if let Some(pos) = rep.likers.iter().position(|u| u == username) {
            rep.likers.remove(pos);
            self.sauvegarder();
            Ok(false)
        } else {
            rep.likers.push(username.to_string());
            self.sauvegarder();
            Ok(true)
        }
    }

    /// Tous les canaux, les plus abonnés d'abord
    pub fn canaux_populaires(&self) -> Vec<&CanalTelegram> {
        let mut v: Vec<&CanalTelegram> = self.canaux.iter().collect();
        v.sort_by(|a, b| b.abonnes.len().cmp(&a.abonnes.len()));
        v
    }

    pub fn to_json(&self) -> JsonValue {
        let mut arr = Vec::new();
        for c in &self.canaux {
            let mut msgs = Vec::new();
            for m in &c.messages {
                let mut mo = HashMap::new();
                mo.insert("de".to_string(), JsonValue::Str(m.de.clone()));
                mo.insert("texte".to_string(), JsonValue::Str(m.texte.clone()));
                mo.insert("heure".to_string(), JsonValue::Int(m.heure));
                let mut cmts = Vec::new();
                for cm in &m.commentaires {
                    let mut co = HashMap::new();
                    co.insert("de".to_string(), JsonValue::Str(cm.de.clone()));
                    co.insert("texte".to_string(), JsonValue::Str(cm.texte.clone()));
                    co.insert("heure".to_string(), JsonValue::Int(cm.heure));
                    co.insert("likers".to_string(), JsonValue::Array(
                        cm.likers.iter().map(|u| JsonValue::Str(u.clone())).collect()));
                    let mut reps = Vec::new();
                    for r in &cm.reponses {
                        let mut ro = HashMap::new();
                        ro.insert("de".to_string(), JsonValue::Str(r.de.clone()));
                        ro.insert("texte".to_string(), JsonValue::Str(r.texte.clone()));
                        ro.insert("heure".to_string(), JsonValue::Int(r.heure));
                        ro.insert("likers".to_string(), JsonValue::Array(
                            r.likers.iter().map(|u| JsonValue::Str(u.clone())).collect()));
                        reps.push(JsonValue::Object(ro));
                    }
                    co.insert("reponses".to_string(), JsonValue::Array(reps));
                    cmts.push(JsonValue::Object(co));
                }
                mo.insert("commentaires".to_string(), JsonValue::Array(cmts));
                msgs.push(JsonValue::Object(mo));
            }
            let mut o = HashMap::new();
            o.insert("id".to_string(), JsonValue::Int(c.id as i64));
            o.insert("nom".to_string(), JsonValue::Str(c.nom.clone()));
            o.insert("description".to_string(), JsonValue::Str(c.description.clone()));
            o.insert("createur".to_string(), JsonValue::Str(c.createur.clone()));
            o.insert("abonnes".to_string(), JsonValue::Array(
                c.abonnes.iter().map(|a| JsonValue::Str(a.clone())).collect()));
            o.insert("messages".to_string(), JsonValue::Array(msgs));
            arr.push(JsonValue::Object(o));
        }
        let mut root = HashMap::new();
        root.insert("canaux".to_string(), JsonValue::Array(arr));
        root.insert("prochain_id".to_string(), JsonValue::Int(self.prochain_id as i64));
        JsonValue::Object(root)
    }

    pub fn from_json(v: &JsonValue) -> Self {
        let mut canaux = Vec::new();
        let mut prochain_id = 1u64;
        if let Some(obj) = v.as_object() {
            prochain_id = obj.get("prochain_id").and_then(|x| x.as_i64()).unwrap_or(1) as u64;
            if let Some(JsonValue::Array(arr)) = obj.get("canaux") {
                for item in arr {
                    if let Some(c) = item.as_object() {
                        let g = |k: &str| c.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let gi = |k: &str| c.get(k).and_then(|v| v.as_i64()).unwrap_or(0);
                        let mut messages = Vec::new();
                        if let Some(JsonValue::Array(msgs)) = c.get("messages") {
                            for m in msgs {
                                if let Some(mo) = m.as_object() {
                                    let mut commentaires = Vec::new();
                                    if let Some(JsonValue::Array(cmts)) = mo.get("commentaires") {
                                        for cm in cmts {
                                            if let Some(co) = cm.as_object() {
                                                let mut reponses = Vec::new();
                                                if let Some(JsonValue::Array(reps)) = co.get("reponses") {
                                                    for r in reps {
                                                        if let Some(ro) = r.as_object() {
                                                            reponses.push(RepTelegram {
                                                                de: ro.get("de").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                                                texte: ro.get("texte").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                                                heure: ro.get("heure").and_then(|v| v.as_i64()).unwrap_or(0),
                                                                likers: ro.get("likers").and_then(|v| v.as_array()).map(|arr|
                                                                    arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
                                                            });
                                                        }
                                                    }
                                                }
                                                commentaires.push(CommentTelegram {
                                                    de: co.get("de").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                                    texte: co.get("texte").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                                    heure: co.get("heure").and_then(|v| v.as_i64()).unwrap_or(0),
                                                    likers: co.get("likers").and_then(|v| v.as_array()).map(|arr|
                                                        arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
                                                    reponses,
                                                });
                                            }
                                        }
                                    }
                                    messages.push(MsgTelegram {
                                        de: mo.get("de").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        texte: mo.get("texte").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        heure: mo.get("heure").and_then(|v| v.as_i64()).unwrap_or(0),
                                        commentaires,
                                    });
                                }
                            }
                        }
                        let mut abonnes = Vec::new();
                        if let Some(JsonValue::Array(abs)) = c.get("abonnes") {
                            for a in abs {
                                if let Some(s) = a.as_str() {
                                    abonnes.push(s.to_string());
                                }
                            }
                        }
                        canaux.push(CanalTelegram {
                            id: gi("id") as u64,
                            nom: g("nom"),
                            description: g("description"),
                            createur: g("createur"),
                            abonnes,
                            messages,
                        });
                    }
                }
            }
        }
        TelegramStore { canaux, prochain_id, chemin: crate::data_path("telegram.json") }
    }
}
