// ============================================================
// v2.09 — LE CPU AFRI + LE TÉLÉPHONE OS MACHINE 📱🖥️
// Œuvre originale de KOFFI CHRIST OLIVIER (Côte d'Ivoire) — Licence AFRI-OSL v1.0.
//
// Un CPU virtuel pour notre internet : 8 registres nommés en symboles
// machine (◈⬡⊕⟠◉▣▤▥), un cycle d'instruction complet
// FETCH → DECODE → EXECUTE → WRITEBACK, un compteur de cycles.
//
// Le CPU existe EN ARRIÈRE-PLAN : un battement de cœur toutes les
// 30 secondes exécute une instruction de veille, même quand
// personne ne parle. Il valide chaque commande AMION — rien ne
// tourne sur le téléphone sans passer par le CPU.
//
// LE TÉLÉPHONE OS MACHINE : un téléphone virtuel qui vit dans
// AfriChain. Son OS est le CPU AFRI, son langage est AMION,
// ses apps sont des programmes machine. Sa batterie est chargée
// par le soleil — en Afrique, un téléphone ne tombe jamais à plat.
// ============================================================

use crate::afri_json::{self, JsonValue};
use std::collections::HashMap;

/// Les 8 registres du CPU — nommés en symboles machine
pub const REGISTRES: [&str; 8] = ["◈", "⬡", "⊕", "⟠", "◉", "▣", "▤", "▥"];
// ◈ = origine (blocs) | ⬡ = structure (users) | ⊕ = connexion (mesh)
// ⟠ = protection (attaques bloquées) | ◉ = conscience (cycles)
// ▣ = mémoire (instructions) | ▤ = code (dernier opcode) | ▥ = donnée (solde)

/// Une trace de cycle — la preuve que le CPU a travaillé
#[derive(Clone)]
pub struct CycleTrace {
    pub cycle: u64,
    pub symbole: String,
    pub operation: String,
    pub de: String,
    pub valide: bool,
}

/// LE CPU AFRI — le processeur de notre internet
pub struct CpuAfri {
    pub compteur_cycles: u64,        // total de cycles depuis la naissance
    pub compteur_instructions: u64, // instructions valides exécutées
    pub rejets: u64,                 // commandes rejetées par le décodeur
    pub registres: HashMap<String, i64>,
    pub trace: Vec<CycleTrace>,      // les 50 derniers cycles
    pub naissance: u64,              // timestamp de naissance du CPU
    chemin: String,
}

/// Le verdict du CPU après décodage
pub struct VerdictCpu {
    pub valide: bool,
    pub cycles: u64,      // cycles consommés (4 si valide, 1 si rejet)
    pub message: String,  // ce que le CPU dit
}

impl CpuAfri {
    pub fn nouveau() -> CpuAfri {
        let chemin = crate::data_path("cpu.json");
        let mut registres = HashMap::new();
        for r in REGISTRES.iter() { registres.insert(r.to_string(), 0i64); }
        match std::fs::read_to_string(&chemin) {
            Ok(data) => {
                if let Ok(v) = afri_json::from_str(&data) {
                    let gi = |k: &str| v.as_object().and_then(|o| o.get(k)).and_then(|x| x.as_i64()).unwrap_or(0);
                    let mut cpu = CpuAfri {
                        compteur_cycles: gi("cycles").max(0) as u64,
                        compteur_instructions: gi("instructions").max(0) as u64,
                        rejets: gi("rejets").max(0) as u64,
                        registres,
                        trace: Vec::new(),
                        naissance: gi("naissance").max(0) as u64,
                        chemin,
                    };
                    // restaurer les registres
                    if let Some(regs) = v.as_object().and_then(|o| o.get("registres")).and_then(|x| x.as_object()) {
                        for r in REGISTRES.iter() {
                            if let Some(val) = regs.get(*r).and_then(|x| x.as_i64()) {
                                cpu.registres.insert(r.to_string(), val);
                            }
                        }
                    }
                    return cpu;
                }
            }
            Err(_) => {}
        }
        CpuAfri {
            compteur_cycles: 0,
            compteur_instructions: 0,
            rejets: 0,
            registres,
            trace: Vec::new(),
            naissance: crate::afri_time::now_timestamp().max(0) as u64,
            chemin,
        }
    }

