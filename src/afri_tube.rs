/// v2.18 — AFRITUBE 📺✈️ — YouTube en MODE AVION.
/// La blockchain ne stocke pas la vidéo : elle stocke l'EMPREINTE et
/// CHEZ QUEL VOISIN la vidéo dort. L'enfant ouvre AfriTube, lit la
/// chaîne, va chercher la vidéo chez le voisin en wifi local (192.168.x.x).
/// Même route coupée, même mode avion : la vidéo est là. 💚
/// Œuvre originale de Koffi Christ Olivier (Côte d'Ivoire) — Licence AFRI-OSL v1.0.
/// Rust std uniquement. Zéro dépendance.
use crate::afri_json::{JsonValue, from_str, to_string};
use std::collections::HashMap;

/// Une vidéo du tube : une empreinte + une adresse de voisin.
pub struct VideoTube {
    pub id: u64,
    pub titre: String,
    pub description: String,
    pub fichier: String,      // nom du fichier média (dans plante_media du voisin)
    pub empreinte: String,    // AfriHash-256 du contenu (les 8 premiers octets, hex)
    pub taille: u64,          // taille en octets
    pub hote_ip: String,      // chez qui la vidéo dort (ex: 192.168.1.7)
    pub hote_port: u64,       // port du voisin (8080)
    pub hote_nom: String,     // nom du node (ex: "Redmi 3 du village")
    pub auteur: String,
    pub heure: u64,
    pub vues: u64,
}

/// Un voisin du tube : un node qui partage ses vidéos en wifi local.
pub struct VoisinTube {
    pub ip: String,
    pub port: u64,
    pub nom: String,
    pub heure: u64,
}

pub struct TubeStore {
    pub videos: Vec<VideoTube>,
    pub voisins: Vec<VoisinTube>,
    pub next_id: u64,
    chemin: String,
}

impl TubeStore {
    pub fn load() -> Self {
        let chemin = crate::data_path("tube.json");
        let mut s = TubeStore { videos: Vec::new(), voisins: Vec::new(), next_id: 1, chemin: chemin.clone() };
        if let Ok(data) = std::fs::read_to_string(&chemin) {
            if let Ok(v) = from_str(&data) {
                if let Some(o) = v.as_object() {
                    s.next_id = o.get("next_id").and_then(|x| x.as_i64()).unwrap_or(1) as u64;
                    if let Some(arr) = o.get("videos").and_then(|a| a.as_array()) {
                        for vo in arr.iter().filter_map(|x| x.as_object()) {
                            s.videos.push(VideoTube {
                                id: vo.get("id").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                titre: vo.get("titre").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                description: vo.get("description").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                fichier: vo.get("fichier").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                empreinte: vo.get("empreinte").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                taille: vo.get("taille").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                hote_ip: vo.get("hote_ip").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                hote_port: vo.get("hote_port").and_then(|x| x.as_i64()).unwrap_or(8080) as u64,
                                hote_nom: vo.get("hote_nom").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                auteur: vo.get("auteur").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                heure: vo.get("heure").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                vues: vo.get("vues").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                            });
                        }
                    }
                    if let Some(arr) = o.get("voisins").and_then(|a| a.as_array()) {
                        for vo in arr.iter().filter_map(|x| x.as_object()) {
                            s.voisins.push(VoisinTube {
                                ip: vo.get("ip").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                port: vo.get("port").and_then(|x| x.as_i64()).unwrap_or(8080) as u64,
                                nom: vo.get("nom").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                heure: vo.get("heure").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                            });
                        }
                    }
                }
            }
        }
        s
    }

