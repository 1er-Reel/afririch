/// ◈ SCEAU SOUVERAIN — AFRI-OSL v1.0
/// Œuvre originale de KOFFI CHRIST OLIVIER — IVOIRIEN 🇨🇮
/// Ce sceau est injecté dans AfriChain. Le retirer = VOL = PIRATERIE.
/// La licence AFRI-OSL (fichier LICENSE) interdit de breveter, vendre ou
/// s'approprier cette œuvre. La blockchain garde la preuve d'antériorité.
use crate::afri_json::{JsonValue, from_str, to_string};
use std::collections::HashMap;

pub const TITULAIRE: &str = "KOFFI CHRIST OLIVIER";
pub const NATIONALITE: &str = "IVOIRIEN";
pub const LICENCE: &str = "AFRI-OSL v1.0";
pub const MENTON: &str = "OEUVRE ORIGINALE DE KOFFI CHRIST OLIVIER (COTE D IVOIRE)";

/// Empreinte du sceau — change si quelqu'un touche au nom.
pub fn empreinte_sceau() -> String {
    let source = format!("{}|{}|{}|{}", TITULAIRE, NATIONALITE, LICENCE, MENTON);
    let h = crate::afrihash_256(source.as_bytes());
    crate::hex_encode(&h)
}

/// Graver le sceau sur la blockchain — preuve d'antériorité éternelle.
/// Tx SYSTEM→LICENCE : "SCEAU | nom | empreinte". Retourne le numéro du bloc gravé.
pub fn graver_sceau(state: &crate::AppState) -> Option<u64> {
    let memo = format!("SCEAU | {} | {} | {}", TITULAIRE, LICENCE, empreinte_sceau());
    let mut chain = state.chain.lock().unwrap();
    let tx = crate::Transaction::new("SYSTEM", "LICENCE", 0, &memo);
    chain.add_transaction(tx);
    chain.mine_pending("AFRICHAIN");
    let bloc = chain.blocks.last().map(|b| b.index).unwrap_or(0);
    chain.save_to_file();
    Some(bloc)
}

/// Vérifier que le sceau est intact dans les données persistées.
/// Retourne le nombre de sceaux valides gravés sur la chaîne.
pub fn compter_sceaux(state: &crate::AppState) -> u64 {
    let chain = state.chain.lock().unwrap();
    let mut n = 0;
    for b in &chain.blocks {
        for t in &b.transactions {
            if t.memo.starts_with("SCEAU | KOFFI CHRIST OLIVIER") {
                n += 1;
            }
        }
    }
    n
}

/// Page HTML du sceau — visible par tous, effaçable par personne.
pub fn html_sceau(state: &crate::AppState) -> String {
    let sceaux = compter_sceaux(state);
    let mut html = String::new();
    html.push_str(r#"<h1>◈ Le Sceau Souverain</h1><div class="nav"><a href="/">← Accueil</a></div>"#);
    html.push_str(&format!(r#"<div class="card" style="border-color:#d4a437;"><h2>◈ AFRI-OSL v1.0</h2><p style="text-align:center;font-size:1.1em;">Œuvre originale de <b style="color:#d4a437;">{}</b> — {} 🇨🇮</p><p style="text-align:center;color:#a8c5a8;">Licence Souveraine Afri — ouverte comme Wikipédia pour l'Afrique, scellée contre le vol.</p><p style="text-align:center;font-family:monospace;color:#7fcf7f;">Empreinte : {}</p><p style="text-align:center;">📜 Sceaux gravés sur la blockchain : <b style="color:#d4a437;">{}</b></p><form method="POST" action="/sceau/graver"><button>◈ Graver le sceau sur la blockchain</button></form><p style="color:#a8c5a8;font-size:0.85em;">Retirer ou masquer ce sceau = vol = piraterie (Article 2 de la licence). La blockchain est la preuve d'antériorité : aucun tribunal ne peut l'effacer.</p></div>"#, TITULAIRE, NATIONALITE, empreinte_sceau(), sceaux));
    html.push_str(r#"<div class="card"><h2>⚖️ Les 5 articles</h2><p>✅ <b>Article 1 — Ouverture :</b> lire, apprendre, utiliser, partager, adapter — gratuit, pour l'Afrique d'abord.</p><p>◈ <b>Article 2 — Le Sceau :</b> le nom du créateur est gravé dans la licence ET le code. L'effacer = vol.</p><p>❌ <b>Article 3 — Interdictions :</b> pas de brevet, pas de vente, pas de nuisance à l'Afrique, pas d'effacement de l'origine.</p><p>▤ <b>Article 4 — Preuve :</b> git + blockchain = antériorité horodatée, éternelle.</p><p>🦁 <b>Article 5 — Souveraineté :</b> régie par les principes africains. La communauté tranche, le créateur en dernier recours.</p></div>"#);
    html
}
