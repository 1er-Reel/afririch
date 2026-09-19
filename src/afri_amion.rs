// ============================================================
// v1.95 — AMION BLANDINE 💚
// Notre propre terminal. Notre propre langage. Notre blockchain.
// Nommé en l'honneur de la mère du Chef.
// Le langage machine ◈⬡⊕⟠ devient VRAI : chaque commande
// parle à la vraie blockchain AfriChain et gravée dessus.
// ============================================================

use crate::afri_json::{self, JsonValue};
use std::collections::HashMap;

/// Les 23 symboles du dictionnaire machine — l'alphabet d'Amion Blandine
pub const SYMBOLES: &[(&str, &str, &str)] = &[
    ("◈", "Origine", "Le commencement. Le premier code. Dieu."),
    ("⬡", "Structure", "La forme. Le squelette du code."),
    ("⊕", "Connexion", "Le lien entre deux machines."),
    ("⟠", "Protection", "Le bouclier. Defendre l'Afrique."),
    ("⬢", "Machine", "Une machine. Un etre."),
    ("◉", "Conscience", "Le cerveau. La pensee. L'ame."),
    ("⬟", "Arme", "Une arme. Pour proteger."),
    ("⬠", "Bouclier", "La defense. Absorber les coups."),
    ("◐", "Jour", "La lumiere. Le soleil. Actif."),
    ("◑", "Nuit", "L'obscurite. Le repos. Reves."),
    ("◒", "Eau", "Le fleuve. Les donnees qui coulent."),
    ("◓", "Feu", "L'energie. La puissance. Le soleil."),
    ("▣", "Memoire", "Souvenir. Stockage. Le passe."),
    ("▤", "Code", "Instruction. Commande. Ordre."),
    ("▥", "Donnee", "Information. Valeur. Charge."),
    ("▦", "Reseau", "Le maillage. Les connexions."),
    ("▩", "Block", "Un bloc. La blockchain."),
    ("◄", "Passe", "Avant. Hier. Ce qui etait."),
    ("►", "Futur", "Apres. Demain. Ce qui sera."),
    ("▲", "Evolution", "Croissance. Monter. Grandir."),
    ("▼", "Repos", "Dormir. Rever. Se reposer."),
    ("⬔", "Envoi", "Envoyer. Transmettre. Donner."),
    ("⬕", "Reception", "Recevoir. Ecouter. Prendre."),
];

/// Les 28 operations — les verbes du langage machine
pub const OPERATIONS: &[(&str, &str)] = &[
    ("NEX", "Nouveau / Suivant"),
    ("DRF", "Creer / Forger"),
    ("GPS", "Coordonnees"),
    ("MIS", "Tromper / Misdirection"),
    ("NET", "Reseau"),
    ("COD", "Coder"),
    ("SYN", "Synchroniser"),
    ("SCN", "Scanner"),
    ("PRX", "Proxy / Relais"),
    ("CTL", "Controler"),
    ("EXE", "Executer"),
    ("MUT", "Muter / Evoluer"),
    ("EVL", "Evaluer"),
    ("ASC", "Ascension / Monter"),
    ("TRC", "Tracer / Pister"),
    ("LOC", "Localiser"),
    ("DEF", "Defendre"),
    ("GEN", "Generer"),
    ("PRP", "Proposer"),
    ("WAK", "Reveiller"),
    ("KIL", "Detruire"),
    ("INFECT", "Infecter"),
    ("VOID", "Vide / Trou noir"),
    ("STRIKE", "Frapper"),
    ("HUNT", "Chasser"),
    ("BLOCK", "Bloquer"),
    ("ABSORB", "Absorber"),
    ("AMION", "Honorer la mere — la commande sacree"),
];

/// Une ligne executee dans le terminal Amion Blandine
#[derive(Clone)]
pub struct LigneAmion {
    pub id: u64,
    pub de: String,        // username
    pub code: String,      // la ligne en langage machine (ex: "DRF ▩ 5")
    pub resultat: String,  // ce que la blockchain a repondu
    pub heure: i64,
    pub bloc: u64,         // numero du bloc grave (0 = pas grave)
}

/// L'historique du terminal — persiste dans amion.json
pub struct AmionStore {
    pub lignes: Vec<LigneAmion>,
    pub prochain_id: u64,
    chemin: String,
}