    pub fn save(&self) {
        let mut m = HashMap::new();
        m.insert("next_id".to_string(), JsonValue::Int(self.next_id as i64));
        let vids: Vec<JsonValue> = self.videos.iter().map(|v| {
            let mut vm = HashMap::new();
            vm.insert("id".to_string(), JsonValue::Int(v.id as i64));
            vm.insert("titre".to_string(), JsonValue::Str(v.titre.clone()));
            vm.insert("description".to_string(), JsonValue::Str(v.description.clone()));
            vm.insert("fichier".to_string(), JsonValue::Str(v.fichier.clone()));
            vm.insert("empreinte".to_string(), JsonValue::Str(v.empreinte.clone()));
            vm.insert("taille".to_string(), JsonValue::Int(v.taille as i64));
            vm.insert("hote_ip".to_string(), JsonValue::Str(v.hote_ip.clone()));
            vm.insert("hote_port".to_string(), JsonValue::Int(v.hote_port as i64));
            vm.insert("hote_nom".to_string(), JsonValue::Str(v.hote_nom.clone()));
            vm.insert("auteur".to_string(), JsonValue::Str(v.auteur.clone()));
            vm.insert("heure".to_string(), JsonValue::Int(v.heure as i64));
            vm.insert("vues".to_string(), JsonValue::Int(v.vues as i64));
            JsonValue::Object(vm)
        }).collect();
        m.insert("videos".to_string(), JsonValue::Array(vids));
        let vs: Vec<JsonValue> = self.voisins.iter().map(|v| {
            let mut vm = HashMap::new();
            vm.insert("ip".to_string(), JsonValue::Str(v.ip.clone()));
            vm.insert("port".to_string(), JsonValue::Int(v.port as i64));
            vm.insert("nom".to_string(), JsonValue::Str(v.nom.clone()));
            vm.insert("heure".to_string(), JsonValue::Int(v.heure as i64));
            JsonValue::Object(vm)
        }).collect();
        m.insert("voisins".to_string(), JsonValue::Array(vs));
        let _ = std::fs::write(&self.chemin, to_string(&JsonValue::Object(m)));
    }

    /// Déposer une empreinte : "la vidéo X dort chez le voisin Y".
    /// Retourne l'id + le memo à graver sur la blockchain.
    pub fn deposer(&mut self, titre: &str, description: &str, fichier: &str,
                   empreinte: &str, taille: u64, hote_ip: &str, hote_port: u64,
                   hote_nom: &str, auteur: &str) -> (u64, String) {
        let id = self.next_id;
        self.next_id += 1;
        self.videos.push(VideoTube {
            id, titre: titre.to_string(), description: description.to_string(),
            fichier: fichier.to_string(), empreinte: empreinte.to_string(),
            taille, hote_ip: hote_ip.to_string(), hote_port,
            hote_nom: hote_nom.to_string(), auteur: auteur.to_string(),
            heure: crate::now_timestamp() as u64, vues: 0,
        });
        self.save();
        let memo = format!("AFRITUBE | {} depose \"{}\" (empreinte {}) chez {}",
            auteur, titre, empreinte, hote_nom);
        (id, memo)
    }

    /// Annoncer un voisin (un node qui partage ses vidéos).
    pub fn annoncer_voisin(&mut self, ip: &str, port: u64, nom: &str) {
        if let Some(v) = self.voisins.iter_mut().find(|v| v.ip == ip && v.port == port) {
            v.nom = nom.to_string();
            v.heure = crate::now_timestamp() as u64;
        } else {
            self.voisins.push(VoisinTube {
                ip: ip.to_string(), port, nom: nom.to_string(),
                heure: crate::now_timestamp() as u64,
            });
        }
        self.save();
    }

    pub fn voir(&mut self, id: u64) {
        if let Some(v) = self.videos.iter_mut().find(|v| v.id == id) { v.vues += 1; }
        self.save();
    }

