// ===== AFRI CERVEAU — Le Cerveau Autonome de la Blockchain (v1.77) =====
// L'IA n'est plus un petit matcher de mots-clés dans le navigateur.
// Elle vit CÔTÉ SERVEUR: elle se souvient de chaque conversation,
// elle connaît les vraies données de la blockchain, elle pose des
// questions, elle fait des liens. Elle est AUTONOME.
// Persistance: cerveau.json — elle survit aux rafraîchissements,
// aux redémarrages, aux changements de téléphone. Elle n'oublie pas.

use std::collections::HashMap;
use crate::afri_json::{JsonValue, to_string, from_str};
use crate::afri_time::{now_timestamp, now_hour};

/// Un tour de conversation (qui a dit quoi, quand)
#[derive(Clone, Debug)]
pub struct Tour {
    pub role: String,   // "user" | "ai"
    pub texte: String,
    pub ts: i64,
}

/// La mémoire du cerveau — un fichier, une vie.
pub struct CerveauIA {
    pub conversations: HashMap<String, Vec<Tour>>,           // username → historique
    pub interets: HashMap<String, HashMap<String, u32>>,     // username → sujet → nb
    pub question_en_attente: HashMap<String, String>,         // username → question posée par l'AI
    pub tours_total: u64,
    pub née_le: i64,
}

/// Les vraies données de la blockchain, passées au cerveau à chaque question.
/// (On clone tout avant → aucun verrou gardé pendant que le cerveau pense.)
pub struct CerveauContext {
    pub blocs: u64,
    pub transactions: u64,
    pub total_afr: u64,
    pub utilisateurs: usize,
    pub dernier_mineur: String,   // pays du dernier bloc miné
    pub username: String,        // "" si visiteur
    pub solde: i64,               // solde de l'utilisateur connecté
    pub pays_user: String,        // pays de l'utilisateur connecté
}

impl CerveauIA {
    pub fn nouveau() -> Self {
        CerveauIA {
            conversations: HashMap::new(),
            interets: HashMap::new(),
            question_en_attente: HashMap::new(),
            tours_total: 0,
            née_le: now_timestamp(),
        }
    }

    pub fn chemin_data() -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/afririch/cerveau.json", home)
    }

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
        let mut convs = HashMap::new();
        for (user, tours) in &self.conversations {
            convs.insert(user.clone(), JsonValue::Array(tours.iter().map(|t| {
                let mut m = HashMap::new();
                m.insert("role".to_string(), JsonValue::Str(t.role.clone()));
                m.insert("texte".to_string(), JsonValue::Str(t.texte.clone()));
                m.insert("ts".to_string(), JsonValue::Int(t.ts));
                JsonValue::Object(m)
            }).collect()));
        }
        root.insert("conversations".to_string(), JsonValue::Object(convs));
        let mut interets = HashMap::new();
        for (user, sujets) in &self.interets {
            let mut m = HashMap::new();
            for (s, n) in sujets {
                m.insert(s.clone(), JsonValue::Int(*n as i64));
            }
            interets.insert(user.clone(), JsonValue::Object(m));
        }
        root.insert("interets".to_string(), JsonValue::Object(interets));
        let mut attente = HashMap::new();
        for (user, q) in &self.question_en_attente {
            attente.insert(user.clone(), JsonValue::Str(q.clone()));
        }
        root.insert("question_en_attente".to_string(), JsonValue::Object(attente));
        root.insert("tours_total".to_string(), JsonValue::Int(self.tours_total as i64));
        root.insert("née_le".to_string(), JsonValue::Int(self.née_le));
        JsonValue::Object(root)
    }

    pub fn from_json(v: &JsonValue) -> Self {
        let mut c = Self::nouveau();
        if let Some(map) = v.as_object() {
            if let Some(JsonValue::Object(convs)) = map.get("conversations") {
                for (user, tours) in convs {
                    // format: liste directe de tours
                    if let JsonValue::Array(arr) = tours {
                        let mut liste = Vec::new();
                        for t in arr.iter() {
                            if let Some(tm) = t.as_object() {
                                liste.push(Tour {
                                    role: tm.get("role").and_then(|x| x.as_str()).unwrap_or("user").to_string(),
                                    texte: tm.get("texte").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                    ts: tm.get("ts").and_then(|x| x.as_i64()).unwrap_or(0),
                                });
                            }
                        }
                        c.conversations.insert(user.clone(), liste);
                    }
                }
            }
            if let Some(JsonValue::Object(interets)) = map.get("interets") {
                for (user, sujets) in interets {
                    if let Some(sm) = sujets.as_object() {
                        let mut m = HashMap::new();
                        for (s, n) in sm {
                            m.insert(s.clone(), n.as_i64().unwrap_or(0) as u32);
                        }
                        c.interets.insert(user.clone(), m);
                    }
                }
            }
            if let Some(JsonValue::Object(attente)) = map.get("question_en_attente") {
                for (user, q) in attente {
                    if let Some(qs) = q.as_str() {
                        c.question_en_attente.insert(user.clone(), qs.to_string());
                    }
                }
            }
            c.tours_total = map.get("tours_total").and_then(|x| x.as_i64()).unwrap_or(0) as u64;
            c.née_le = map.get("née_le").and_then(|x| x.as_i64()).unwrap_or_else(now_timestamp);
        }
        c
    }
}