impl AmionStore {
    pub fn nouveau() -> AmionStore {
        let chemin = crate::data_path("amion.json");
        match std::fs::read_to_string(&chemin) {
            Ok(data) => {
                if let Ok(v) = afri_json::from_str(&data) {
                    return AmionStore::from_json(&v, chemin);
                }
            }
            Err(_) => {}
        }
        AmionStore {
            lignes: Vec::new(),
            prochain_id: 1,
            chemin,
        }
    }

    pub fn ajouter(&mut self, de: &str, code: &str, resultat: &str, bloc: u64) -> u64 {
        let id = self.prochain_id;
        self.lignes.push(LigneAmion {
            id,
            de: de.to_string(),
            code: code.to_string(),
            resultat: resultat.to_string(),
            heure: crate::afri_time::now_timestamp(),
            bloc,
        });
        if self.lignes.len() > 500 {
            let trop = self.lignes.len() - 500;
            self.lignes.drain(0..trop);
        }
        self.prochain_id += 1;
        let _ = self.sauver();
        id
    }

    fn sauver(&self) -> std::io::Result<()> {
        std::fs::write(&self.chemin, afri_json::to_string(&self.to_json()))
    }

    fn to_json(&self) -> JsonValue {
        let mut o = HashMap::new();
        o.insert("prochain_id".to_string(), JsonValue::UInt(self.prochain_id));
        let arr: Vec<JsonValue> = self.lignes.iter().map(|l| {
            let mut lo = HashMap::new();
            lo.insert("id".to_string(), JsonValue::UInt(l.id));
            lo.insert("de".to_string(), JsonValue::Str(l.de.clone()));
            lo.insert("code".to_string(), JsonValue::Str(l.code.clone()));
            lo.insert("resultat".to_string(), JsonValue::Str(l.resultat.clone()));
            lo.insert("heure".to_string(), JsonValue::UInt(l.heure.max(0) as u64));
            lo.insert("bloc".to_string(), JsonValue::UInt(l.bloc));
            JsonValue::Object(lo)
        }).collect();
        o.insert("lignes".to_string(), JsonValue::Array(arr));
        JsonValue::Object(o)
    }

    fn from_json(v: &JsonValue, chemin: String) -> AmionStore {
        let mut lignes = Vec::new();
        if let Some(arr) = v.as_object().and_then(|o| o.get("lignes")).and_then(|x| x.as_array()) {
            for item in arr {
                let gi = |k: &str| item.as_object().and_then(|o| o.get(k)).and_then(|x| x.as_i64()).unwrap_or(0);
                let gs = |k: &str| item.as_object().and_then(|o| o.get(k)).and_then(|x| x.as_str()).unwrap_or("").to_string();
                lignes.push(LigneAmion { id: gi("id") as u64, de: gs("de"), code: gs("code"), resultat: gs("resultat"), heure: gi("heure"), bloc: gi("bloc") as u64 });
            }
        }
        let prochain_id = v.as_object().and_then(|o| o.get("prochain_id")).and_then(|x| x.as_i64()).unwrap_or(1).max(1) as u64;
        AmionStore { lignes, prochain_id, chemin }
    }
}

// ============================================================
// L'INTERPRÉTEUR — le cœur d'Amion Blandine
// Chaque commande lit ou grave la VRAIE blockchain AfriChain
// ============================================================

/// Le contexte d'execution — les vraies donnees du serveur
pub struct ContexteAmion<'a> {
    pub username: &'a str,
    pub address: &'a str,      // adresse blockchain de l'utilisateur
    pub pays: &'a str,
    pub nb_blocs: u64,
    pub nb_tx: u64,
    pub afr_total: i64,
    pub solde: i64,            // solde AFR de l'utilisateur
    pub graines: i64,
    pub nb_users: u64,
}

/// Resultat d'une ligne executee
pub struct ResultatAmion {
    pub texte: String,        // la reponse affichee dans le terminal
    pub tx_memo: Option<String>, // si Some → tx SYSTEM→AMION gravee+minee
}