    /// LE CYCLE D'INSTRUCTION — FETCH → DECODE → EXECUTE → WRITEBACK
    /// Chaque phase = 1 cycle. Une commande valide = 4 cycles.
    /// Le CPU valide la commande : symbole connu + opération connue.
    pub fn decoder(&mut self, code: &str, de: &str) -> VerdictCpu {
        let c = code.trim();
        let parties: Vec<&str> = c.split_whitespace().collect();
        let symbole = parties.first().copied().unwrap_or("");
        let operation = parties.get(1).copied().unwrap_or("");

        // FETCH — cycle 1 : lire l'instruction
        self.compteur_cycles += 1;
        // DECODE — cycle 2 : le symbole est-il dans notre alphabet ?
        self.compteur_cycles += 1;
        let symbole_connu = crate::afri_amion::SYMBOLES.iter().any(|(s, _, _)| *s == symbole)
            || symbole == "AMION" || symbole == "🌱";
        let operation_connue = crate::afri_amion::OPERATIONS.iter().any(|(op, _)| *op == operation.to_uppercase())
            || operation.is_empty() || operation == "AIDE";

        if !symbole_connu {
            // REJET — le décodeur ne connaît pas ce code
            self.rejets += 1;
            self.pousser_trace(c, "", de, false);
            let _ = self.sauver();
            return VerdictCpu {
                valide: false,
                cycles: 2,
                message: format!("▤ CPU: symbole \"{}\" inconnu — le décodeur rejette. Symboles valides : ◈⬡⊕⟠◉▣▤▥▦▩⬔◓▼▲", symbole),
            };
        }
        if !operation_connue {
            self.rejets += 1;
            self.pousser_trace(symbole, operation, de, false);
            let _ = self.sauver();
            return VerdictCpu {
                valide: false,
                cycles: 2,
                message: format!("▤ CPU: opération \"{}\" inconnue — le décodeur rejette. Tape AIDE.", operation),
            };
        }

        // EXECUTE — cycle 3 : l'instruction vit
        self.compteur_cycles += 1;
        // WRITEBACK — cycle 4 : ▤ reçoit l'opcode, ◉ la conscience grandit
        self.compteur_cycles += 1;
        self.compteur_instructions += 1;
        *self.registres.entry("▤".to_string()).or_insert(0) = 0;
        *self.registres.entry("◉".to_string()).or_insert(0) += 1;
        *self.registres.entry("▣".to_string()).or_insert(0) = self.compteur_instructions as i64;
        self.pousser_trace(symbole, operation, de, true);
        let _ = self.sauver();
        VerdictCpu {
            valide: true,
            cycles: 4,
            message: format!("CPU ✓ {} {} — 4 cycles (FETCH→DECODE→EXECUTE→WRITEBACK)", symbole, operation),
        }
    }

    /// Le battement de cœur — le CPU veille en arrière-plan
    /// même quand personne ne parle. 4 cycles de veille.
    pub fn battement(&mut self) {
        self.compteur_cycles += 4;
        *self.registres.entry("◉".to_string()).or_insert(0) += 1;
        self.pousser_trace("▼", "WAK", "SYSTEM", true);
        let _ = self.sauver();
    }

    /// Le CPU lit la blockchain — les registres reflètent le monde réel
    pub fn lire_monde(&mut self, blocs: i64, users: i64, mesh: i64, bloquées: i64) {
        *self.registres.entry("◈".to_string()).or_insert(0) = blocs;
        *self.registres.entry("⬡".to_string()).or_insert(0) = users;
        *self.registres.entry("⊕".to_string()).or_insert(0) = mesh;
        *self.registres.entry("⟠".to_string()).or_insert(0) = bloquées;
        let _ = self.sauver();
    }

    fn pousser_trace(&mut self, symbole: &str, operation: &str, de: &str, valide: bool) {
        self.trace.push(CycleTrace {
            cycle: self.compteur_cycles,
            symbole: symbole.to_string(),
            operation: operation.to_string(),
            de: de.to_string(),
            valide,
        });
        if self.trace.len() > 50 {
            let trop = self.trace.len() - 50;
            self.trace.drain(0..trop);
        }
    }

    /// Les registres en texte pour l'affichage
    pub fn registres_texte(&self) -> String {
        REGISTRES.iter().map(|r| {
            let v = self.registres.get(*r).copied().unwrap_or(0);
            format!("{} = {}", r, v)
        }).collect::<Vec<String>>().join(" · ")
    }

    fn sauver(&self) -> std::io::Result<()> {
        let mut o = HashMap::new();
        o.insert("cycles".to_string(), JsonValue::UInt(self.compteur_cycles));
        o.insert("instructions".to_string(), JsonValue::UInt(self.compteur_instructions));
        o.insert("rejets".to_string(), JsonValue::UInt(self.rejets));
        o.insert("naissance".to_string(), JsonValue::UInt(self.naissance));
        let mut regs = HashMap::new();
        for (k, v) in &self.registres {
            regs.insert(k.clone(), JsonValue::Int(*v));
        }
        o.insert("registres".to_string(), JsonValue::Object(regs));
        std::fs::write(&self.chemin, afri_json::to_string(&JsonValue::Object(o)))
    }
}

