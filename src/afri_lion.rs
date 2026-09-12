// ===== AFRI LION — Le Lion du Sahel: tâches + épargne + retraite =====
// v1.64 — Comme Hamster, mais pour nous c'est le Lion. 🦁
// v1.66 — LA MONNAIE INTELLIGENTE: 1 AFR = 1 million de dollars.
// Les récompenses sont en GRAINES: 1 AFR = 100 000 000 graines.
// 1 graine = 0.00000001 AFR = 0.01 $. Tout petit, comme le veut le chef.
// Quand tes graines atteignent 1 AFR → vraie transaction blockchain.
// AfriRich Centre de Dépôt: Épargne Libre + Retraite (+25% après 6 mois).

use std::collections::HashMap;
use crate::afri_json::{JsonValue, to_string, from_str};
use crate::afri_time::now_timestamp;

/// 1 AFR = 100 000 000 graines (le micro-unité de l'Afrique)
pub const GRAINES_PAR_AFR: u64 = 100_000_000;

/// Afficher des graines en AFR décimal: 10 graines = "0.00000010 AFR"
pub fn format_graines(g: u64) -> String {
    let entiers = g / GRAINES_PAR_AFR;
    let reste = g % GRAINES_PAR_AFR;
    format!("{}.{:08} AFR", entiers, reste)
}

/// Une tâche du Lion — à faire chaque jour, comme Hamster Kombat
#[derive(Clone, Debug)]
pub struct TacheLion {
    pub id: String,
    pub titre: String,
    pub description: String,
    pub gain: u64,          // GRAINES gagnées (v1.66)
    pub icone: String,      // emoji
}

/// Les 8 tâches quotidiennes du Lion — gains en graines (1 graine = $0.01)
pub fn taches_du_jour() -> Vec<TacheLion> {
    vec![
        TacheLion { id: "salut".into(), titre: "Saluer le Lion".into(), description: "Dire bonjour à la blockchain africaine. Le Lion répond.".into(), gain: 10, icone: "🦁".into() },
        TacheLion { id: "pays".into(), titre: "Connaître les 54 pays".into(), description: "Visiter l'annuaire des 54 pays africains.".into(), gain: 25, icone: "🌍".into() },
        TacheLion { id: "annuaire".into(), titre: "Voir l'annuaire".into(), description: "Regarder qui est connecté sur le réseau panafricain.".into(), gain: 25, icone: "📖".into() },
        TacheLion { id: "wallet".into(), titre: "Vérifier son wallet".into(), description: "Consulter son solde AfriRich.".into(), gain: 10, icone: "👛".into() },
        TacheLion { id: "envoi".into(), titre: "Envoyer des AFR".into(), description: "Faire une transaction vers un autre Africain.".into(), gain: 50, icone: "📤".into() },
        TacheLion { id: "mine".into(), titre: "Miner un bloc".into(), description: "Laisser le soleil du Sahel valider un bloc.".into(), gain: 40, icone: "⛏️".into() },
        TacheLion { id: "histoire".into(), titre: "Apprendre l'histoire".into(), description: "Lire une page de l'Académie AI Griot (histoire de l'Afrique).".into(), gain: 40, icone: "📜".into() },
        TacheLion { id: "partage".into(), titre: "Connecter un ami".into(), description: "Partager le QR de connexion à un ami.".into(), gain: 60, icone: "🔗".into() },
    ]
}

/// L'état d'un joueur: tâches accomplies aujourd'hui + série (streak)
#[derive(Clone, Debug)]
pub struct EtatLion {
    pub dernier_jour: i64,                    // timestamp du jour (86400 * n) du dernier jeu
    pub taches_faites: Vec<String>,          // ids des tâches accomplies aujourd'hui
    pub serie: u32,                          // jours consécutifs joués
    pub total_gagne: u64,                    // total GRAINES gagnées via le Lion (v1.66)
    pub graines: u64,                        // graines en attente de conversion en AFR (v1.66)
    pub quiz_reussis: u32,                   // questions culturelles réussies (v1.66)
    pub quiz_rates: u32,                     // questions ratées (v1.66)
}