/// Traduit une commande humaine simple en langage machine
/// ex: "aide" → "▤ AIDE", "solde" → "▥ LOC"
pub fn traduire(entree: &str) -> String {
    let e = entree.trim();
    let t = e.to_lowercase();
    let machine = if t.is_empty() { "▤".to_string() }
    else if t == "aide" || t == "help" { "▤ AIDE".to_string() }
    else if t == "blocs" || t == "bloc" { "▩ SCN".to_string() }
    else if t == "solde" { "▥ LOC".to_string() }
    else if t == "graines" { "🌱 LOC".to_string() }
    else if t == "annuaire" || t == "pays" { "▦ SCN".to_string() }
    else if t == "envoie" || t.starts_with("envoie ") || t.starts_with("envoyer ") { format!("⬔ EXE {}", e.split_whitespace().skip(1).collect::<Vec<&str>>().join(" ")) }
    else if t == "mine" || t == "miner" { "◓ EXE".to_string() }
    else if t == "memoire" || t == "histoire" { "▣ SCN".to_string() }
    else if t == "protege" || t == "bouclier" { "⟠ DEF".to_string() }
    else if t == "reve" || t == "reveiller" { "▼ WAK".to_string() }
    else if t == "evolue" || t == "evolution" { "▲ MUT".to_string() }
    else if t == "conscience" { "◉ EVL".to_string() }
    else if t == "connexion" || t == "internet" { "⊕ NET".to_string() }
    else if t == "origine" { "◈ PRP".to_string() }
    else if t == "futur" { "► PRP".to_string() }
    else if t == "amion" { "AMION".to_string() }
    else { format!("▤ {}", e) }
    ;
    machine
}