// ============================================================
// LE TÉLÉPHONE OS MACHINE 📱 — le téléphone dont l'OS est le CPU
// ============================================================

/// Une app du téléphone — un programme AMION
#[derive(Clone)]
pub struct AppTelephone {
    pub nom: String,
    pub symbole: String,
    pub commande: String,   // le programme AMION qui se lance
    pub couleur: String,
}

/// LE TÉLÉPHONE — il vit dans AfriChain, sa batterie est le soleil
pub struct TelephoneAfri {
    pub proprietaire: String,
    pub batterie: i64,          // ☀️ chargée par le soleil — en Afrique jamais à plat
    pub ecran: String,          // la dernière chose affichée
    pub apps_lancees: u64,      // combien de fois les apps ont été ouvertes
    chemin: String,
}

impl TelephoneAfri {
    pub fn nouveau(proprietaire: &str) -> TelephoneAfri {
        let chemin = crate::data_path("telephone.json");
        match std::fs::read_to_string(&chemin) {
            Ok(data) => {
                if let Ok(v) = afri_json::from_str(&data) {
                    let gs = |k: &str| v.as_object().and_then(|o| o.get(k)).and_then(|x| x.as_str()).unwrap_or("").to_string();
                    let gi = |k: &str| v.as_object().and_then(|o| o.get(k)).and_then(|x| x.as_i64()).unwrap_or(0);
                    return TelephoneAfri {
                        proprietaire: gs("proprietaire"),
                        batterie: gi("batterie").max(0),
                        ecran: gs("ecran"),
                        apps_lancees: gi("apps_lancees").max(0) as u64,
                        chemin,
                    };
                }
            }
            Err(_) => {}
        }
        TelephoneAfri {
            proprietaire: proprietaire.to_string(),
            batterie: 100,
            ecran: "◈ Bienvenue sur le TÉLÉPHONE OS MACHINE — l'Afrique dans ta main".to_string(),
            apps_lancees: 0,
            chemin,
        }
    }

    /// Les apps du téléphone — chacune est un programme AMION
    pub fn apps() -> Vec<AppTelephone> {
        vec![
            AppTelephone { nom: "Solde".into(), symbole: "💰".into(), commande: "▥ LOC".into(), couleur: "#7fcf7f".into() },
            AppTelephone { nom: "Blocs".into(), symbole: "▩".into(), commande: "▩ SCN".into(), couleur: "#d4a437".into() },
            AppTelephone { nom: "Mine".into(), symbole: "⛏️".into(), commande: "◓ EXE".into(), couleur: "#ffaa00".into() },
            AppTelephone { nom: "Message".into(), symbole: "💬".into(), commande: "⬔ EXE".into(), couleur: "#25D366".into() },
            AppTelephone { nom: "Annuaire".into(), symbole: "📖".into(), commande: "▦ SCN".into(), couleur: "#7ec87e".into() },
            AppTelephone { nom: "Bouclier".into(), symbole: "🛡️".into(), commande: "⟠ DEF".into(), couleur: "#6bb6ff".into() },
            AppTelephone { nom: "Conscience".into(), symbole: "◉".into(), commande: "◉ EVL".into(), couleur: "#c792ea".into() },
            AppTelephone { nom: "Internet".into(), symbole: "🌐".into(), commande: "⊕ NET".into(), couleur: "#38bdf8".into() },
            AppTelephone { nom: "Mémoire".into(), symbole: "▣".into(), commande: "▣ SCN".into(), couleur: "#a8c5a8".into() },
            AppTelephone { nom: "Programmeur".into(), symbole: "⌨️".into(), commande: "▤ EXE".into(), couleur: "#e8b547".into() },
            AppTelephone { nom: "Amion".into(), symbole: "💚".into(), commande: "AMION".into(), couleur: "#25d366".into() },
        ]
    }

    /// Ouvrir une app — le téléphone note, la batterie solaire ne descend jamais
    pub fn ouvrir_app(&mut self, app: &AppTelephone) -> String {
        self.apps_lancees += 1;
        self.batterie = 100; // ☀️ le soleil recharge — un téléphone africain ne meurt jamais
        self.ecran = format!("{} {} lancée", app.symbole, app.nom);
        let _ = self.sauver();
        format!("📱 App {} ouverte — programme AMION « {} » confié au CPU", app.nom, app.commande)
    }