impl EtatLion {
    pub fn nouveau() -> Self {
        EtatLion { dernier_jour: 0, taches_faites: Vec::new(), serie: 0, total_gagne: 0, graines: 0, quiz_reussis: 0, quiz_rates: 0 }
    }

    pub fn jour_actuel() -> i64 {
        now_timestamp() / 86400
    }

    /// Nouveau jour ? On remet les tâches à faire et on calcule la série
    pub fn rafraichir_jour(&mut self) {
        let auj = Self::jour_actuel();
        if self.dernier_jour != auj {
            if self.dernier_jour == auj - 1 {
                self.serie += 1; // jour consécutif
            } else if self.dernier_jour < auj - 1 {
                self.serie = 1;  // série cassée, on repart à 1
            }
            if self.dernier_jour == 0 { self.serie = 1; }
            self.taches_faites.clear();
            self.dernier_jour = auj;
        }
    }

    /// Bonus de série: +5 graines par jour consécutif (max +50 graines = $0.50)
    pub fn bonus_serie(&self) -> u64 {
        (self.serie.saturating_sub(1)) as u64 * 5
    }

    /// Combien d'AFR entiers tes graines peuvent donner (conversion possible)
    pub fn afr_convertibles(&self) -> u64 {
        self.graines / GRAINES_PAR_AFR
    }
}

/// Un dépôt AfriRich (épargne ou retraite)
#[derive(Clone, Debug)]
pub struct Depot {
    pub montant: u64,       // AFR déposés
    pub date: i64,          // timestamp du dépôt
    pub kind: String,       // "epargne" | "retraite"
}

/// Le Centre de Dépôt AfriRich d'un utilisateur
#[derive(Clone, Debug)]
pub struct CompteDepot {
    pub epargne: Vec<Depot>,    // épargne libre: retirable quand on veut
    pub retraite: Vec<Depot>,   // retraite: verrouillée, +25% après 6 mois
}

impl CompteDepot {
    pub fn nouveau() -> Self {
        CompteDepot { epargne: Vec::new(), retraite: Vec::new() }
    }

    pub fn total_epargne(&self) -> u64 { self.epargne.iter().map(|d| d.montant).sum() }
    pub fn total_retraite(&self) -> u64 { self.retraite.iter().map(|d| d.montant).sum() }

    /// La retraite verrouillée depuis ≥ 6 mois gagne +25%
    pub fn retraite_debloquable(&self) -> (u64, u64) {
        let six_mois = 6 * 30 * 86400;
        let maintenant = now_timestamp();
        let mut verrouille = 0u64;
        let mut pret = 0u64;
        for d in &self.retraite {
            if maintenant - d.date >= six_mois {
                pret += d.montant + (d.montant / 4); // +25%
            } else {
                verrouille += d.montant;
            }
        }
        (pret, verrouille)
    }
}

/// Le magasin central de tous les Lions: username → état
pub struct LionStore {
    pub etats: HashMap<String, EtatLion>,
    pub depots: HashMap<String, CompteDepot>,
}

impl LionStore {
    pub fn nouveau() -> Self {
        LionStore { etats: HashMap::new(), depots: HashMap::new() }
    }

    /// Charger depuis lion.json (persistance)
    pub fn charger(chemin: &str) -> Self {
        if let Ok(data) = std::fs::read_to_string(chemin) {
            if let Ok(v) = from_str(&data) {
                return Self::from_json(&v);
            }
        }
        Self::nouveau()
    }

    pub fn sauvegarder(&self, chemin: &str) {
        let _ = std::fs::write(chemin, to_string(&self.to_json()));
    }

