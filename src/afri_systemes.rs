/// v2.05 — LE PLAY STORE DES SYSTÈMES 🦁 + AGENTS WARI + PASSEPORT 54 + COFFRE DES INTELLIGENTS
/// Œuvre originale de KOFFI CHRIST OLIVIER (Côte d'Ivoire) — Licence AFRI-OSL v1.0.
/// Le Chef seul distribue et active. Pas un code sans l'autorisation de la blockchain.
use crate::afri_json::{JsonValue, from_str, to_string};
use std::collections::HashMap;

/// Une app SYSTÈME — un module de la plateforme, contrôlé par le Chef seul.
pub struct AppSysteme {
    pub id: String,          // "SYS-1"
    pub nom: String,         // "Planté Verte (Facebook)"
    pub emoji: String,
    pub description: String,
    pub route: String,       // "/plante"
    pub active: bool,        // le Chef seul décide
    pub bloc_activation: u64, // bloc où l'état a changé
}

/// Un agent Wari — la personne physique qui encaisse/distribue le cash.
/// Deux types : AFRI (Afri.Wari) et AES (AES.Wari). Activé par le Chef seul.
pub struct AgentWari {
    pub username: String,
    pub type_wari: String,   // "AFRI" ou "AES"
    pub nom_complet: String,
    pub telephone: String,
    pub pays: String,
    pub actif: bool,
    pub bloc: u64,
    pub heure: u64,
}

/// Le Passeport Africain — un seul passeport pour les 54 pays.
pub struct Passeport {
    pub username: String,
    pub nom_complet: String,
    pub pays: String,
    pub date_naissance: String,
    pub numero: String,      // AF54-CI-482913
    pub delivre: bool,       // délivré par le Chef
    pub bloc: u64,
    pub heure: u64,
}

/// Une app du Coffre des Intelligents — publiée par un développeur, validée par le Chef.
pub struct AppCoffre {
    pub id: u64,
    pub nom: String,
    pub emoji: String,
    pub description: String,
    pub auteur: String,
    pub version: String,
    pub statut: String,      // "attente" / "valide" / "refuse"
    pub heure: u64,
}

/// Le store des systèmes — persisté dans systemes.json (gitignoré).
pub struct SystemeStore {
    pub apps: Vec<AppSysteme>,
    pub agents: Vec<AgentWari>,
    pub passeports: Vec<Passeport>,
    pub coffre: Vec<AppCoffre>,
    pub next_coffre: u64,
    pub chemin: String,
}

impl SystemeStore {
    pub fn load() -> Self {
        let chemin = crate::data_path("systemes.json");
        let mut s = SystemeStore { apps: Vec::new(), agents: Vec::new(), passeports: Vec::new(), coffre: Vec::new(), next_coffre: 1, chemin: chemin.clone() };
        if let Ok(data) = std::fs::read_to_string(&chemin) {
            if let Ok(v) = from_str(&data) {
                if let Some(o) = v.as_object() {
                    if let Some(arr) = o.get("apps").and_then(|a| a.as_array()) {
                        for a in arr {
                            if let Some(ao) = a.as_object() {
                                s.apps.push(AppSysteme {
                                    id: ao.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    nom: ao.get("nom").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    emoji: ao.get("emoji").and_then(|x| x.as_str()).unwrap_or("📱").to_string(),
                                    description: ao.get("description").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    route: ao.get("route").and_then(|x| x.as_str()).unwrap_or("/").to_string(),
                                    active: ao.get("active").and_then(|x| x.as_i64()).unwrap_or(0) == 1,
                                    bloc_activation: ao.get("bloc_activation").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                });
                            }
                        }
                    }
                    if let Some(arr) = o.get("agents").and_then(|a| a.as_array()) {
                        for a in arr {
                            if let Some(ao) = a.as_object() {
                                s.agents.push(AgentWari {
                                    username: ao.get("username").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    type_wari: ao.get("type_wari").and_then(|x| x.as_str()).unwrap_or("AFRI").to_string(),
                                    nom_complet: ao.get("nom_complet").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    telephone: ao.get("telephone").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    pays: ao.get("pays").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    actif: ao.get("actif").and_then(|x| x.as_i64()).unwrap_or(0) == 1,
                                    bloc: ao.get("bloc").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                    heure: ao.get("heure").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                });
                            }
                        }
                    }
                    if let Some(arr) = o.get("passeports").and_then(|a| a.as_array()) {
                        for a in arr {
                            if let Some(ao) = a.as_object() {
                                s.passeports.push(Passeport {
                                    username: ao.get("username").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    nom_complet: ao.get("nom_complet").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    pays: ao.get("pays").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    date_naissance: ao.get("date_naissance").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    numero: ao.get("numero").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    delivre: ao.get("delivre").and_then(|x| x.as_i64()).unwrap_or(0) == 1,
                                    bloc: ao.get("bloc").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                    heure: ao.get("heure").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                });
                            }
                        }
                    }
                    if let Some(arr) = o.get("coffre").and_then(|a| a.as_array()) {
                        for a in arr {
                            if let Some(ao) = a.as_object() {
                                s.coffre.push(AppCoffre {
                                    id: ao.get("id").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                    nom: ao.get("nom").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    emoji: ao.get("emoji").and_then(|x| x.as_str()).unwrap_or("📦").to_string(),
                                    description: ao.get("description").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    auteur: ao.get("auteur").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    version: ao.get("version").and_then(|x| x.as_str()).unwrap_or("1.0").to_string(),
                                    statut: ao.get("statut").and_then(|x| x.as_str()).unwrap_or("attente").to_string(),
                                    heure: ao.get("heure").and_then(|x| x.as_i64()).unwrap_or(0) as u64,
                                });
                            }
                        }
                    }
                    s.next_coffre = o.get("next_coffre").and_then(|x| x.as_i64()).unwrap_or(1) as u64;
                }
            }
        }
        // Première fois : installer les apps système par défaut — TOUTES ACTIVES, le Chef peut couper
        if s.apps.is_empty() {
            s.apps = Self::apps_defaut();
            s.save();
        }
        s
    }

