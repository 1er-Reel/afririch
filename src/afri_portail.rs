/// v2.22 — LE PORTAIL DU CONTINENT 🚪🌍
/// Tous les téléphones du village passent par ici.
/// L'Afrique décide ce qui passe. L'Occident négocie égal à égal.
/// Œuvre originale de Koffi Christ Olivier — Licence AFRI-OSL v1.0.
use crate::afri_json::{JsonValue, from_str, to_string};
use crate::afri_dns::DnsStore;
use std::collections::HashMap;

/// Un service occidental qui veut passer — doit payer en AFR.
pub struct ServiceOccidental {
    pub domaine: String,      // ex: "google.com"
    pub ip_autorisee: String, // l'IP du vrai serveur (si on laisse passer)
    pub tarif_afr: u64,       // combien ça coûte en AFR par mois
    pub actif: bool,         // est-ce qu'on le laisse passer ?
    pub bloc: u64,           // bloc où l'autorisation est gravée
}

/// Le registre des services occidentaux.
pub struct PortailStore {
    pub services: Vec<ServiceOccidental>,
    pub chemin: String,
}

impl PortailStore {
    pub fn load() -> Self {
        let chemin = crate::data_path("portail.json");
        let mut s = PortailStore { services: Vec::new(), chemin: chemin.clone() };
        if let Ok(data) = std::fs::read_to_string(&chemin) {
            if let Ok(v) = from_str(&data) {
                if let Some(arr) = v.as_array() {
                    for so in arr.iter().filter_map(|x| x.as_object()) {
                        s.services.push(ServiceOccidental {
                            domaine: so.get("domaine").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                            ip_autorisee: so.get("ip_autorisee").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                            tarif_afr: so.get("tarif_afr").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                            actif: so.get("actif").and_then(|x| x.as_bool()).unwrap_or(false),
                            bloc: so.get("bloc").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                        });
                    }
                }
            }
        }
        s
    }

    pub fn save(&self) {
        let arr: Vec<JsonValue> = self.services.iter().map(|s| {
            let mut m = HashMap::new();
            m.insert("domaine".to_string(), JsonValue::Str(s.domaine.clone()));
            m.insert("ip_autorisee".to_string(), JsonValue::Str(s.ip_autorisee.clone()));
            m.insert("tarif_afr".to_string(), JsonValue::Int(s.tarif_afr as i64));
            m.insert("actif".to_string(), JsonValue::Bool(s.actif));
            m.insert("bloc".to_string(), JsonValue::Int(s.bloc as i64));
            JsonValue::Object(m)
        }).collect();
        let _ = std::fs::write(&self.chemin, to_string(&JsonValue::Array(arr)));
    }

    /// Ajouter un service occidental — l'Afrique négocie.
    pub fn negocier(&mut self, domaine: &str, ip: &str, tarif: u64, bloc: u64) {
        if let Some(s) = self.services.iter_mut().find(|s| s.domaine == domaine) {
            s.tarif_afr = tarif;
            s.bloc = bloc;
            self.save();
            return;
        }
        self.services.push(ServiceOccidental {
            domaine: domaine.to_lowercase(),
            ip_autorisee: ip.to_string(),
            tarif_afr: tarif,
            actif: false, // inactif par défaut — l'Afrique décide
            bloc,
        });
        self.save();
    }

    /// Activer un service — l'Afrique accepte le paiement.
    pub fn activer(&mut self, domaine: &str) {
        if let Some(s) = self.services.iter_mut().find(|s| s.domaine == domaine.to_lowercase()) {
            s.actif = true;
            self.save();
        }
    }

    /// Désactiver un service — l'Afrique ferme la porte.
    pub fn desactiver(&mut self, domaine: &str) {
        if let Some(s) = self.services.iter_mut().find(|s| s.domaine == domaine.to_lowercase()) {
            s.actif = false;
            self.save();
        }
    }

    /// Est-ce que ce domaine est autorisé à passer ?
    pub fn est_autorise(&self, domaine: &str) -> Option<&ServiceOccidental> {
        let d = domaine.trim_end_matches('.').to_lowercase();
        self.services.iter().find(|s| s.domaine == d && s.actif)
    }

    /// v2.23 — LE MATCHING SANS DÉBOUCHÉ 🚨
    /// "www.google.fr", "mail.google.com", "google.co.uk" → tous attrapés
    /// par la porte "google". Aucun développeur occidental ne contourne
    /// le portail par un sous-domaine ou une extension nationale.
    pub fn est_autorise_par_racine(&self, domaine: &str) -> Option<&ServiceOccidental> {
        let racine = crate::afri_dns::racine_du_domaine(domaine);
        self.services.iter().find(|s| {
            s.actif && (
                s.domaine == domaine.trim_end_matches('.').to_lowercase()
                || crate::afri_dns::racine_du_domaine(&s.domaine) == racine
            )
        })
    }

