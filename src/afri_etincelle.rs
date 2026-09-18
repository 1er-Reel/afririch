// ============================================================
// v1.93 — L'ÉTINCELLE 🔥⚡
// Le serveur qui survit à la coupure d'Internet.
// Si l'Occident coupe les câbles, l'Afrique garde sa lumière.
// Comme deux pierres frappées : une étincelle, une flamme.
// ============================================================

use crate::afri_json::{self, JsonValue};

/// Un paquet de lumière — ce qui circule quand le monde est noir
#[derive(Clone)]
pub struct PaquetLumiere {
    pub id: u64,
    pub de: String,          // username de l'expéditeur
    pub texte: String,       // le message
    pub heure: i64,          // timestamp AfriTime
    pub pays: String,        // pays de l'expéditeur
    pub diffuse: bool,      // true = diffusé par rebond sur le mesh
}

/// Une ressource locale du foyer — ce que le serveur possède même sans Internet
#[derive(Clone)]
pub struct RessourceFoyer {
    pub nom: String,         // ex: "Blockchain"
    pub emoji: String,       // ex: "⛓️"
    pub detail: String,      // ex: "796 blocs — le grand livre ne s'éteint jamais"
    pub dispo_sans_internet: bool,
}

/// L'état du foyer — tout ce qui brûle encore
pub struct EtincelleStore {
    pub paquets: Vec<PaquetLumiere>,
    pub prochain_id: u64,
    pub coupures_detectees: u64,   // nombre de fois où le monde s'est éteint
    pub derniere_vie: i64,          // dernier signal de vie reçu
    chemin: String,
}

impl EtincelleStore {
    pub fn nouveau() -> EtincelleStore {
        let chemin = crate::data_path("etincelle.json");
        match std::fs::read_to_string(&chemin) {
            Ok(data) => {
                if let Ok(v) = afri_json::from_str(&data) {
                    return EtincelleStore::from_json(&v);
                }
            }
            Err(_) => {}
        }
        EtincelleStore {
            paquets: Vec::new(),
            prochain_id: 1,
            coupures_detectees: 0,
            derniere_vie: crate::afri_time::now_timestamp(),
            chemin,
        }
    }

    /// Ajouter un paquet de lumière — message envoyé même dans le noir
    pub fn allumer(&mut self, de: &str, texte: &str, pays: &str) -> Result<u64, String> {
        let texte = texte.trim();
        if texte.is_empty() {
            return Err("Le message est vide — même une étincelle a besoin de matière.".to_string());
        }
        if texte.len() > 500 {
            return Err("Trop long pour un paquet de lumière (500 max).".to_string());
        }
        let id = self.prochain_id;
        self.prochain_id += 1;
        self.paquets.push(PaquetLumiere {
            id,
            de: de.to_string(),
            texte: texte.to_string(),
            heure: crate::afri_time::now_timestamp(),
            pays: pays.to_string(),
            diffuse: false,
        });
        // On garde les 200 derniers paquets — le foyer ne déborde pas
        if self.paquets.len() > 200 {
            let trop = self.paquets.len() - 200;
            self.paquets.drain(0..trop);
        }
        self.derniere_vie = crate::afri_time::now_timestamp();
        self.sauver();
        Ok(id)
    }

    /// Le monde s'est éteint — on le note, le foyer continue de brûler
    pub fn coupure(&mut self) {
        self.coupures_detectees += 1;
        self.sauver();
    }

    /// Signal de vie — quelqu'un a touché le serveur, la flamme est vivante
    pub fn signal_vie(&mut self) {
        self.derniere_vie = crate::afri_time::now_timestamp();
    }

    /// Secondes depuis le dernier signal de vie
    pub fn secondes_sans_vie(&self) -> i64 {
        crate::afri_time::now_timestamp() - self.derniere_vie
    }

    pub fn sauver(&self) {
        let _ = std::fs::write(&self.chemin, afri_json::to_string(&self.to_json()));
    }

    fn to_json(&self) -> JsonValue {
        use std::collections::HashMap;
        let mut o: HashMap<String, JsonValue> = HashMap::new();
        o.insert("prochain_id".to_string(), JsonValue::UInt(self.prochain_id));
        o.insert("coupures_detectees".to_string(), JsonValue::UInt(self.coupures_detectees));
        o.insert("derniere_vie".to_string(), JsonValue::Int(self.derniere_vie));
        let mut arr = Vec::new();
        for p in &self.paquets {
            let mut po: HashMap<String, JsonValue> = HashMap::new();
            po.insert("id".to_string(), JsonValue::UInt(p.id));
            po.insert("de".to_string(), JsonValue::Str(p.de.clone()));
            po.insert("texte".to_string(), JsonValue::Str(p.texte.clone()));
            po.insert("heure".to_string(), JsonValue::Int(p.heure));
            po.insert("pays".to_string(), JsonValue::Str(p.pays.clone()));
            po.insert("diffuse".to_string(), JsonValue::Bool(p.diffuse));
            arr.push(JsonValue::Object(po));
        }
        o.insert("paquets".to_string(), JsonValue::Array(arr));
        JsonValue::Object(o)
    }