    /// Les apps officielles de la plateforme — le catalogue du Play Store des Systèmes.
    fn apps_defaut() -> Vec<AppSysteme> {
        let defs = [
            ("Planté Verte (Facebook)", "🌱", "Le réseau social africain — posts, photos, stories, likes.", "/plante"),
            ("LES NOIRES (WhatsApp)", "💬", "Messagerie africaine — bulles vertes, codes de compte, zéro puce.", "/noires"),
            ("Afri Vidéo (YouTube)", "🎬", "Notre YouTube — publier et regarder les vidéos de l'Afrique.", "/video"),
            ("SAHARA (Google)", "🔍", "La mémoire du continent — l'histoire vraie des 54 pays.", "/sahara"),
            ("Afri Télégram (Telegram)", "📢", "Canaux et annonces — le Telegram africain.", "/afri-telegram"),
            ("AFRI STORE (Play Store)", "🏪", "Le Play Store africain — apps des développeurs d'Afrique.", "/store"),
            ("Appels Verts", "📞", "Appeler sans opérateur — de puce à puce sur AfriChain.", "/appels"),
            ("SMS Verts", "💬", "SMS sans opérateur — le réseau Planète Verte.", "/sms"),
            ("Afri.Wari #144#", "🏦", "La banque mobile africaine — dépôts, retraits, QR.", "/wari"),
            ("Banque des 54 États", "🏛️", "Une Grande Banque par pays + sous-banques de villes.", "/banque"),
            ("Le Lion du Sahel (jeu)", "🦁", "Tâches quotidiennes, graines, quiz culturel — gagne des AFR.", "/lion"),
            ("Afri Sites", "🏗️", "Crée ton site web hébergé sur la blockchain.", "/sites"),
            ("Coffre des Intelligents", "🧠", "Les développeurs publient — le Chef valide.", "/coffre"),
            ("Passeport 54 Pays", "🪪", "Un seul passeport pour toute l'Afrique.", "/passeport"),
            ("Internet Afri", "🌐", "Notre internet — protocole machine, port 8181.", "/internet"),
            ("Navigateur Souverain", "🌐", "Naviguer DANS AfriChain — jamais Google.", "/navigateur"),
            ("Annuaire du Continent", "📖", "L'annuaire des 54 pays — numéros verts.", "/annuaire"),
            ("L'Étincelle", "🔥", "Le foyer qui survit à la coupure du monde.", "/etincelle"),
            ("Amion Blandine", "💚", "Le terminal en langage machine — hommage à la mère du Chef.", "/amion"),
            ("Langage AMION", "▤", "Notre langage de programmation — DIRE, SOIT, SI, ENVOIE, MINE.", "/langage"),
            ("Le Sceau (AFRI-OSL)", "◈", "La licence souveraine — le nom du Chef gravé à jamais.", "/sceau"),
            ("Afri Rich Dépôt", "💰", "Épargne libre + retraite verrouillée +25%.", "/afri-rich"),
        ];
        defs.iter().enumerate().map(|(i, (nom, emoji, description, route))| AppSysteme {
            id: format!("SYS-{}", i + 1),
            nom: nom.to_string(),
            emoji: emoji.to_string(),
            description: description.to_string(),
            route: route.to_string(),
            active: true,
            bloc_activation: 0,
        }).collect()
    }