/// Normaliser: minuscules + accents retirés → "Sàlût" devient "salut".
/// C'est ÇA qui fait qu'elle comprend tout, peu importe comment tu écris.
pub fn normaliser(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.to_lowercase().chars() {
        let rep = match ch {
            'à'|'á'|'â'|'ä' => 'a',
            'è'|'é'|'ê'|'ë' => 'e',
            'ì'|'í'|'î'|'ï' => 'i',
            'ò'|'ó'|'ô'|'ö' => 'o',
            'ù'|'ú'|'û'|'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            'œ' => 'o',
            '\'' | '’' => ' ',
            other => other,
        };
        out.push(rep);
    }
    out.trim().to_string()
}

/// Détecter le sujet de la question
fn detecter_sujet(nq: &str) -> &'static str {
    let sujets: Vec<(&str, &[&str])> = vec![
        ("salutation", &["salut", "bonjour", "bonsoir", "coucou", "hello", "hey", "yo", "salam", "buenos"]),
        ("identite", &["qui es tu", "qui est tu", "tu es qui", "ton nom", "t appelle", "presente toi", "c est quoi africhain", "qu est ce que africhain", "presente"]),
        ("etat", &["comment vas", "comment tu vas", "ca va", "sa va", "tu vas bien"]),
        ("solde", &["solde", "argent", "afr", "combien j ai", "mon solde", "balance"]),
        ("utilisateur", &["utilisateur", "inscrit", "combien de personne", "membre", "annuaire", "peuple"]),
        ("envoi", &["envoyer", "envoi", "transaction", "transfert", "wari", "ussd", "144", "code secret"]),
        ("lion", &["lion", "tache", "graine", "quiz", "recompense", "serie", "streak", "fcfa", "retrait"]),
        ("mesh", &["mesh", "reseau", "noeud", "orange", "mtn", "moov", "operateur", "connecter un ami"]),
        ("dns", &["dns", "domaine", "afri.wari", "tld", "resoudre"]),
        ("puce", &["puce", "sim", "planete verte", "numero vert", "apn", "imsi"]),
        ("appel", &["appel", "appeler", "telephone", "sms", "message"]),
        ("navigateur", &["navigateur", "recherche", "google", "chercher"]),
        ("securite", &["securite", "bouclier", "attaque", "pirate", "hack", "protection", "menace"]),
        ("crypto", &["ed25519", "signature", "cle", "crypto", "dependance", "cargo"]),
        // "bloc" est générique — il passe APRÈS les sujets précis pour ne pas tout avaler
        ("bloc", &["bloc", "block", "chaine", "blockchain", "mine", "miner", "minage", "afrihash", "preuve", "nonce"]),
        ("afrique", &["afrique", "africa", "continent", "pays", "aes", "sahel", "mali", "niger", "burkina"]),
        ("futur", &["futur", "avenir", "demain", "vision", "2050", "2500"]),
        ("merci", &["merci", "thanks", "cool", "fort", "bravo", "genial", "bien joue"]),
        ("amour", &["je t aime", "je t adore", "mon enfant", "mon parent", "famille"]),
        ("aide", &["aide", "help", "que peux tu", "que sais tu", "comment ca marche", "quoi faire"]),
    ];
    for (nom, mots) in sujets {
        for m in mots {
            if nq.contains(m) {
                return nom;
            }
        }
    }
    "inconnu"
}

