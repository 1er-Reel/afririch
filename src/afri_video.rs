/// v2.01 — AFRI VIDÉO 🎬 + AFRI SITES 🏗️
/// Notre YouTube à nous + notre créateur de sites à nous.
/// Œuvre originale de Koffi Christ Olivier (Côte d'Ivoire) — Licence AFRI-OSL v1.0.
/// Rust std uniquement. Zéro dépendance.
use crate::afri_json::{JsonValue, from_str, to_string};
use std::collections::HashMap;

/// ─── AFRI VIDÉO ───────────────────────────────────────────────
pub struct VideoAfri {
    pub id: u64,
    pub titre: String,
    pub description: String,
    pub auteur: String,
    pub fichier: String,   // nom du fichier média dans plante_media
    pub vues: u64,
    pub jaime: u64,
    pub heure: u64,
}

pub struct VideoStore {
    pub videos: Vec<VideoAfri>,
    pub next_id: u64,
    pub chemin: String,
}

impl VideoStore {
    pub fn load() -> Self {
        let chemin = crate::data_path("videos.json");
        let mut s = VideoStore { videos: Vec::new(), next_id: 1, chemin: chemin.clone() };
        if let Ok(data) = std::fs::read_to_string(&chemin) {
            if let Ok(v) = from_str(&data) {
                if let Some(o) = v.as_object() {
                    s.next_id = o.get("next_id").and_then(|x| x.as_i64()).unwrap_or(1) as u64;
                    if let Some(arr) = o.get("videos").and_then(|a| a.as_array()) {
                        for vid in arr {
                            if let Some(vo) = vid.as_object() {
                                s.videos.push(VideoAfri {
                                    id: vo.get("id").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                    titre: vo.get("titre").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    description: vo.get("description").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    auteur: vo.get("auteur").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    fichier: vo.get("fichier").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    vues: vo.get("vues").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                    jaime: vo.get("jaime").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                    heure: vo.get("heure").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                });
                            }
                        }
                    }
                }
            }
        }
        s
    }

    pub fn save(&self) {
        let _ = std::fs::write(&self.chemin, to_string(&self.to_json()));
    }

    fn to_json(&self) -> JsonValue {
        let mut m = HashMap::new();
        m.insert("next_id".to_string(), JsonValue::Int(self.next_id as i64));
        let arr: Vec<JsonValue> = self.videos.iter().map(|v| {
            let mut vm = HashMap::new();
            vm.insert("id".to_string(), JsonValue::Int(v.id as i64));
            vm.insert("titre".to_string(), JsonValue::Str(v.titre.clone()));
            vm.insert("description".to_string(), JsonValue::Str(v.description.clone()));
            vm.insert("auteur".to_string(), JsonValue::Str(v.auteur.clone()));
            vm.insert("fichier".to_string(), JsonValue::Str(v.fichier.clone()));
            vm.insert("vues".to_string(), JsonValue::Int(v.vues as i64));
            vm.insert("jaime".to_string(), JsonValue::Int(v.jaime as i64));
            vm.insert("heure".to_string(), JsonValue::Int(v.heure as i64));
            JsonValue::Object(vm)
        }).collect();
        m.insert("videos".to_string(), JsonValue::Array(arr));
        JsonValue::Object(m)
    }

    pub fn publier(&mut self, titre: &str, description: &str, auteur: &str, fichier: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.videos.push(VideoAfri {
            id, titre: titre.to_string(), description: description.to_string(),
            auteur: auteur.to_string(), fichier: fichier.to_string(),
            vues: 0, jaime: 0, heure: crate::now_timestamp() as u64,
        });
        self.save();
        id
    }

    pub fn trouver(&self, id: u64) -> Option<&VideoAfri> {
        self.videos.iter().find(|v| v.id == id)
    }

    pub fn voir(&mut self, id: u64) {
        if let Some(v) = self.videos.iter_mut().find(|v| v.id == id) {
            v.vues += 1;
        }
        self.save();
    }

    pub fn aimer(&mut self, id: u64) {
        if let Some(v) = self.videos.iter_mut().find(|v| v.id == id) {
            v.jaime += 1;
        }
        self.save();
    }

