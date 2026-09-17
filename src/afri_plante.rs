// ===== AFRI PLANTE — Planté Verte: le réseau social africain =====
// v1.82 — LE VRAI FACEBOOK AFRICAIN 🌱✨
// Profils (avatar + bio), commentaires, partages, photos/vidéos (reels),
// stories 24h, invitations → amis, étoiles, boost 🚀, boutique, transferts AFR.
// Règle du chef: on ne voit QUE ses propres posts et ceux de ses amis. 🔒
// Chaque publication est gravée sur la blockchain — rien ne sort du continent.

use std::collections::HashMap;
use crate::afri_json::{JsonValue, to_string, from_str};
use crate::afri_time::{now_timestamp, now_timestamp_millis};

/// Une story vit 24h — comme dans la vraie vie, elle disparaît. ⏳
pub const DUREE_STORY: i64 = 86400;
/// 🚀 Booster une publication: 10 AFR — elle monte en haut du fil.
pub const PRIX_BOOST: u64 = 10;

/// Les extensions média autorisées (photos + vidéos = reels)
pub const EXTENSIONS_MEDIA: [&str; 9] = ["jpg", "jpeg", "png", "gif", "webp", "mp4", "webm", "3gp", "m4v"];

/// Un commentaire sous un post
#[derive(Clone, Debug)]
pub struct Commentaire {
    pub author: String,
    pub texte: String,
    pub date: i64,
}

/// Un post de Planté Verte
#[derive(Clone, Debug)]
pub struct PostPlante {
    pub author: String,
    pub contenu: String,
    pub date: i64,
    pub likes: u64,
    pub likers: Vec<String>,          // qui a aimé (1 seul like par personne)
    pub media: Option<String>,        // fichier média dans plante_media/
    pub media_type: String,           // "image" | "video" | ""
    pub commentaires: Vec<Commentaire>,
    pub partages: u64,
    pub boost: bool,                  // 🚀 post boosté (en haut du fil)
    pub shared_from: Option<String>,  // auteur d'origine si c'est un partage
}

/// Une story (expire après 24h)
#[derive(Clone, Debug)]
pub struct StoryPlante {
    pub author: String,
    pub media: String,
    pub date: i64,
}

/// Une invitation: de → pour, en attente d'acceptation 🤝
#[derive(Clone, Debug)]
pub struct InvitationPlante {
    pub de: String,
    pub pour: String,
    pub date: i64,
}

/// Profil: avatar + bio + badges achetés à la boutique
#[derive(Clone, Debug, Default)]
pub struct ProfilPlante {
    pub avatar: Option<String>,
    pub bio: String,
    pub badges: Vec<String>,
}

/// Le magasin Planté Verte
pub struct PlanteStore {
    pub posts: Vec<PostPlante>,
    pub contacts: HashMap<String, Vec<String>>,  // username → ses amis
    pub profils: HashMap<String, ProfilPlante>,
    pub stories: Vec<StoryPlante>,
    pub invitations: Vec<InvitationPlante>,
}

impl PlanteStore {
    pub fn nouveau() -> Self {
        PlanteStore {
            posts: Vec::new(),
            contacts: HashMap::new(),
            profils: HashMap::new(),
            stories: Vec::new(),
            invitations: Vec::new(),
        }
    }