    /// La page AfriTube — YouTube en mode avion.
    pub fn html_page(&self, session: Option<&str>, mon_ip: &str, msg: Option<&str>) -> String {
        let mut html = String::new();
        html.push_str("<h1>📺 AFRITUBE</h1>");
        html.push_str(r#"<p style="text-align:center;color:#a8c5a8;">YouTube en <b style="color:#d4a437;">MODE AVION</b> ✈️ — La blockchain ne stocke pas la vidéo. Elle stocke l'<b>empreinte</b> et <b>chez quel voisin la vidéo dort</b>. L'enfant ouvre AfriTube, lit la chaîne, va chercher la vidéo chez le voisin en wifi local. Route coupée, mode avion — la vidéo est là. 💚</p>"#);
        html.push_str(r#"<div class="nav"><a href="/">← Accueil</a> | <a href="/video">🎬 Afri Vidéo</a></div>"#);
        if let Some(m) = msg {
            html.push_str(&format!(r#"<div class="msg">{}</div>"#, m));
        }

        // Ce node
        html.push_str(&format!(r#"<div class="card"><h2>📡 Ce node</h2><p style="color:#a8c5a8;">Ce serveur est <b style="color:#d4a437;">{}</b> — les voisins viennent chercher les vidéos ici : <span style="font-family:monospace;color:#7fcf7f;">http://{}:8080</span></p></div>"#,
            "Node AfriChain", mon_ip));

        // Déposer une empreinte
        if session.is_some() {
            html.push_str(&format!(r#"<div class="card"><h2>📤 Déposer une empreinte sur la chaîne</h2><p style="color:#a8c5a8;">Tu as une vidéo sur CE serveur (publiée sur Afri Vidéo) ? Dépose son empreinte : la chaîne dira aux enfants qu'elle dort ici. Si la vidéo dort chez un AUTRE voisin, écris son adresse IP.</p><form method="POST" action="/afritube/deposer"><label>Titre :</label><input name="titre" maxlength="100" placeholder="Ex: Comment planter le mil" required><label>Description :</label><textarea name="description" maxlength="500" rows="2" placeholder="De quoi parle la vidéo ?"></textarea><label>Nom du fichier média (ex: 1695_3f2a.mp4) :</label><input name="fichier" maxlength="80" placeholder="Le nom du fichier dans plante_media" required><label>Empreinte AfriHash (8 hex, optionnel) :</label><input name="empreinte" maxlength="16" placeholder="Calculée automatiquement si vide"><label>Adresse IP du voisin qui garde la vidéo :</label><input name="hote_ip" maxlength="40" placeholder="Ex: 192.168.1.7" required><label>Nom du node voisin :</label><input name="hote_nom" maxlength="40" placeholder="Ex: Redmi 3 du village" required><button>📡 Graver l'empreinte sur la blockchain</button></form></div>"#));
        }

        // Les voisins
        if !self.voisins.is_empty() {
            html.push_str("<div class=\"card\"><h2>🏘️ Les nodes du village</h2>");
            for v in &self.voisins {
                html.push_str(&format!(r#"<div class="tx">📡 <b>{}</b> — <span style="font-family:monospace;color:#7fcf7f;">{}:{}</span></div>"#,
                    v.nom, v.ip, v.port));
            }
            html.push_str("</div>");
        }

        // Les vidéos
        if self.videos.is_empty() {
            html.push_str(r#"<div class="card"><p style="text-align:center;color:#a8c5a8;">Aucune empreinte encore sur la chaîne. Dépose la première vidéo du village ! 📺</p></div>"#);
        } else {
            let mut triees: Vec<&VideoTube> = self.videos.iter().collect();
            triees.sort_by(|a, b| b.heure.cmp(&a.heure));
            for v in triees {
                let ko = v.taille / 1024;
                html.push_str(&format!(r#"<div class="card"><h2>📺 {}</h2><div id="tube-{}"></div><p style="color:#a8c5a8;">👤 <b>{}</b> — 👁️ {} vues — 💾 {} Ko</p><p>{}</p><p style="color:#d4a437;font-size:0.9em;">🔗 Empreinte : <span style="font-family:monospace;">{}</span></p><p style="color:#7fcf7f;">🏠 La vidéo dort chez <b>{}</b> — <span style="font-family:monospace;">{}:{}</span></p><div style="display:flex;gap:8px;"><button onclick="tubeJouer({},{}, '{}', '{}', '{}')">▶️ Chercher chez le voisin</button><form method="POST" action="/afritube/voir"><input type="hidden" name="id" value="{}"><button>👁️ Vue comptée</button></form></div></div>"#,
                    v.titre, v.id, v.auteur, v.vues, ko, v.description, v.empreinte,
                    v.hote_nom, v.hote_ip, v.hote_port, v.id, v.id, v.fichier, v.hote_ip, v.hote_port, v.id));
            }
        }

        // Le lecteur magique : local d'abord, voisin ensuite — MODE AVION
        html.push_str(r#"<script>
function tubeJouer(id, vid, fichier, ip, port) {
  var box = document.getElementById('tube-' + vid);
  // 1. Essayer CE serveur (la vidéo est peut-être ici)
  var local = document.createElement('video');
  local.controls = true;
  local.preload = 'metadata';
  local.style = 'width:100%;border-radius:8px;max-height:400px;';
  local.src = '/plante/media?f=' + encodeURIComponent(fichier);
  local.onerror = function() {
    // 2. La vidéo n'est pas ici → aller la chercher CHEZ LE VOISIN en wifi local
    var v = document.createElement('video');
    v.controls = true;
    v.preload = 'metadata';
    v.style = 'width:100%;border-radius:8px;max-height:400px;';
    v.src = 'http://' + ip + ':' + port + '/plante/media?f=' + encodeURIComponent(fichier);
    v.onerror = function() {
      box.innerHTML = '<p style="color:#cf7f7f;">⚠️ La vidéo dort chez ' + ip + ' mais ce node ne répond pas. Allume le node du voisin ou connecte-toi au même wifi. ✈️</p>';
    };
    box.innerHTML = '';
    box.appendChild(v);
    v.play().catch(function(){});
  };
  box.innerHTML = '';
  box.appendChild(local);
  local.play().catch(function(){});
}
</script>"#);
        html
    }
}