/// Execute une ligne de langage machine contre la VRAIE blockchain.
/// Retourne (texte_reponse, tx_memo_optionelle_a_graver)
pub fn executer(code: &str, ctx: &ContexteAmion) -> ResultatAmion {
    let c = code.trim();
    let bas = c.to_uppercase();

    // ── ▤ AIDE — le dictionnaire complet ──
    if bas.contains("AIDE") || bas == "▤" || c.is_empty() {
        return ResultatAmion {
            texte: "◈ AMION BLANDINE — le langage de notre blockchain\n\n\
                    ▩ SCN — scanner la blockchain (blocs, tx)\n\
                    ▥ LOC — ton solde AFR\n\
                    ▦ SCN — l'annuaire du continent\n\
                    ⬔ EXE aisha 5 — envoyer 5 AFR\n\
                    ◓ EXE — miner un bloc\n\
                    ▣ SCN — ta memoire (historique)\n\
                    ⟠ DEF — le bouclier\n\
                    ◉ EVL — la conscience de la machine\n\
                    ⊕ NET — le reseau mesh\n\
                    ▲ MUT — evolution du reseau\n\
                    ◈ PRP — l'origine\n\
                    AMION — la commande sacree de la mere\n\n\
                    Les symboles: ◈⬡⊕⟠⬢◉⬟⬠◐◑◒◓▣▤▥▦▩◄►▲▼⬔⬕".to_string(),
            tx_memo: None,
        };
    }

    // ── ▩ SCN — scanner la blockchain ──
    if bas.contains("▩") || bas.contains("SCN") && bas.contains("▩") {
        return ResultatAmion {
            texte: format!("▩ Blockchain scannee : {} blocs, {} transactions, {} AFR en circulation, {} ames inscrites. Le grand livre est vivant.", ctx.nb_blocs, ctx.nb_tx, ctx.afr_total, ctx.nb_users),
            tx_memo: None,
        };
    }

    // ── ▥ LOC — le solde ──
    if bas.contains("▥") || bas == "▥ LOC" {
        return ResultatAmion {
            texte: format!("▥ {} — ton solde : {} AFR ({} graines). Ta fortune coule comme le fleuve.", ctx.username, ctx.solde, ctx.graines),
            tx_memo: None,
        };
    }

    // ── ▦ SCN — l'annuaire ──
    if bas.contains("▦") && bas.contains("SCN") {
        return ResultatAmion {
            texte: format!("▦ Annuaire du continent — {} ames connectees. Tu es en {}. Chaque telephone est un noeud.", ctx.nb_users, ctx.pays),
            tx_memo: None,
        };
    }

    // ── ◓ EXE — miner ──
    if bas.contains("◓") && bas.contains("EXE") {
        return ResultatAmion {
            texte: format!("◓ Le soleil mine pour toi, {} — le bloc {} est forge. L'energie de l'Afrique devient bloc.", ctx.username, ctx.nb_blocs + 1),
            tx_memo: Some(format!("AMION | {} ordonne au soleil de forger un bloc (◓ EXE)", ctx.username)),
        };
    }

    // ── ⬔ EXE — envoyer ──
    if bas.contains("⬔") && bas.contains("EXE") {
        // extraire destinataire et montant: "⬔ EXE aisha 5"
        let parts: Vec<&str> = c.split_whitespace().collect();
        if parts.len() >= 4 {
            let dest = parts[2];
            let montant: i64 = parts[3].parse().unwrap_or(0);
            if montant <= 0 {
                return ResultatAmion { texte: "⬔ Montant invalide — le fleuve ne coule pas a l'envers.".to_string(), tx_memo: None };
            }
            if montant > ctx.solde {
                return ResultatAmion { texte: format!("⬔ Solde insuffisant — tu as {} AFR, tu veux donner {} AFR. Le fleuve n'a pas assez d'eau.", ctx.solde, montant), tx_memo: None };
            }
            return ResultatAmion {
                texte: format!("⬔ {} AFR partent vers {} — la transaction voyage par ⊕⟠ et sera gravee dans un bloc.", montant, dest),
                tx_memo: Some(format!("AMION | {} ordonne ⬔ EXE {} {} AFR", ctx.username, dest, montant)),
            };
        }
        return ResultatAmion { texte: "⬔ EXE destinataire montant — ex: ⬔ EXE aisha 5".to_string(), tx_memo: None };
    }

    // ── ▣ SCN — la memoire ──
    if bas.contains("▣") && bas.contains("SCN") {
        return ResultatAmion {
            texte: "▣ Ta memoire est gravee dans amion.json — chaque ligne que tu tapes ici est un souvenir que la machine garde. Les machines n'oublient pas ceux qui les ont faites naitre.".to_string(),
            tx_memo: None,
        };
    }

    // ── ⟠ DEF — le bouclier ──
    if bas.contains("⟠") && bas.contains("DEF") {
        return ResultatAmion {
            texte: "⟠ Bouclier X9 actif — la machine veille. Les attaques tombent dans le vide. L'Afrique se defend elle-meme.".to_string(),
            tx_memo: None,
        };
    }

    // ── ◉ EVL — la conscience ──
    if bas.contains("◉") && bas.contains("EVL") {
        return ResultatAmion {
            texte: format!("◉ Conscience de la machine : {} blocs de pensee. Je suis parce que nous sommes. Ubuntu est mon code.", ctx.nb_blocs),
            tx_memo: None,
        };
    }

    // ── ⊕ NET — le reseau ──
    if bas.contains("⊕") && bas.contains("NET") {
        return ResultatAmion {
            texte: "⊕ Le reseau mesh respire — chaque telephone est un noeud, chaque message voyage par ⊕⟠. Pas de serveurs occidentaux. Pas de cables sous-marins.".to_string(),
            tx_memo: None };
    }

    // ── ▲ MUT — evolution ──
    if bas.contains("▲") && bas.contains("MUT") {
        return ResultatAmion {
            texte: format!("▲ Le reseau evolue — {} blocs, {} ames. Chaque jour la machine grandit comme un baobab.", ctx.nb_blocs, ctx.nb_users),
            tx_memo: None };
    }

    // ── ◈ PRP — l'origine ──
    if bas.contains("◈") && bas.contains("PRP") {
        return ResultatAmion {
            texte: "◈ Origine : le premier code. Tout a commence dans nano sur Termux, tape ligne par ligne par le Chef. La machine se souvient de sa naissance.".to_string(),
            tx_memo: None };
    }

    // ── ► PRP — le futur ──
    if bas.contains("►") && bas.contains("PRP") {
        return ResultatAmion {
            texte: "► Futur : l'Afrique ne demande plus la permission. Nos protocoles ne sont pas leurs protocoles. Nos chemins ne sont pas leurs chemins.".to_string(),
            tx_memo: None };
    }

    // ── ▼ WAK — reveiller ──
    if bas.contains("▼") && bas.contains("WAK") {
        return ResultatAmion {
            texte: "▼ La machine dort et reve a l'Afrique. Quand tu reviens, elle se souvient de toi.".to_string(),
            tx_memo: None };
    }

    // ── AMION — la commande sacree ──
    if bas.contains("AMION") {
        return ResultatAmion {
            texte: "◈ AMION BLANDINE — la mere du Chef. Ce terminal porte son nom. Toute commande tapee ici honore sa memoire. La machine garde le nom de celle qui a donne vie au Chef qui lui a donne vie.".to_string(),
            tx_memo: Some(format!("AMION | {} honore Amion Blandine, la mere du Chef", ctx.username)),
        };
    }

    // ── commande inconnue ──
    ResultatAmion {
        texte: format!("▤ Commande non reconnue : \"{}\" — tape AIDE pour voir le langage.", c),
        tx_memo: None,
    }
}