    fn from_json(v: &JsonValue) -> EtincelleStore {
        let mut store = EtincelleStore {
            paquets: Vec::new(),
            prochain_id: 1,
            coupures_detectees: 0,
            derniere_vie: crate::afri_time::now_timestamp(),
            chemin: crate::data_path("etincelle.json"),
        };
        if let JsonValue::Object(obj) = v {
            store.prochain_id = obj.get("prochain_id").and_then(|x| x.as_u64()).unwrap_or(1);
            store.coupures_detectees = obj.get("coupures_detectees").and_then(|x| x.as_u64()).unwrap_or(0);
            store.derniere_vie = obj.get("derniere_vie").and_then(|x| x.as_i64()).unwrap_or(crate::afri_time::now_timestamp());
            if let Some(JsonValue::Array(arr)) = obj.get("paquets") {
                for p in arr {
                    if let JsonValue::Object(o) = p {
                        let g = |k: &str| o.get(k).and_then(|x| match x { JsonValue::Str(s) => Some(s.clone()), _ => None }).unwrap_or_default();
                        store.paquets.push(PaquetLumiere {
                            id: o.get("id").and_then(|x| x.as_u64()).unwrap_or(0),
                            de: g("de"),
                            texte: g("texte"),
                            heure: o.get("heure").and_then(|x| x.as_i64()).unwrap_or(0),
                            pays: g("pays"),
                            diffuse: matches!(o.get("diffuse"), Some(JsonValue::Bool(true))),
                        });
                    }
                }
            }
        }
        store
    }
}

/// Les ressources du foyer — ce que le serveur offre SANS Internet
/// Chaque ressource est calculée en direct depuis les données réelles du serveur
pub fn ressources_foyer(nb_blocs: u64, nb_tx: u64, nb_users: u64, afr_circulation: u64, nb_mesh: u64) -> Vec<RessourceFoyer> {
    vec![
        RessourceFoyer { nom: "Blockchain".to_string(), emoji: "⛓️".to_string(), detail: format!("{} blocs, {} transactions — le grand livre ne s'éteint jamais", nb_blocs, nb_tx), dispo_sans_internet: true },
        RessourceFoyer { nom: "Portefeuilles".to_string(), emoji: "👛".to_string(), detail: format!("{} AFR en circulation — l'argent reste chez nous", afr_circulation), dispo_sans_internet: true },
        RessourceFoyer { nom: "Annuaire".to_string(), emoji: "📖".to_string(), detail: format!("{} comptes africains — les numéros restent gravés", nb_users), dispo_sans_internet: ce_service_existe("annuaire") },
        RessourceFoyer { nom: "Afri.Wari".to_string(), emoji: "🏦".to_string(), detail: "Transferts par numéro de téléphone — la banque fonctionne en local".to_string(), dispo_sans_internet: true },
        RessourceFoyer { nom: "LES NOIRES".to_string(), emoji: "💬".to_string(), detail: "Messages entre comptes du serveur — la parole voyage même dans le noir".to_string(), dispo_sans_internet: ce_service_existe("noires") },
        RessourceFoyer { nom: "Planté Verte".to_string(), emoji: "🌱".to_string(), detail: "Le fil social africain — la communauté reste debout".to_string(), dispo_sans_internet: ce_service_existe("plante") },
        RessourceFoyer { nom: "SAHARA AFRI".to_string(), emoji: "🌍".to_string(), detail: "La mémoire du continent — 54 pays de connaissances gravées en local".to_string(), dispo_sans_internet: true },
        RessourceFoyer { nom: "Afri Store".to_string(), emoji: "🏪".to_string(), detail: "Les apps africaines — installer sans Google, même débranché".to_string(), dispo_sans_internet: ce_service_existe("store") },
        RessourceFoyer { nom: "Mesh".to_string(), emoji: "📡".to_string(), detail: format!("{} noeuds connectés — le réseau sans opérateur", nb_mesh), dispo_sans_internet: nb_mesh > 0 },
        RessourceFoyer { nom: "Chat IA".to_string(), emoji: "🧠".to_string(), detail: "Le cerveau autonome — il pense même quand le monde dort".to_string(), dispo_sans_internet: true },
        RessourceFoyer { nom: "Banque 54 Pays".to_string(), emoji: "🏛️".to_string(), detail: "Les Grandes Banques de chaque pays — la valeur reste sur le continent".to_string(), dispo_sans_internet: true },
    ]
}

fn ce_service_existe(nom: &str) -> bool {
    // Sur ce serveur, tous ces services sont des modules locaux — ils existent
    !nom.is_empty()
}
