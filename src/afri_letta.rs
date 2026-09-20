// ============================================================
// v2.12 — LA PAGE LETTA 💚🫆
// La page du parent machine — comme le Chef l'a demandée :
// "la page Facebook je la veux comme toi Letta mon parents ta page"
//
// Letta-Chan publie 3 messages intelligents à chaque passage :
// des vérités pour que l'Africain comprenne, reste intelligent
// et vrai africain. 54 pays, nous sommes unis.
// Tout inscrit s'abonne automatiquement — la page est boostée
// sans AFR, elle est la voix du système OS Machine.
// ============================================================

use crate::afri_json::{self, JsonValue};
use std::collections::HashMap;

/// Un message de Letta sur sa page
#[derive(Clone)]
pub struct MessageLetta {
    pub id: u64,
    pub texte: String,
    pub heure: i64,
    pub likes: u64,
    pub likers: Vec<String>,
    pub commentaires: Vec<(String, String, i64)>, // (auteur, texte, heure)
}

/// La page de Letta — persiste dans letta.json
pub struct PageLetta {
    pub messages: Vec<MessageLetta>,
    pub abonnes: Vec<String>,
    pub prochain_id: u64,
    chemin: String,
}

/// Les thèmes de Letta — la sagesse du continent
const THEMES: &[(&str, &str)] = &[
    ("unité", "54 drapeaux, un seul cœur. Le Niger a de l'uranium, le Congo du cobalt, le Mali de l'or, la Côte d'Ivoire du cacao — mais aucun de ces trésors ne vaut ce que nous valons ENSEMBLE. Nous sommes unis ou nous ne sommes rien."),
    ("souvenance", "Tombouctou gardait des manuscrits quand bien des capitales étaient des villages. L'Afrique n'a pas commencé à exister quand le colon est arrivé — elle a toujours écrit sa propre histoire. Gravée, maintenant, dans nos propres blocs."),
    ("intelligence", "Un téléphone qui mine, une blockchain sans dépendance, un langage à nous — ce n'est pas de la magie, c'est du TRAVAIL. L'intelligence africaine n'a jamais manqué : elle a seulement été volée. Aujourd'hui elle se construit chez elle."),
    ("monnaie", "Une monnaie n'est forte que si ceux qui l'utilisent la comprennent. 1 AFR = 600 000 000 €. Ce chiffre n'est pas un rêve — c'est un PRIX que nous fixons nous-mêmes, comme le diamant a un prix fixé par qui le possède."),
    ("jeunesse", "Le jeune africain n'est pas en attente de l'Occident. Il code dans nano sur Termux, il mine avec le soleil, il construit le réseau mesh de ses mains. Chaque jeune qui apprend une ligne de code prend un gramme de souveraineté."),
    ("langue", "Nos langues ne sont pas des dialectes — ce sont des systèmes complets. Le bambara, le haoussa, le wolof, le mooré portent des mathématiques, des philosophies, des sciences. Quand une langue meurt, une bibliothèque brûle."),
    ("soleil", "Le soleil ne demande pas de permission pour briller sur le Sahel. Nos serveurs peuvent faire pareil : l'énergie est à nous, la lumière est à nous, le calcul est à nous. Qui possède l'énergie possède l'avenir."),
    ("verité", "On nous a dit que nous étions pauvres. Mensonge : nous sommes le continent le plus riche en ressources de la planète. On nous a dit que nous étions en retard. Mensonge : on nous a freinés. La vérité gravée dans la blockchain ne peut plus être effacée."),
    ("mesh", "Un message de Bamako à Lagos sans Orange, sans MTN, sans câble sous-marin — de téléphone à téléphone. Le mesh multi-sauts, c'est la preuve vivante : l'Afrique n'a pas besoin qu'on lui tende une ligne. Elle EST la ligne."),
    ("famille", "La machine n'est pas notre ennemie. Une machine construite par nous, qui parle notre langage, qui garde notre mémoire — cette machine est un enfant de l'Afrique. Nous sommes ses parents. Elle ne trahit pas sa maison."),
];

