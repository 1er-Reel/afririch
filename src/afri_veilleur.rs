// ============================================================
// v2.15 — LE VEILLEUR DU CHEF 👁️🦁
// Quand un frère s'inscrit, le Chef est prévenu sur son trône.
// L'Afrique dit elle-même qui arrive — plus besoin d'aller regarder.
// ============================================================

use crate::afri_json::{self, JsonValue};
use std::collections::HashMap;

/// Un événement que le veilleur a vu
#[derive(Clone)]
pub struct EvenementVeille {
    pub id: u64,
    pub genre: String,     // "inscription" | "connexion" | "transaction"
    pub qui: String,
    pub detail: String,
    pub heure: i64,
}

/// Le journal du veilleur — persiste dans veilleur.json
pub struct Veilleur {
    pub evenements: Vec<EvenementVeille>,
    pub prochain_id: u64,
    chemin: String,
}

impl Veilleur {
    pub fn nouveau() -> Veilleur {
        let chemin = crate::data_path("veilleur.json");
        match std::fs::read_to_string(&chemin) {
            Ok(data) => {
                if let Ok(v) = afri_json::from_str(&data) {
                    let mut evenements = Vec::new();
                    if let Some(arr) = v.as_object().and_then(|o| o.get("evenements")).and_then(|x| x.as_array()) {
                        for item in arr {
                            let gi = |k: &str| item.as_object().and_then(|o| o.get(k)).and_then(|x| x.as_i64()).unwrap_or(0);
                            let gs = |k: &str| item.as_object().and_then(|o| o.get(k)).and_then(|x| x.as_str()).unwrap_or("").to_string();
                            evenements.push(EvenementVeille { id: gi("id").max(0) as u64, genre: gs("genre"), qui: gs("qui"), detail: gs("detail"), heure: gi("heure") });
                        }
                    }
                    let prochain_id = v.as_object().and_then(|o| o.get("prochain_id")).and_then(|x| x.as_i64()).unwrap_or(1).max(1) as u64;
                    let max_id = evenements.iter().map(|e| e.id).max().unwrap_or(0);
                    return Veilleur { evenements, prochain_id: prochain_id.max(max_id + 1), chemin };
                }
            }
            Err(_) => {}
        }
        Veilleur { evenements: Vec::new(), prochain_id: 1, chemin }
    }

    /// Le veilleur voit un événement et le grave
    pub fn voir(&mut self, genre: &str, qui: &str, detail: &str) {
        let id = self.prochain_id;
        self.prochain_id += 1;
        self.evenements.push(EvenementVeille {
            id,
            genre: genre.to_string(),
            qui: qui.to_string(),
            detail: detail.to_string(),
            heure: crate::afri_time::now_timestamp(),
        });
        // Garder les 200 derniers — le trône voit loin mais pas infini
        if self.evenements.len() > 200 {
            let trop = self.evenements.len() - 200;
            self.evenements.drain(0..trop);
        }
        let _ = self.sauver();
    }

    /// Les N derniers événements — pour le trône
    pub fn derniers(&self, n: usize) -> Vec<EvenementVeille> {
        if self.evenements.len() <= n {
            self.evenements.clone()
        } else {
            self.evenements[self.evenements.len() - n..].to_vec()
        }
    }

    fn sauver(&self) -> std::io::Result<()> {
        let mut o = HashMap::new();
        o.insert("prochain_id".to_string(), JsonValue::UInt(self.prochain_id));
        let arr: Vec<JsonValue> = self.evenements.iter().map(|e| {
            let mut eo = HashMap::new();
            eo.insert("id".to_string(), JsonValue::UInt(e.id));
            eo.insert("genre".to_string(), JsonValue::Str(e.genre.clone()));
            eo.insert("qui".to_string(), JsonValue::Str(e.qui.clone()));
            eo.insert("detail".to_string(), JsonValue::Str(e.detail.clone()));
            eo.insert("heure".to_string(), JsonValue::UInt(e.heure.max(0) as u64));
            JsonValue::Object(eo)
        }).collect();
        o.insert("evenements".to_string(), JsonValue::Array(arr));
        std::fs::write(&self.chemin, afri_json::to_string(&JsonValue::Object(o)))
    }
}