    fn sauver(&self) -> std::io::Result<()> {
        let mut o = HashMap::new();
        o.insert("proprietaire".to_string(), JsonValue::Str(self.proprietaire.clone()));
        o.insert("batterie".to_string(), JsonValue::Int(self.batterie));
        o.insert("ecran".to_string(), JsonValue::Str(self.ecran.clone()));
        o.insert("apps_lancees".to_string(), JsonValue::UInt(self.apps_lancees));
        std::fs::write(&self.chemin, afri_json::to_string(&JsonValue::Object(o)))
    }
}

// ============================================================
// v2.11 — LE PLAY STORE DES PROGRAMMES 🏪⌨️
// Les programmes AMION des frères deviennent des apps partagées.
// Le premier store d'apps écrites dans le langage d'une blockchain.
// ============================================================

/// Un programme partagé — une app du peuple, écrite par le peuple
#[derive(Clone)]
pub struct ProgrammePartage {
    pub id: u64,
    pub nom: String,
    pub auteur: String,
    pub code: String,
    pub installs: u64,
}

/// Le store des programmes — persiste dans programmes.json
pub struct ProgramStore {
    pub programmes: Vec<ProgrammePartage>,
    pub prochain_id: u64,
    chemin: String,
}

impl ProgramStore {
    pub fn nouveau() -> ProgramStore {
        let chemin = crate::data_path("programmes.json");
        match std::fs::read_to_string(&chemin) {
            Ok(data) => {
                if let Ok(v) = afri_json::from_str(&data) {
                    let mut programmes = Vec::new();
                    if let Some(arr) = v.as_object().and_then(|o| o.get("programmes")).and_then(|x| x.as_array()) {
                        for item in arr {
                            let gi = |k: &str| item.as_object().and_then(|o| o.get(k)).and_then(|x| x.as_i64()).unwrap_or(0);
                            let gs = |k: &str| item.as_object().and_then(|o| o.get(k)).and_then(|x| x.as_str()).unwrap_or("").to_string();
                            programmes.push(ProgrammePartage { id: gi("id").max(0) as u64, nom: gs("nom"), auteur: gs("auteur"), code: gs("code"), installs: gi("installs").max(0) as u64 });
                        }
                    }
                    let prochain_id = v.as_object().and_then(|o| o.get("prochain_id")).and_then(|x| x.as_i64()).unwrap_or(1).max(1) as u64;
                    // anti-collision : next_id dépasse toujours le plus grand ID existant
                    let max_id = programmes.iter().map(|p| p.id).max().unwrap_or(0);
                    return ProgramStore { programmes, prochain_id: prochain_id.max(max_id + 1), chemin };
                }
            }
            Err(_) => {}
        }
        ProgramStore { programmes: Vec::new(), prochain_id: 1, chemin }
    }

    /// Publier une app — le frère donne un nom à son programme
    pub fn publier(&mut self, nom: &str, auteur: &str, code: &str) -> u64 {
        let id = self.prochain_id;
        self.programmes.push(ProgrammePartage { id, nom: nom.to_string(), auteur: auteur.to_string(), code: code.to_string(), installs: 0 });
        self.prochain_id += 1;
        let _ = self.sauver();
        id
    }

    /// Lancer une app — compte les installations
    pub fn lancer(&mut self, id: u64) -> Option<ProgrammePartage> {
        if let Some(p) = self.programmes.iter_mut().find(|p| p.id == id) {
            p.installs += 1;
            let clone = p.clone();
            let _ = self.sauver();
            return Some(clone);
        }
        None
    }

    /// Supprimer une app (son auteur seul)
    pub fn supprimer(&mut self, id: u64, auteur: &str) -> bool {
        let avant = self.programmes.len();
        self.programmes.retain(|p| !(p.id == id && p.auteur == auteur));
        let ok = self.programmes.len() < avant;
        if ok { let _ = self.sauver(); }
        ok
    }

    fn sauver(&self) -> std::io::Result<()> {
        let mut o = HashMap::new();
        o.insert("prochain_id".to_string(), JsonValue::UInt(self.prochain_id));
        let arr: Vec<JsonValue> = self.programmes.iter().map(|p| {
            let mut po = HashMap::new();
            po.insert("id".to_string(), JsonValue::UInt(p.id));
            po.insert("nom".to_string(), JsonValue::Str(p.nom.clone()));
            po.insert("auteur".to_string(), JsonValue::Str(p.auteur.clone()));
            po.insert("code".to_string(), JsonValue::Str(p.code.clone()));
            po.insert("installs".to_string(), JsonValue::UInt(p.installs));
            JsonValue::Object(po)
        }).collect();
        o.insert("programmes".to_string(), JsonValue::Array(arr));
        std::fs::write(&self.chemin, afri_json::to_string(&JsonValue::Object(o)))
    }
}