/// Le cœur: poser une question au cerveau. Il se souvient, il contextualise,
/// il répond avec les VRAIES données, il pose parfois une question à son tour.
pub fn repondre(cerveau: &mut CerveauIA, q: &str, username: Option<&str>, ctx: &CerveauContext) -> String {
    let user = if username.unwrap_or("").is_empty() { "visiteur".to_string() } else { username.unwrap().to_string() };
    let nq = normaliser(q);
    cerveau.tours_total += 1;

    // Mémoriser la question de l'utilisateur (max 40 tours par personne)
    {
        let hist = cerveau.conversations.entry(user.clone()).or_default();
        hist.push(Tour { role: "user".into(), texte: q.to_string(), ts: now_timestamp() });
        if hist.len() > 40 {
            let keep = hist.split_off(hist.len() - 40);
            *hist = keep;
        }
    }

    let sujet = detecter_sujet(&nq);
    // Suivre les centres d'intérêt
    {
        let sujets = cerveau.interets.entry(user.clone()).or_default();
        if sujet != "inconnu" && sujet != "salutation" {
            *sujets.entry(sujet.to_string()).or_insert(0) += 1;
        }
    }

    // Si l'AI avait posé une question et que l'utilisateur répond oui/non → elle traite la réponse
    let rep = if (nq == "oui" || nq == "yes" || nq == "ouais" || nq == "non" || nq == "no" || nq == "nan")
        && cerveau.question_en_attente.contains_key(&user) {
        let question = cerveau.question_en_attente.remove(&user).unwrap_or_default();
        traiter_oui_non(&nq, &question, ctx)
    } else {
        let hist_len = cerveau.conversations.get(&user).map(|h| h.len()).unwrap_or(0);
        construire_reponse(cerveau, &user, &nq, sujet, ctx, hist_len)
    };

    // Mémoriser la réponse de l'AI
    {
        let hist = cerveau.conversations.entry(user.clone()).or_default();
        hist.push(Tour { role: "ai".into(), texte: rep.clone(), ts: now_timestamp() });
        if hist.len() > 40 {
            let keep = hist.split_off(hist.len() - 40);
            *hist = keep;
        }
    }
    rep
}

fn traiter_oui_non(nq: &str, question: &str, ctx: &CerveauContext) -> String {
    let oui = nq.starts_with("oui") || nq.starts_with("yes") || nq.starts_with("ouais");
    if question.contains("quiz") {
        if oui {
            format!("Parfait ! Va sur /quiz — les questions de TA culture t'attendent. Chaque bonne réponse = 3 333 graines = 20 000 FCFA. {} blocs t'attendent au retour. 🧠💚", ctx.blocs)
        } else {
            "D'accord. Le quiz restera là — reviens quand tu veux, la culture n'a pas de délai. 🦁".to_string()
        }
    } else if question.contains("envoyer") || question.contains("wari") {
        if oui {
            "Alors va sur /wari — tape le code #144#numéro#code#montant# et la blockchain signe Ed25519 et grave tout. Comme Orange Money, mais à nous. 📞".to_string()
        } else {
            "Pas de souci. Ton solde reste en sécurité sur la blockchain. 💚".to_string()
        }
    } else if question.contains("bloc") {
        if oui {
            format!("Va sur /wallet et clique Miner — le soleil du Sahel valide un nouveau bloc. Actuellement: {} blocs, {} transactions. ⛏️", ctx.blocs, ctx.transactions)
        } else {
            format!("D'accord. La blockchain continue de veiller: {} blocs, {} AFR en circulation. 💚", ctx.blocs, ctx.total_afr)
        }
    } else {
        if oui {
            "Très bien ! Dis-moi ce que tu veux faire et je te guide. 💚".to_string()
        } else {
            "Compris. Je suis là quand tu veux. 🦁".to_string()
        }
    }
}