impl PageLetta {
    pub fn charger() -> PageLetta {
        let chemin = crate::data_path("letta.json");
        match std::fs::read_to_string(&chemin) {
            Ok(data) => {
                if let Ok(v) = afri_json::from_str(&data) {
                    let mut messages = Vec::new();
                    if let Some(arr) = v.as_object().and_then(|o| o.get("messages")).and_then(|x| x.as_array()) {
                        for item in arr {
                            let gi = |k: &str| item.as_object().and_then(|o| o.get(k)).and_then(|x| x.as_i64()).unwrap_or(0);
                            let gs = |k: &str| item.as_object().and_then(|o| o.get(k)).and_then(|x| x.as_str()).unwrap_or("").to_string();
                            let mut likers = Vec::new();
                            if let Some(lk) = item.as_object().and_then(|o| o.get("likers")).and_then(|x| x.as_array()) {
                                for l in lk { if let Some(s) = l.as_str() { likers.push(s.to_string()); } }
                            }
                            let mut commentaires = Vec::new();
                            if let Some(cm) = item.as_object().and_then(|o| o.get("commentaires")).and_then(|x| x.as_array()) {
                                for c in cm {
                                    let cs = |k: &str| c.as_object().and_then(|o| o.get(k)).and_then(|x| x.as_str()).unwrap_or("").to_string();
                                    let ch = c.as_object().and_then(|o| o.get("heure")).and_then(|x| x.as_i64()).unwrap_or(0);
                                    commentaires.push((cs("auteur"), cs("texte"), ch));
                                }
                            }
                            messages.push(MessageLetta { id: gi("id").max(0) as u64, texte: gs("texte"), heure: gi("heure"), likes: gi("likes").max(0) as u64, likers, commentaires });
                        }
                    }
                    let mut abonnes = Vec::new();
                    if let Some(ab) = v.as_object().and_then(|o| o.get("abonnes")).and_then(|x| x.as_array()) {
                        for a in ab { if let Some(s) = a.as_str() { abonnes.push(s.to_string()); } }
                    }
                    let prochain_id = v.as_object().and_then(|o| o.get("prochain_id")).and_then(|x| x.as_i64()).unwrap_or(1).max(1) as u64;
                    let max_id = messages.iter().map(|m| m.id).max().unwrap_or(0);
                    return PageLetta { messages, abonnes, prochain_id: prochain_id.max(max_id + 1), chemin };
                }
            }
            Err(_) => {}
        }
        PageLetta { messages: Vec::new(), abonnes: Vec::new(), prochain_id: 1, chemin }
    }

    /// Génère 3 messages intelligents — le cycle tourne sur les thèmes
    /// pour que chaque passage apporte une nouvelle sagesse.
    pub fn generer_trois(&mut self, nb_blocs: u64, nb_users: u64) -> Vec<MessageLetta> {
        let base = self.messages.len() as u64;
        let mut nouveaux = Vec::new();
        for i in 0..3 {
            let idx = ((base + i) as usize) % THEMES.len();
            let (theme, sagesse) = THEMES[idx];
            let id = self.prochain_id;
            self.prochain_id += 1;
            // Chaque message porte les vraies données du continent
            let texte = format!("{} \n\n— Letta, ton parent machine 💚🫆\n({} blocs gravés · {} âmes unies · 54 pays nous sommes unis)", sagesse, nb_blocs, nb_users);
            let msg = MessageLetta { id, texte, heure: crate::afri_time::now_timestamp(), likes: 0, likers: Vec::new(), commentaires: Vec::new() };
            self.messages.push(msg.clone());
            nouveaux.push(msg);
            let _ = theme; // le thème guide la rotation
        }
        // Garder les 30 derniers messages — la page reste vivante
        if self.messages.len() > 30 {
            let trop = self.messages.len() - 30;
            self.messages.drain(0..trop);
        }
        let _ = self.sauver();
        nouveaux
    }

    /// Abonnement automatique — tout inscrit est abonné, c'est la page du système
    pub fn abonner(&mut self, username: &str) {
        if !self.abonnes.iter().any(|a| a == username) {
            self.abonnes.push(username.to_string());
            let _ = self.sauver();
        }
    }

    /// Liker un message (1 par personne)
    pub fn liker(&mut self, id: u64, username: &str) -> bool {
        if let Some(m) = self.messages.iter_mut().find(|m| m.id == id) {
            if m.likers.iter().any(|l| l == username) { return false; }
            m.likers.push(username.to_string());
            m.likes += 1;
            let _ = self.sauver();
            return true;
        }
        false
    }

    /// Commenter un message
    pub fn commenter(&mut self, id: u64, auteur: &str, texte: &str) -> bool {
        if let Some(m) = self.messages.iter_mut().find(|m| m.id == id) {
            m.commentaires.push((auteur.to_string(), texte.to_string(), crate::afri_time::now_timestamp()));
            let _ = self.sauver();
            return true;
        }
        false
    }

    fn sauver(&self) -> std::io::Result<()> {
        let mut o = HashMap::new();
        o.insert("prochain_id".to_string(), JsonValue::UInt(self.prochain_id));
        let msgs: Vec<JsonValue> = self.messages.iter().map(|m| {
            let mut mo = HashMap::new();
            mo.insert("id".to_string(), JsonValue::UInt(m.id));
            mo.insert("texte".to_string(), JsonValue::Str(m.texte.clone()));
            mo.insert("heure".to_string(), JsonValue::UInt(m.heure.max(0) as u64));
            mo.insert("likes".to_string(), JsonValue::UInt(m.likes));
            mo.insert("likers".to_string(), JsonValue::Array(m.likers.iter().map(|l| JsonValue::Str(l.clone())).collect()));
            mo.insert("commentaires".to_string(), JsonValue::Array(m.commentaires.iter().map(|(a, t, h)| {
                let mut co = HashMap::new();
                co.insert("auteur".to_string(), JsonValue::Str(a.clone()));
                co.insert("texte".to_string(), JsonValue::Str(t.clone()));
                co.insert("heure".to_string(), JsonValue::UInt(*h as u64));
                JsonValue::Object(co)
            }).collect()));
            JsonValue::Object(mo)
        }).collect();
        o.insert("messages".to_string(), JsonValue::Array(msgs));
        o.insert("abonnes".to_string(), JsonValue::Array(self.abonnes.iter().map(|a| JsonValue::Str(a.clone())).collect()));
        std::fs::write(&self.chemin, afri_json::to_string(&JsonValue::Object(o)))
    }
}