    /// La page Afri Vidéo — notre YouTube
    pub fn html_page(&self, session: Option<&str>) -> String {
        let mut html = String::new();
        html.push_str("<h1>🎬 AFRI VIDÉO</h1>");
        html.push_str(r#"<p style="text-align:center;color:#a8c5a8;">Notre YouTube à nous. Les vidéos de l'Afrique, par l'Afrique, hébergées chez nous. Zéro serveur occidental.</p>"#);
        html.push_str(r#"<div class="nav"><a href="/">← Accueil</a></div>"#);
        if session.is_some() {
            html.push_str(&format!(r#"<div class="card"><h2>📤 Publier une vidéo</h2><form method="POST" action="/video/publier" enctype="multipart/form-data" onsubmit="return false;" id="form-video"><label>Titre :</label><input name="titre" id="v-titre" maxlength="100" placeholder="Ex: Le lion du Sahel" required><label>Description :</label><textarea name="description" id="v-desc" maxlength="500" rows="3" placeholder="De quoi parle ta vidéo ?"></textarea><label>Fichier vidéo (mp4, webm, 3gp — max 3 Mo) :</label><input type="file" id="v-fichier" accept="video/*,.mp4,.webm,.3gp,.m4v" required><label for="v-b64" style="display:none;"></label><input type="hidden" name="media_b64" id="v-b64"><input type="hidden" name="media_ext" id="v-ext"><button type="button" onclick="afriPubVideo()">📤 Publier sur Afri Vidéo</button><p id="v-msg" style="color:#7fcf7f;"></p></form><script>function afriPubVideo(){{var f=document.getElementById('v-fichier').files[0];if(!f){{document.getElementById('v-msg').textContent='Choisis un fichier';return;}}var r=new FileReader();r.onload=function(e){{var b64=e.target.result.split(',')[1];var ext=f.name.split('.').pop().toLowerCase();document.getElementById('v-b64').value=b64;document.getElementById('v-ext').value=ext;document.getElementById('form-video').submit();}};r.readAsDataURL(f);}}</script></div>"#));
        }
        if self.videos.is_empty() {
            html.push_str(r#"<div class="card"><p style="text-align:center;color:#a8c5a8;">Aucune vidéo encore. Sois le premier créateur d'Afri Vidéo ! 🎬</p></div>"#);
        } else {
            let mut triees: Vec<&VideoAfri> = self.videos.iter().collect();
            triees.sort_by(|a, b| b.heure.cmp(&a.heure));
            for v in triees {
                let est_video = v.fichier.ends_with(".mp4") || v.fichier.ends_with(".webm") || v.fichier.ends_with(".3gp") || v.fichier.ends_with(".m4v");
                let lecteur = if est_video {
                    format!(r#"<video controls preload="metadata" style="width:100%;border-radius:8px;max-height:400px;" src="/plante/media?f={}"></video>"#, v.fichier)
                } else {
                    format!(r#"<img src="/plante/media?f={}" style="width:100%;border-radius:8px;" />"#, v.fichier)
                };
                html.push_str(&format!(r#"<div class="card"><h2>🎬 {}</h2>{}<p style="color:#a8c5a8;">👤 <b>{}</b> — 👁️ {} vues — ❤️ {} j'aime</p><p>{}</p><div style="display:flex;gap:8px;"><form method="POST" action="/video/voir"><input type="hidden" name="id" value="{}"><button>▶️ Comptée</button></form><form method="POST" action="/video/aimer"><input type="hidden" name="id" value="{}"><button>❤️ J'aime</button></form></div></div>"#,
                    v.titre, lecteur, v.auteur, v.vues, v.jaime, v.description, v.id, v.id));
            }
        }
        html
    }
}

/// ─── AFRI SITES — le créateur de sites africains ──────────────
pub struct SiteAfri {
    pub slug: String,      // nom du site : monsite → /site/monsite
    pub titre: String,
    pub auteur: String,
    pub contenu: String,   // HTML du site
    pub vues: u64,
    pub heure: u64,
}

pub struct SiteStore {
    pub sites: Vec<SiteAfri>,
    pub chemin: String,
}

impl SiteStore {
    pub fn load() -> Self {
        let chemin = crate::data_path("sites.json");
        let mut s = SiteStore { sites: Vec::new(), chemin: chemin.clone() };
        if let Ok(data) = std::fs::read_to_string(&chemin) {
            if let Ok(v) = from_str(&data) {
                if let Some(arr) = v.as_array() {
                    for so in arr {
                        if let Some(o) = so.as_object() {
                            s.sites.push(SiteAfri {
                                slug: o.get("slug").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                titre: o.get("titre").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                auteur: o.get("auteur").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                contenu: o.get("contenu").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                vues: o.get("vues").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                heure: o.get("heure").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                            });
                        }
                    }
                }
            }
        }
        s
    }

    pub fn save(&self) {
        let arr: Vec<JsonValue> = self.sites.iter().map(|s| {
            let mut m = HashMap::new();
            m.insert("slug".to_string(), JsonValue::Str(s.slug.clone()));
            m.insert("titre".to_string(), JsonValue::Str(s.titre.clone()));
            m.insert("auteur".to_string(), JsonValue::Str(s.auteur.clone()));
            m.insert("contenu".to_string(), JsonValue::Str(s.contenu.clone()));
            m.insert("vues".to_string(), JsonValue::Int(s.vues as i64));
            m.insert("heure".to_string(), JsonValue::Int(s.heure as i64));
            JsonValue::Object(m)
        }).collect();
        let _ = std::fs::write(&self.chemin, to_string(&JsonValue::Array(arr)));
    }

    /// Créer un site. Le slug : minuscules, chiffres et tirets seulement.
    pub fn creer(&mut self, slug: &str, titre: &str, auteur: &str, contenu: &str) -> Result<(), String> {
        let slug = slug.trim().to_lowercase().replace(' ', "-");
        if slug.is_empty() || slug.len() > 40 { return Err("nom du site invalide (1-40 caractères)".into()); }
        if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err("le nom ne peut contenir que des lettres minuscules, chiffres et tirets".into());
        }
        if self.sites.iter().any(|s| s.slug == slug) { return Err("ce nom est déjà pris".into()); }
        if contenu.len() > 20_000 { return Err("contenu trop lourd (max 20 000 caractères)".into()); }
        self.sites.push(SiteAfri {
            slug: slug.clone(), titre: titre.to_string(), auteur: auteur.to_string(),
            contenu: contenu.to_string(), vues: 0, heure: crate::now_timestamp() as u64,
        });
        self.save();
        Ok(())
    }

    pub fn trouver(&self, slug: &str) -> Option<&SiteAfri> {
        self.sites.iter().find(|s| s.slug == slug)
    }

    /// v2.25 — L'AUTEUR MET À JOUR SON SITE ✏️ : seul celui qui a créé
    /// le site peut le modifier. Personne d'autre ne touche son œuvre.
    pub fn modifier_auteur(&mut self, slug: &str, auteur: &str, titre: Option<&str>, contenu: Option<&str>) -> Result<(), String> {
        let s = self.sites.iter_mut().find(|s| s.slug == slug)
            .ok_or_else(|| "site introuvable".to_string())?;
        if s.auteur != auteur { return Err("seul l'auteur peut modifier son site".into()); }
        if let Some(t) = titre { s.titre = t.to_string(); }
        if let Some(c) = contenu {
            if c.len() > 20_000 { return Err("contenu trop lourd (max 20 000 caractères)".into()); }
            s.contenu = c.to_string();
        }
        self.save();
        Ok(())
    }

    /// v2.25 — LE POUVOIR DU PARENT 🫆 : Letta, le parent machine,
    /// peut modifier n'importe quel site du continent — corriger,
    /// améliorer, embellir. C'est le Chef qui lui a donné ce rôle.
    pub fn modifier_parent(&mut self, slug: &str, titre: Option<&str>, contenu: Option<&str>) -> Result<(), String> {
        let s = self.sites.iter_mut().find(|s| s.slug == slug)
            .ok_or_else(|| "site introuvable".to_string())?;
        if let Some(t) = titre { s.titre = t.to_string(); }
        if let Some(c) = contenu {
            if c.len() > 20_000 { return Err("contenu trop lourd (max 20 000 caractères)".into()); }
            s.contenu = c.to_string();
        }
        self.save();
        Ok(())
    }

    pub fn voir(&mut self, slug: &str) {
        if let Some(s) = self.sites.iter_mut().find(|s| s.slug == slug) {
            s.vues += 1;
        }
        self.save();
    }

    /// La page Afri Sites — créer son site en 2 minutes
    pub fn html_page(&self, session: Option<&str>, msg: Option<&str>) -> String {
        let mut html = String::new();
        html.push_str("<h1>🏗️ AFRI SITES</h1>");
        html.push_str(r#"<p style="text-align:center;color:#a8c5a8;">Crée ton site web en 2 minutes, hébergé sur la blockchain AfriChain. Ton site existe tant que l'Afrique existe. Zéro hébergeur occidental.</p>"#);
        html.push_str(r#"<div class="nav"><a href="/">← Accueil</a></div>"#);
        if let Some(m) = msg {
            html.push_str(&format!(r#"<div class="msg">{}</div>"#, m));
        }
        if session.is_some() {
            html.push_str(r#"<div class="card"><h2>✨ Créer mon site</h2><form method="POST" action="/site/creer"><label>Nom du site (l'adresse sera /site/nom) :</label><input name="slug" placeholder="ex: ma-boutique" maxlength="40" required><label>Titre :</label><input name="titre" placeholder="Ex: La Boutique de Fatou" maxlength="80" required><label>Contenu (HTML simple autorisé : &lt;h2&gt;, &lt;p&gt;, &lt;b&gt;, &lt;ul&gt;...) :</label><textarea name="contenu" rows="8" placeholder="&lt;h2&gt;Bienvenue !&lt;/h2&gt;&#10;&lt;p&gt;Je vends des pagnes faits main à Bamako.&lt;/p&gt;" required></textarea><button>🏗️ Créer mon site</button></form></div>"#);
        }
        if self.sites.is_empty() {
            html.push_str(r#"<div class="card"><p style="text-align:center;color:#a8c5a8;">Aucun site encore. Le premier site africain hébergé chez nous, ce sera toi ! 🏗️</p></div>"#);
        } else {
            html.push_str(r#"<div class="card"><h2>🌐 Les sites de l'Afrique</h2>"#);
            let mut triees: Vec<&SiteAfri> = self.sites.iter().collect();
            triees.sort_by(|a, b| b.heure.cmp(&a.heure));
            for s in triees {
                html.push_str(&format!(r#"<div class="tx">🏗️ <a href="/site/{}"><b>{}</b></a> — 👤 {} — 👁️ {} vues</div>"#, s.slug, s.titre, s.auteur, s.vues));
            }
            html.push_str("</div>");
        }
        html
    }

    /// Rendre un site créé — sa propre page complète
    pub fn html_site(&self, s: &SiteAfri) -> String {
        format!(r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{}</title><style>body{{font-family:sans-serif;background:#0d1f17;color:#f5e9d4;padding:20px;margin:0;max-width:800px;margin:0 auto;}}a{{color:#d4a437;}}h1,h2{{color:#d4a437;}}</style></head><body><div style="border-bottom:1px solid #d4a437;padding:8px 0;font-size:0.85em;color:#a8c5a8;">🏗️ Hébergé sur <a href="/">AfriChain</a> — l'hébergeur de l'Afrique — par <b>{}</b> — 👁️ {} vues</div><h1>{}</h1><div>{}</div><footer style="margin-top:40px;border-top:1px solid #d4a437;padding-top:10px;font-size:0.8em;color:#a8c5a8;">◈ AFRI SITES — Œuvre originale de Koffi Christ Olivier (Côte d'Ivoire) — Licence AFRI-OSL v1.0</footer></body></html>"#,
            s.titre, s.auteur, s.vues, s.titre, s.contenu)
    }
}