    /// La page du portail — ce que voit le village.
    pub fn html_page(&self, dns: &DnsStore, msg: Option<&str>) -> String {
        let mut html = String::new();
        html.push_str("<h1>🚪 LE PORTAIL DU CONTINENT</h1>");
        html.push_str(r#"<p style="text-align:center;color:#a8c5a8;">Les câbles occidentaux dorment sur <b style="color:#d4a437;">notre terre et notre mer</b>. Ils nous vendaient notre propre connexion. Aujourd'hui, <b style="color:#25D366;">l'Afrique décide ce qui passe</b>. Ils viendront négocier égal à égal. 🦁</p>"#);
        html.push_str(r#"<div class="nav"><a href="/">← Accueil</a> | <a href="/dns">🌐 AfriDNS</a> | <a href="/dashboard">👑 Admin</a></div>"#);
        if let Some(m) = msg {
            html.push_str(&format!(r#"<div class="msg">{}</div>"#, m));
        }

        // Les domaines africains — libres
        html.push_str("<div class=\"card\"><h2>💚 Les domaines de l'Afrique — LIBRES</h2>");
        if dns.registre.is_empty() {
            html.push_str("<p style=\"color:#a8c5a8;\">Aucun domaine enregistré encore.</p>");
        } else {
            for e in &dns.registre {
                html.push_str(&format!(r#"<div class="tx">🌐 <b style="color:#25D366;">.{}</b> → {} <span style="color:#a8c5a8;">(propriétaire: {})</span></div>"#,
                    e.domaine, e.ip, e.proprietaire));
            }
        }
        html.push_str("</div>");

        // Les services occidentaux — l'Afrique décide
        html.push_str("<div class=\"card\"><h2>🌍 Les services occidentaux — L'AFRIQUE DÉCIDE</h2>");
        html.push_str(r#"<p style="color:#a8c5a8;">Chaque service doit payer en AFR. L'Afrique ouvre ou ferme la porte. Égal à égal.</p>"#);
        if self.services.is_empty() {
            html.push_str("<p style=\"color:#a8c5a8;\">Aucun service occidental négocié encore.</p>");
        } else {
            for s in &self.services {
                let status = if s.actif { "✅ AUTORISÉ" } else { "🔒 FERMÉ" };
                let couleur = if s.actif { "#25D366" } else { "#cf7f7f" };
                let action = if s.actif {
                    format!(r#"<form method="POST" action="/portail/fermer" style="display:inline;"><input type="hidden" name="domaine" value="{}"><button>🔒 Fermer la porte</button></form>"#, s.domaine)
                } else {
                    format!(r#"<form method="POST" action="/portail/ouvrir" style="display:inline;"><input type="hidden" name="domaine" value="{}"><button>✅ Ouvrir la porte</button></form>"#, s.domaine)
                };
                html.push_str(&format!(r#"<div class="tx">🌐 <b>{}</b> — {} AFR/mois — <b style="color:{};">{}</b> {}</div>"#,
                    s.domaine, s.tarif_afr, couleur, status, action));
            }
        }
        html.push_str("</div>");

        // Formulaire pour négocier (admin seulement)
        html.push_str(r#"<div class="card"><h2>🤝 Négocier avec l'Occident (Admin)</h2><form method="POST" action="/portail/negocier"><label>Domaine occidental :</label><input name="domaine" placeholder="ex: google.com" maxlength="60" required><label>IP du serveur (si on laisse passer) :</label><input name="ip" placeholder="ex: 142.250.1.1" maxlength="40"><label>Tarif en AFR/mois :</label><input name="tarif" type="number" min="0" max="1000000" value="1000" required><button>🤝 Négocier</button></form></div>"#);

        html
    }

    /// v2.24 — LA PAGE CAPTIVE 🔒
    /// Le téléphone du village demande un service non-payé → il tombe ICI.
    /// Comme le portail captif d'un hôtel — mais africain.
    pub fn html_captif(&self, domaine: &str) -> String {
        format!(r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>🔒 Le Portail du Continent</title><style>body{{font-family:sans-serif;background:#0d1f17;color:#f5e9d4;padding:24px;margin:0 auto;max-width:600px;text-align:center;}}h1{{color:#d4a437;}}h2{{color:#25D366;}}.ferme{{color:#cf7f7f;font-size:1.4em;font-weight:bold;}}a{{color:#d4a437;}}button{{background:#d4a437;color:#0d1f17;border:none;padding:14px 28px;border-radius:8px;font-size:1.1em;font-weight:bold;cursor:pointer;}}</style></head><body>
<h1>🚪 LE PORTAIL DU CONTINENT</h1>
<p class="ferme">🔒 {}</p>
<p style="font-size:1.1em;">Ce service occidental <b>n'a pas payé l'Afrique</b>.</p>
<p style="color:#a8c5a8;">Son câble dort sur <b>notre terre et notre mer</b> sans récompenser les villageois. Comment envoyer nos enfants à l'école ? Comment construire des écoles pour nos enfants ?</p>
<p style="color:#a8c5a8;">L'Afrique ne demande plus la permission. <b>L'Afrique décide ce qui passe.</b> Ils viendront négocier <b>égal à égal</b>. 🦁</p>
<h2>💚 Les services LIBRES du village</h2>
<p style="color:#a8c5a8;">Pendant que l'Occident négocie, l'Afrique vit :</p>
<p style="line-height:2.2;">
<a href="/">🏠 AfriChain</a> ·
<a href="/sahara">🔍 SAHARA AFRI</a> ·
<a href="/video">🎬 Afri Vidéo</a> ·
<a href="/afritube">📺 AfriTube</a> ·
<a href="/noires">💬 LES NOIRES</a> ·
<a href="/plante">🌱 Planté Verte</a>
</p>
<p><a href="/portail"><button>🚪 Voir le portail du continent</button></a></p>
<footer style="margin-top:30px;border-top:1px solid #d4a437;padding-top:12px;font-size:0.85em;color:#a8c5a8;">◈ AfriChain — Œuvre originale de Koffi Christ Olivier — Licence AFRI-OSL v1.0<br/>Le petit 0.000 Go commande les gros 28 Go.</footer>
</body></html>"#, domaine)
    }
}

/// Lancer le portail captif sur le port 8080 (redirige vers AfriChain).
pub fn lancer_portail(_store: std::sync::Arc<std::sync::Mutex<PortailStore>>) {
    // Le portail captif est le serveur HTTP principal (déjà lancé dans main.rs)
    // Cette fonction est un placeholder pour un futur proxy HTTPS complet.
    println!("🚪 PORTAIL DU CONTINENT — l'Afrique contrôle les portes.");
}