    pub fn save(&self) {
        let mut m = HashMap::new();
        let apps: Vec<JsonValue> = self.apps.iter().map(|a| {
            let mut am = HashMap::new();
            am.insert("id".to_string(), JsonValue::Str(a.id.clone()));
            am.insert("nom".to_string(), JsonValue::Str(a.nom.clone()));
            am.insert("emoji".to_string(), JsonValue::Str(a.emoji.clone()));
            am.insert("description".to_string(), JsonValue::Str(a.description.clone()));
            am.insert("route".to_string(), JsonValue::Str(a.route.clone()));
            am.insert("active".to_string(), JsonValue::Int(if a.active { 1 } else { 0 }));
            am.insert("bloc_activation".to_string(), JsonValue::Int(a.bloc_activation as i64));
            JsonValue::Object(am)
        }).collect();
        m.insert("apps".to_string(), JsonValue::Array(apps));
        let agents: Vec<JsonValue> = self.agents.iter().map(|a| {
            let mut am = HashMap::new();
            am.insert("username".to_string(), JsonValue::Str(a.username.clone()));
            am.insert("type_wari".to_string(), JsonValue::Str(a.type_wari.clone()));
            am.insert("nom_complet".to_string(), JsonValue::Str(a.nom_complet.clone()));
            am.insert("telephone".to_string(), JsonValue::Str(a.telephone.clone()));
            am.insert("pays".to_string(), JsonValue::Str(a.pays.clone()));
            am.insert("actif".to_string(), JsonValue::Int(if a.actif { 1 } else { 0 }));
            am.insert("bloc".to_string(), JsonValue::Int(a.bloc as i64));
            am.insert("heure".to_string(), JsonValue::Int(a.heure as i64));
            JsonValue::Object(am)
        }).collect();
        m.insert("agents".to_string(), JsonValue::Array(agents));
        let passeports: Vec<JsonValue> = self.passeports.iter().map(|p| {
            let mut pm = HashMap::new();
            pm.insert("username".to_string(), JsonValue::Str(p.username.clone()));
            pm.insert("nom_complet".to_string(), JsonValue::Str(p.nom_complet.clone()));
            pm.insert("pays".to_string(), JsonValue::Str(p.pays.clone()));
            pm.insert("date_naissance".to_string(), JsonValue::Str(p.date_naissance.clone()));
            pm.insert("numero".to_string(), JsonValue::Str(p.numero.clone()));
            pm.insert("delivre".to_string(), JsonValue::Int(if p.delivre { 1 } else { 0 }));
            pm.insert("bloc".to_string(), JsonValue::Int(p.bloc as i64));
            pm.insert("heure".to_string(), JsonValue::Int(p.heure as i64));
            JsonValue::Object(pm)
        }).collect();
        m.insert("passeports".to_string(), JsonValue::Array(passeports));
        let coffre: Vec<JsonValue> = self.coffre.iter().map(|c| {
            let mut cm = HashMap::new();
            cm.insert("id".to_string(), JsonValue::Int(c.id as i64));
            cm.insert("nom".to_string(), JsonValue::Str(c.nom.clone()));
            cm.insert("emoji".to_string(), JsonValue::Str(c.emoji.clone()));
            cm.insert("description".to_string(), JsonValue::Str(c.description.clone()));
            cm.insert("auteur".to_string(), JsonValue::Str(c.auteur.clone()));
            cm.insert("version".to_string(), JsonValue::Str(c.version.clone()));
            cm.insert("statut".to_string(), JsonValue::Str(c.statut.clone()));
            cm.insert("heure".to_string(), JsonValue::Int(c.heure as i64));
            JsonValue::Object(cm)
        }).collect();
        m.insert("coffre".to_string(), JsonValue::Array(coffre));
        m.insert("next_coffre".to_string(), JsonValue::Int(self.next_coffre as i64));
        let _ = std::fs::write(&self.chemin, to_string(&JsonValue::Object(m)));
    }