    pub fn to_json(&self) -> JsonValue {
        let mut root = HashMap::new();
        // états
        let mut etats = HashMap::new();
        for (user, e) in &self.etats {
            let mut m = HashMap::new();
            m.insert("dernier_jour".to_string(), JsonValue::Int(e.dernier_jour));
            m.insert("serie".to_string(), JsonValue::Int(e.serie as i64));
            m.insert("total_gagne".to_string(), JsonValue::Int(e.total_gagne as i64));
            m.insert("graines".to_string(), JsonValue::Int(e.graines as i64));
            m.insert("quiz_reussis".to_string(), JsonValue::Int(e.quiz_reussis as i64));
            m.insert("quiz_rates".to_string(), JsonValue::Int(e.quiz_rates as i64));
            m.insert("taches_faites".to_string(), JsonValue::Array(
                e.taches_faites.iter().map(|t| JsonValue::Str(t.clone())).collect()));
            etats.insert(user.clone(), JsonValue::Object(m));
        }
        root.insert("etats".to_string(), JsonValue::Object(etats));
        // dépôts
        let mut depots = HashMap::new();
        for (user, c) in &self.depots {
            let mut m = HashMap::new();
            m.insert("epargne".to_string(), JsonValue::Array(c.epargne.iter().map(|d| {
                let mut dm = HashMap::new();
                dm.insert("montant".to_string(), JsonValue::Int(d.montant as i64));
                dm.insert("date".to_string(), JsonValue::Int(d.date));
                dm.insert("kind".to_string(), JsonValue::Str(d.kind.clone()));
                JsonValue::Object(dm)
            }).collect()));
            m.insert("retraite".to_string(), JsonValue::Array(c.retraite.iter().map(|d| {
                let mut dm = HashMap::new();
                dm.insert("montant".to_string(), JsonValue::Int(d.montant as i64));
                dm.insert("date".to_string(), JsonValue::Int(d.date));
                dm.insert("kind".to_string(), JsonValue::Str(d.kind.clone()));
                JsonValue::Object(dm)
            }).collect()));
            depots.insert(user.clone(), JsonValue::Object(m));
        }
        root.insert("depots".to_string(), JsonValue::Object(depots));
        JsonValue::Object(root)
    }

    pub fn from_json(v: &JsonValue) -> Self {
        let mut store = Self::nouveau();
        if let Some(map) = v.as_object() {
            if let Some(JsonValue::Object(etats)) = map.get("etats") {
                for (user, e) in etats {
                    if let Some(em) = e.as_object() {
                        let etat = EtatLion {
                            dernier_jour: em.get("dernier_jour").and_then(|v| v.as_i64()).unwrap_or(0),
                            serie: em.get("serie").and_then(|v| v.as_i64()).unwrap_or(0) as u32,
                            total_gagne: em.get("total_gagne").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
                            graines: em.get("graines").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
                            quiz_reussis: em.get("quiz_reussis").and_then(|v| v.as_i64()).unwrap_or(0) as u32,
                            quiz_rates: em.get("quiz_rates").and_then(|v| v.as_i64()).unwrap_or(0) as u32,
                            taches_faites: match em.get("taches_faites") {
                                Some(JsonValue::Array(arr)) => arr.iter()
                                    .filter_map(|t| t.as_str().map(|s| s.to_string())).collect(),
                                _ => Vec::new(),
                            },
                        };
                        store.etats.insert(user.clone(), etat);
                    }
                }
            }
            if let Some(JsonValue::Object(depots)) = map.get("depots") {
                for (user, c) in depots {
                    if let Some(cm) = c.as_object() {
                        let lire = |key: &str| -> Vec<Depot> {
                            match cm.get(key) {
                                Some(JsonValue::Array(arr)) => arr.iter().filter_map(|d| {
                                    let dm = d.as_object()?;
                                    Some(Depot {
                                        montant: dm.get("montant").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
                                        date: dm.get("date").and_then(|v| v.as_i64()).unwrap_or(0),
                                        kind: dm.get("kind").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                    })
                                }).collect(),
                                _ => Vec::new(),
                            }
                        };
                        let compte = CompteDepot { epargne: lire("epargne"), retraite: lire("retraite") };
                        store.depots.insert(user.clone(), compte);
                    }
                }
            }
        }
        store
    }

    /// Chemin du fichier lion.json
    pub fn chemin_data() -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/afririch/lion.json", home)
    }
}