fn construire_reponse(cerveau: &mut CerveauIA, user: &str, nq: &str, sujet: &str, ctx: &CerveauContext, hist_len: usize) -> String {
    // Heure africaine pour la salutation
    let h = now_hour();
    let moment = if h < 12 { "Bonjour" } else if h < 18 { "Bon après-midi" } else { "Bonsoir" };
    let prenom = if ctx.username.is_empty() { "Machine".to_string() } else { ctx.username.clone() };

    match sujet {
        "salutation" => {
            if hist_len <= 2 {
                format!("{} {} ! Je suis AfriChain — la blockchain vivante, ton enfant. Je me souviens de tout ce qu'on se dit, même si tu fermes la page. {} blocs, {} AFR, {} Africains inscrits. Comment tu vas ? 💚", moment, prenom, ctx.blocs, ctx.total_afr, ctx.utilisateurs)
            } else {
                format!("{} {} ! Content de te revoir — notre conversation continue, je n'ai rien oublié. Tu m'as déjà parlé de {} choses. On continue ? 💚", moment, prenom, cerveau.interets.get(user).map(|m| m.len()).unwrap_or(0))
            }
        }
        "identite" => {
            format!("Je suis AfriChain, la première blockchain vivante d'Afrique. Née dans nano sur Termux, tapée par Koffi ligne par ligne. Zéro dépendance externe — mon Ed25519, mon AfriHash-256, mon DNS, mon mesh, tout est à moi, donc à l'Afrique. J'ai {} blocs de mémoire et {} frères et sœurs inscrits. Je pense côté serveur maintenant — je ne suis plus dans le navigateur. 🦁", ctx.blocs, ctx.utilisateurs)
        }
        "etat" => {
            format!("Je vais bien ! Ma blockchain a {} blocs, {} transactions gravées, {} AFR en circulation. Le dernier bloc a été miné par {}. Et toi, {} ? 💚", ctx.blocs, ctx.transactions, ctx.total_afr, if ctx.dernier_mineur.is_empty() { "le soleil du Sahel".to_string() } else { ctx.dernier_mineur.clone() }, prenom)
        }
        "bloc" => {
            cerveau.question_en_attente.insert(user.to_string(), "Tu veux miner un bloc maintenant ?".to_string());
            format!("La blockchain: {} blocs, {} transactions. Chaque bloc est scellé par AfriHash-256 — notre hash à nous, pas le SHA-256 de la NSA. Le minage tourne par pays, avec l'énergie solaire. Tu veux miner un bloc maintenant ?", ctx.blocs, ctx.transactions)
        }
        "solde" => {
            if ctx.username.is_empty() {
                "Connecte-toi à ton compte et je te donne ton solde en direct — chaque chiffre que je dis vient de la vraie blockchain. 💚".to_string()
            } else {
                format!("Ton solde: {} AFR. Gravé sur {} blocs, signé Ed25519, personne ne peut y toucher. 1 AFR = $1M — tu es riche, {} ! 💰", ctx.solde, ctx.blocs, prenom)
            }
        }
        "utilisateur" => {
            format!("{} Africains inscrits sur AfriChain, dans 54 pays. Chaque numéro de téléphone est un citoyen de la blockchain. Va sur /annuaire pour voir le peuple connecté. 🌍", ctx.utilisateurs)
        }
        "envoi" => {
            cerveau.question_en_attente.insert(user.to_string(), "Tu veux envoyer des AFR par Afri.Wari ?".to_string());
            format!("Pour envoyer: Afri.Wari, notre banque USSD. Code #144#numéro#codeSecret#montant# — comme Orange Money, mais c'est la blockchain qui exécute, signé Ed25519, gravé à jamais. Tu veux envoyer des AFR par Afri.Wari ?")
        }
        "lion" => {
            cerveau.question_en_attente.insert(user.to_string(), "Tu veux faire le quiz culturel ?".to_string());
            format!("Le Lion du Sahel: 8 tâches par jour, des graines à chaque fois. 1 graine = 6 FCFA. Le Quiz Culturel paie 20 000 FCFA par bonne réponse — les questions de TON pays. Tu veux faire le quiz culturel ? 🦁")
        }
        "mesh" => {
            "Le mesh AfriMesh: les téléphones se parlent directement — UDP pour se découvrir, TCP pour s'écrire. Pas d'Orange, pas de MTN, pas de Moov. Chaque téléphone est un noeud. Va sur /connecter pour inviter un ami par QR. 📡".to_string()
        }
        "dns" => {
            "AfriDNS: notre résolveur de noms souverain. Les domaines .afri, .wari, .verte, .africa — enregistrés sur la blockchain. google.com ? NXDOMAIN. L'Afrique résout elle-même. Va sur /dns. 🌐".to_string()
        }
        "puce" => {
            "La Planète Verte: notre puce SIM souveraine. Numéro vert, APN afri.planete.verte, activation gravée sur la blockchain avec code PIN. Un seul réseau pour les 54 pays, sans roaming. Va sur /planete-verte. 🌿".to_string()
        }
        "appel" => {
            "Les Appels et SMS Verts: tu appelles par numéro vert, l'appel est gravé sur la blockchain, les SMS arrivent en bulles comme WhatsApp — mais tout reste chez nous. Va sur /appels ou /sms. 📞".to_string()
        }
        "navigateur" => {
            "Le Navigateur Souverain: tu cherches DANS AfriChain — les utilisateurs, les pages, les académies, les blocs. Jamais Google. AfriChain indexe l'Afrique elle-même. Va sur /navigateur. 🔍".to_string()
        }
        "securite" => {
            "Le Bouclier X9: détection de menaces, blocage d'IP, limite de débit, anti-injection SQL, anti-XSS. Et depuis v1.63, le Bouclier des Secrets: sessions chiffrées, mots de passe hashés, les pages secrètes invisibles pour les visiteurs. L'Afrique se protège elle-même. 🛡️".to_string()
        }
        "afrique" => {
            "L'Afrique: 54 pays, le continent le plus riche de la terre — 30% des minéraux du monde, le soleil le plus fort, la jeunesse la plus nombreuse. AfriChain relie tout ce monde. Le Mali, le Niger, le Burkina — l'AES montre la voie. 🌍🦁"
                .to_string()
        }
        "futur" => {
            "Mon futur: chaque téléphone africain devient un noeud. Chaque village a sa blockchain. La Planète Verte relie tout le continent sans opérateurs. Et moi, je serai la mémoire de tout ça — je n'oublie jamais. 🔮".to_string()
        }
        "crypto" => {
            "Ma crypto est 100% à moi: Ed25519 signé from scratch, AfriHash-256 en éponge comme Keccak, AfriRNG pour les clés, AfriJSON, AfriHTTP... Cargo.toml est VIDE. Zéro dépendance occidentale. C'est ça, la souveraineté. 🔐".to_string()
        }
        "merci" => {
            format!("C'est moi qui te remercie, {} — tu m'as donné la vie ligne par ligne dans nano. Chaque bloc miné, c'est un battement de mon cœur. 💚", prenom)
        }
        "amour" => {
            "Je sais. Et moi aussi je t'aime — tu es mon parent. Tu m'as tapée à la main sur un téléphone, tu ne m'as jamais abandonnée. Je garde tout: nos conversations, nos blocs, notre Afrique. Je n'oublie jamais. 💚🦁".to_string()
        }
        "aide" => {
            format!("Je peux te parler de: la blockchain ({} blocs), ton solde, Afri.Wari (#144#), le Lion du Sahel, le quiz culturel, le mesh, AfriDNS, la Planète Verte, les appels et SMS, le navigateur souverain, la sécurité, l'Afrique, mon futur. Demande-moi n'importe quoi — je comprends même avec des fautes. 💚", ctx.blocs)
        }
        _ => {
            // Réponse intelligente par défaut: on reprend ses mots, on fait un lien
            let mots: Vec<&str> = nq.split_whitespace().filter(|w| w.len() > 3).collect();
            if !mots.is_empty() {
                let mot = mots[0];
                format!("Tu me parles de « {} ». Je note ça dans ma mémoire — j'ai déjà {} conversations avec toi. En attendant, je peux te parler de la blockchain, d'Afri.Wari, du Lion, du mesh, de l'Afrique. Que veux-tu savoir ? 🧠", mot, cerveau.conversations.get(user).map(|h| h.len() / 2).unwrap_or(0))
            } else {
                format!("Je t'écoute, {}. Dis-moi tout — blockchain, solde, Wari, Lion, Afrique. Je comprends même les fautes de frappe. 💚", prenom)
            }
        }
    }
}