    pub fn chemin_data() -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/afririch/plante.json", home)
    }

    /// Le dossier où vivent les photos et vidéos (rien ne sort du continent)
    pub fn chemin_media() -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/afririch/plante_media", home)
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

    // ===== AMIS (CONTACTS) =====

    pub fn ajouter_contact(&mut self, moi: &str, ami: &str) -> bool {
        if moi == ami { return false; }
        let liste = self.contacts.entry(moi.to_string()).or_default();
        if liste.iter().any(|c| c == ami) { return false; }
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

    pub fn est_contact(&self, moi: &str, ami: &str) -> bool {
        self.contacts.get(moi).map(|l| l.iter().any(|c| c == ami)).unwrap_or(false)
    }

    pub fn contacts_de(&self, moi: &str) -> Vec<String> {
        self.contacts.get(moi).cloned().unwrap_or_default()
    }

    // ===== INVITATIONS 🤝 =====

    /// Inviter un frère: refuse si déjà amis ou invitation déjà en attente (dans un sens ou l'autre)
    pub fn inviter(&mut self, de: &str, pour: &str) -> bool {
        if de == pour { return false; }
        if self.est_contact(de, pour) || self.est_contact(pour, de) { return false; }
        if self.invitations.iter().any(|i|
            (i.de == de && i.pour == pour) || (i.de == pour && i.pour == de)) { return false; }
        self.invitations.push(InvitationPlante {
            de: de.to_string(), pour: pour.to_string(), date: now_timestamp(),
        });
        true
    }

    /// Accepter une invitation → amitié MUTUELLE gravée 💚
    pub fn accepter_invitation(&mut self, de: &str, moi: &str) -> bool {
        let avant = self.invitations.len();
        self.invitations.retain(|i| !(i.de == de && i.pour == moi));
        if self.invitations.len() == avant { return false; }
        self.ajouter_contact(moi, de);
        self.ajouter_contact(de, moi);
        true
    }

    pub fn refuser_invitation(&mut self, de: &str, moi: &str) -> bool {
        let avant = self.invitations.len();
        self.invitations.retain(|i| !(i.de == de && i.pour == moi));
        self.invitations.len() < avant
    }

    /// Les invitations qui attendent MA réponse
    pub fn invitations_recues(&self, moi: &str) -> Vec<(String, i64)> {
        self.invitations.iter()
            .filter(|i| i.pour == moi)
            .map(|i| (i.de.clone(), i.date))
            .collect()
    }

    /// Les invitations que J'AI envoyées (en attente)
    pub fn invitations_envoyees(&self, moi: &str) -> Vec<String> {
        self.invitations.iter().filter(|i| i.de == moi).map(|i| i.pour.clone()).collect()
    }

    // ===== POSTS =====

    pub fn publier(&mut self, author: &str, contenu: &str) {
        self.publier_media(author, contenu, None, "", None);
    }

    pub fn publier_media(&mut self, author: &str, contenu: &str, media: Option<String>, media_type: &str, shared_from: Option<String>) {
        self.posts.push(PostPlante {
            author: author.to_string(),
            contenu: contenu.to_string(),
            date: now_timestamp(),
            likes: 0,
            likers: Vec::new(),
            media,
            media_type: media_type.to_string(),
            commentaires: Vec::new(),
            partages: 0,
            boost: false,
            shared_from,
        });
    }

    /// LE FIL PRIVÉ (indices): mes posts + les posts de MES amis seulement. 🔒
    /// Boostés 🚀 en premier, puis du plus récent au plus vieux.
    pub fn fil_prive_indices(&self, moi: &str) -> Vec<usize> {
        let mes_amis = self.contacts_de(moi);
        let mut idx: Vec<usize> = self.posts.iter().enumerate()
            .filter(|(_, p)| p.author == moi || mes_amis.iter().any(|c| *c == p.author))
            .map(|(i, _)| i)
            .collect();
        idx.sort_by(|a, b| {
            let pa = &self.posts[*a];
            let pb = &self.posts[*b];
            pb.boost.cmp(&pa.boost).then(pb.date.cmp(&pa.date))
        });
        idx
    }

    /// Compat: le fil privé (références)
    pub fn fil_prive(&self, moi: &str) -> Vec<&PostPlante> {
        self.fil_prive_indices(moi).into_iter()
            .filter_map(|i| self.posts.get(i))
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

    /// Commenter un post 💬
    pub fn commenter(&mut self, index: usize, author: &str, texte: &str) -> bool {
        if let Some(post) = self.posts.get_mut(index) {
            post.commentaires.push(Commentaire {
                author: author.to_string(),
                texte: texte.to_string(),
                date: now_timestamp(),
            });
            true
        } else {
            false
        }
    }

    /// Partager un post: le repost entre dans MON fil, l'original gagne +1 partage ↗️
    pub fn partager(&mut self, index: usize, moi: &str) -> bool {
        if let Some(post) = self.posts.get(index) {
            let orig = post.clone();
            if orig.author == moi { return false; } // partager son propre post n'a pas de sens
            self.publier_media(
                moi,
                &orig.contenu,
                orig.media.clone(),
                &orig.media_type,
                Some(orig.author.clone()),
            );
            if let Some(p) = self.posts.get_mut(index) { p.partages += 1; }
            true
        } else {
            false
        }
    }

    /// 🚀 Booster un post (déjà payé par la route) — il monte en haut du fil
    pub fn booster(&mut self, index: usize) -> bool {
        if let Some(post) = self.posts.get_mut(index) {
            post.boost = true;
            true
        } else {
            false
        }
    }

    // ===== STORIES ⏳ =====

    pub fn ajouter_story(&mut self, author: &str, media: &str) {
        self.stories.push(StoryPlante {
            author: author.to_string(),
            media: media.to_string(),
            date: now_timestamp(),
        });
    }

    /// Les stories vivantes (moins de 24h) — visibles dans le fil privé
    pub fn stories_actives(&self, moi: &str) -> Vec<&StoryPlante> {
        let mes_amis = self.contacts_de(moi);
        let limite = now_timestamp() - DUREE_STORY;
        self.stories.iter()
            .filter(|s| s.date > limite)
            .filter(|s| s.author == moi || mes_amis.iter().any(|c| *c == s.author))
            .collect()
    }

    // ===== PROFILS + ÉTOILES =====

    pub fn profil_de(&self, user: &str) -> ProfilPlante {
        self.profils.get(user).cloned().unwrap_or_default()
    }

    pub fn set_profil(&mut self, user: &str, profil: ProfilPlante) {
        self.profils.insert(user.to_string(), profil);
    }

    pub fn ajouter_badge(&mut self, user: &str, badge: &str) {
        let mut p = self.profil_de(user);
        if !p.badges.iter().any(|b| b == badge) {
            p.badges.push(badge.to_string());
            self.set_profil(user, p);
        }
    }

    /// ⭐ Les étoiles d'un utilisateur = le total de likes reçus sur ses posts
    pub fn etoiles_de(&self, moi: &str) -> u64 {
        self.posts.iter().filter(|p| p.author == moi).map(|p| p.likes).sum()
    }

    // ===== MÉDIAS (photos + vidéos) =====

    /// Décoder base64 (from scratch — zéro dépendance)
    pub fn base64_decode(s: &str) -> Option<Vec<u8>> {
        fn val(c: u8) -> Option<u32> {
            match c {
                b'A'..=b'Z' => Some((c - b'A') as u32),
                b'a'..=b'z' => Some((c - b'a' + 26) as u32),
                b'0'..=b'9' => Some((c - b'0' + 52) as u32),
                b'+' => Some(62),
                b'/' => Some(63),
                _ => None,
            }
        }
        let clean: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
        let mut out = Vec::with_capacity(clean.len() * 3 / 4);
        let mut buf: u32 = 0;
        let mut bits: u32 = 0;
        for &c in &clean {
            if c == b'=' { break; }
            buf = (buf << 6) | val(c)?;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push(((buf >> bits) & 0xFF) as u8);
            }
        }
        Some(out)
    }

    /// Sauver un média (base64 + extension) → nom de fichier. Max 3 Mo.
    pub fn sauver_media(b64: &str, ext: &str) -> Result<String, String> {
        let ext = ext.to_lowercase();
        if !EXTENSIONS_MEDIA.contains(&ext.as_str()) {
            return Err(format!("format .{} non autorisé (jpg, png, gif, webp, mp4, webm, 3gp)", ext));
        }
        let bytes = Self::base64_decode(b64).ok_or("média illisible".to_string())?;
        if bytes.is_empty() { return Err("média vide".to_string()); }
        if bytes.len() > 3 * 1024 * 1024 { return Err("trop lourd (max 3 Mo)".to_string()); }
        let dir = Self::chemin_media();
        std::fs::create_dir_all(&dir).map_err(|_| "erreur disque".to_string())?;
        let h = crate::afri_hash::afrihash_256(&bytes);
        let nom = format!("{}_{}.{}", now_timestamp_millis(), &crate::afri_hex::encode(&h[..4]), ext);
        std::fs::write(format!("{}/{}", dir, nom), &bytes).map_err(|_| "erreur écriture".to_string())?;
        Ok(nom)
    }

    /// Le type MIME d'un média servi
    pub fn media_content_type(fichier: &str) -> &'static str {
        if fichier.ends_with(".png") { "image/png" }
        else if fichier.ends_with(".gif") { "image/gif" }
        else if fichier.ends_with(".webp") { "image/webp" }
        else if fichier.ends_with(".mp4") || fichier.ends_with(".m4v") { "video/mp4" }
        else if fichier.ends_with(".webm") { "video/webm" }
        else if fichier.ends_with(".3gp") { "video/3gpp" }
        else { "image/jpeg" }
    }

    /// Est-ce une extension vidéo ? (pour les reels 🎬)
    pub fn est_video(ext: &str) -> bool {
        matches!(ext.to_lowercase().as_str(), "mp4" | "webm" | "3gp" | "m4v")
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
            m.insert("media".to_string(), match &p.media {
                Some(f) => JsonValue::Str(f.clone()),
                None => JsonValue::Null,
            });
            m.insert("media_type".to_string(), JsonValue::Str(p.media_type.clone()));
            m.insert("commentaires".to_string(), JsonValue::Array(
                p.commentaires.iter().map(|c| {
                    let mut cm = HashMap::new();
                    cm.insert("author".to_string(), JsonValue::Str(c.author.clone()));
                    cm.insert("texte".to_string(), JsonValue::Str(c.texte.clone()));
                    cm.insert("date".to_string(), JsonValue::Int(c.date));
                    JsonValue::Object(cm)
                }).collect()));
            m.insert("partages".to_string(), JsonValue::Int(p.partages as i64));
            m.insert("boost".to_string(), JsonValue::Bool(p.boost));
            m.insert("shared_from".to_string(), match &p.shared_from {
                Some(a) => JsonValue::Str(a.clone()),
                None => JsonValue::Null,
            });
            JsonValue::Object(m)
        }).collect();
        root.insert("posts".to_string(), JsonValue::Array(posts));
        let mut contacts = HashMap::new();
        for (user, liste) in &self.contacts {
            contacts.insert(user.clone(), JsonValue::Array(
                liste.iter().map(|c| JsonValue::Str(c.clone())).collect()));
        }
        root.insert("contacts".to_string(), JsonValue::Object(contacts));
        let mut profils = HashMap::new();
        for (user, p) in &self.profils {
            let mut pm = HashMap::new();
            pm.insert("avatar".to_string(), match &p.avatar {
                Some(a) => JsonValue::Str(a.clone()),
                None => JsonValue::Null,
            });
            pm.insert("bio".to_string(), JsonValue::Str(p.bio.clone()));
            pm.insert("badges".to_string(), JsonValue::Array(
                p.badges.iter().map(|b| JsonValue::Str(b.clone())).collect()));
            profils.insert(user.clone(), JsonValue::Object(pm));
        }
        root.insert("profils".to_string(), JsonValue::Object(profils));
        root.insert("stories".to_string(), JsonValue::Array(
            self.stories.iter().map(|s| {
                let mut sm = HashMap::new();
                sm.insert("author".to_string(), JsonValue::Str(s.author.clone()));
                sm.insert("media".to_string(), JsonValue::Str(s.media.clone()));
                sm.insert("date".to_string(), JsonValue::Int(s.date));
                JsonValue::Object(sm)
            }).collect()));
        root.insert("invitations".to_string(), JsonValue::Array(
            self.invitations.iter().map(|i| {
                let mut im = HashMap::new();
                im.insert("de".to_string(), JsonValue::Str(i.de.clone()));
                im.insert("pour".to_string(), JsonValue::Str(i.pour.clone()));
                im.insert("date".to_string(), JsonValue::Int(i.date));
                JsonValue::Object(im)
            }).collect()));
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
                            media: pm.get("media").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            media_type: pm.get("media_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            commentaires: match pm.get("commentaires") {
                                Some(JsonValue::Array(arr)) => arr.iter().filter_map(|c| {
                                    let cm = c.as_object()?;
                                    Some(Commentaire {
                                        author: cm.get("author").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        texte: cm.get("texte").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        date: cm.get("date").and_then(|v| v.as_i64()).unwrap_or(0),
                                    })
                                }).collect(),
                                _ => Vec::new(),
                            },
                            partages: pm.get("partages").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
                            boost: matches!(pm.get("boost"), Some(JsonValue::Bool(true))),
                            shared_from: pm.get("shared_from").and_then(|v| v.as_str()).map(|s| s.to_string()),
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
            if let Some(JsonValue::Object(profils)) = map.get("profils") {
                for (user, p) in profils {
                    let pm = match p.as_object() { Some(m) => m, None => continue };
                    store.profils.insert(user.clone(), ProfilPlante {
                        avatar: pm.get("avatar").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        bio: pm.get("bio").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        badges: match pm.get("badges") {
                            Some(JsonValue::Array(arr)) => arr.iter()
                                .filter_map(|b| b.as_str().map(|s| s.to_string())).collect(),
                            _ => Vec::new(),
                        },
                    });
                }
            }
            if let Some(JsonValue::Array(stories)) = map.get("stories") {
                for s in stories {
                    if let Some(sm) = s.as_object() {
                        store.stories.push(StoryPlante {
                            author: sm.get("author").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            media: sm.get("media").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            date: sm.get("date").and_then(|v| v.as_i64()).unwrap_or(0),
                        });
                    }
                }
            }
            if let Some(JsonValue::Array(invitations)) = map.get("invitations") {
                for i in invitations {
                    if let Some(im) = i.as_object() {
                        store.invitations.push(InvitationPlante {
                            de: im.get("de").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            pour: im.get("pour").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            date: im.get("date").and_then(|v| v.as_i64()).unwrap_or(0),
                        });
                    }
                }
            }
        }
        store
    }
}