    /// Basculer une app système — retourne (nouvel_état, nom) pour graver sur la blockchain.
    pub fn basculer(&mut self, id: &str) -> Option<(bool, String)> {
        if let Some(a) = self.apps.iter_mut().find(|a| a.id == id) {
            a.active = !a.active;
            let r = (a.active, a.nom.clone());
            self.save();
            return Some(r);
        }
        None
    }

    /// Un module est-il actif ? (le Chef seul décide)
    pub fn est_actif(&self, route: &str) -> bool {
        self.apps.iter().find(|a| a.route == route).map(|a| a.active).unwrap_or(true)
    }

    /// Créer un agent Wari — activé par le Chef seul.
    pub fn creer_agent(&mut self, username: &str, type_wari: &str, nom_complet: &str, telephone: &str, pays: &str) {
        if self.agents.iter().any(|a| a.username == username) { return; }
        self.agents.push(AgentWari {
            username: username.to_string(), type_wari: type_wari.to_string(),
            nom_complet: nom_complet.to_string(), telephone: telephone.to_string(),
            pays: pays.to_string(), actif: false, bloc: 0,
            heure: crate::now_timestamp() as u64,
        });
        self.save();
    }

    /// Activer/désactiver un agent — le Chef seul. Retourne (actif, type, username).
    pub fn basculer_agent(&mut self, username: &str) -> Option<(bool, String, String)> {
        if let Some(a) = self.agents.iter_mut().find(|a| a.username == username) {
            a.actif = !a.actif;
            let r = (a.actif, a.type_wari.clone(), a.username.clone());
            self.save();
            return Some(r);
        }
        None
    }

    /// L'agent connecté est-il actif ?
    pub fn agent_actif(&self, username: &str) -> Option<&AgentWari> {
        self.agents.iter().find(|a| a.username == username && a.actif)
    }

    /// Demander un passeport (l'utilisateur demande, le Chef délivre).
    pub fn demander_passeport(&mut self, username: &str, nom_complet: &str, pays: &str, date_naissance: &str) {
        if self.passeports.iter().any(|p| p.username == username) { return; }
        self.passeports.push(Passeport {
            username: username.to_string(), nom_complet: nom_complet.to_string(),
            pays: pays.to_string(), date_naissance: date_naissance.to_string(),
            numero: String::new(), delivre: false, bloc: 0,
            heure: crate::now_timestamp() as u64,
        });
        self.save();
    }

    /// Délivrer le passeport — le Chef seul. Numéro unique gravé sur la blockchain.
    pub fn delivrer_passeport(&mut self, username: &str) -> Option<(String, String)> {
        if let Some(p) = self.passeports.iter_mut().find(|p| p.username == username && !p.delivre) {
            let code = p.pays.chars().take(2).collect::<String>().to_uppercase();
            let n = crate::afri_rng::random_u64() % 900_000 + 100_000;
            p.numero = format!("AF54-{}-{}", code, n);
            p.delivre = true;
            let r = (p.numero.clone(), p.nom_complet.clone());
            self.save();
            return Some(r);
        }
        None
    }

    /// Publier dans le Coffre des Intelligents (développeur) — statut attente.
    pub fn publier_coffre(&mut self, nom: &str, emoji: &str, description: &str, auteur: &str, version: &str) -> u64 {
        let id = self.next_coffre;
        self.next_coffre += 1;
        self.coffre.push(AppCoffre {
            id, nom: nom.to_string(), emoji: emoji.to_string(),
            description: description.to_string(), auteur: auteur.to_string(),
            version: version.to_string(), statut: "attente".to_string(),
            heure: crate::now_timestamp() as u64,
        });
        self.save();
        id
    }

    /// Valider ou refuser une app du coffre — le Chef seul.
    pub fn statut_coffre(&mut self, id: u64, statut: &str) -> Option<(String, String, String)> {
        if let Some(c) = self.coffre.iter_mut().find(|c| c.id == id) {
            c.statut = statut.to_string();
            let r = (c.nom.clone(), c.auteur.clone(), statut.to_string());
            self.save();
            return Some(r);
        }
        None
    }
}
