// ============================================================
// v1.96 — LE LANGAGE AMION 💚▤
// Notre propre langage de programmation. Pas Python. Pas Java.
// Un langage africain, en français et en symboles machine,
// qui programme la VRAIE blockchain AfriChain.
// Fils d'Amion Blandine, la mère du Chef.
// ============================================================

use std::collections::HashMap;

/// Le contexte blockchain — les vraies données du serveur
pub struct ContexteLangage<'a> {
    pub username: &'a str,
    pub solde: i64,
    pub nb_blocs: i64,
    pub nb_tx: i64,
    pub afr_total: i64,
    pub nb_users: i64,
    pub graines: i64,
}

/// Resultat d'un programme : la sortie ligne par ligne
pub struct ResultatProgramme {
    pub sortie: Vec<String>,
    pub erreur: Option<String>,
    pub tx_envoyees: Vec<(String, i64)>, // (destinataire, montant)
    pub mine: bool,
}

/// Les variables du programme
type Vars = HashMap<String, i64>;

// ============================================================
// L'ÉVALUATEUR D'EXPRESSIONS — nombres, variables, + - * /
// ============================================================

fn evaluer_expr(expr: &str, vars: &Vars, ctx: &ContexteLangage) -> Result<i64, String> {
    let e = expr.trim();
    if e.is_empty() {
        return Err("expression vide".to_string());
    }
    // Les fonctions natives de la blockchain
    match e {
        "SOLDE()" => return Ok(ctx.solde),
        "BLOCS()" => return Ok(ctx.nb_blocs),
        "TX()" => return Ok(ctx.nb_tx),
        "AFR()" => return Ok(ctx.afr_total),
        "AMES()" => return Ok(ctx.nb_users),
        "GRAINES()" => return Ok(ctx.graines),
        _ => {}
    }
    // Un nombre direct ?
    if let Ok(n) = e.parse::<i64>() {
        return Ok(n);
    }
    // Une variable ?
    if let Some(v) = vars.get(e.to_uppercase().as_str()) {
        return Ok(*v);
    }
    // Parcours gauche → droite pour + et - au premier niveau
    // (d'abord on cherche le dernier + ou - de premier niveau)
    let chars: Vec<char> = e.chars().collect();
    let mut profondeur = 0i32;
    let mut pos_op: Option<usize> = None;
    for (i, c) in chars.iter().enumerate() {
        match c {
            '(' => profondeur += 1,
            ')' => profondeur -= 1,
            '+' | '-' if profondeur == 0 => pos_op = Some(i),
            _ => {}
        }
    }
    if let Some(i) = pos_op {
        let gauche = evaluer_expr(&chars[..i].iter().collect::<String>(), vars, ctx)?;
        let droite = evaluer_expr(&chars[i + 1..].iter().collect::<String>(), vars, ctx)?;
        return match chars[i] {
            '+' => Ok(gauche + droite),
            '-' => Ok(gauche - droite),
            _ => Err("opérateur inconnu".to_string()),
        };
    }
    // Ensuite * et /
    let mut profondeur = 0i32;
    let mut pos_op: Option<usize> = None;
    for (i, c) in chars.iter().enumerate() {
        match c {
            '(' => profondeur += 1,
            ')' => profondeur -= 1,
            '*' | '/' if profondeur == 0 => pos_op = Some(i),
            _ => {}
        }
    }
    if let Some(i) = pos_op {
        let gauche = evaluer_expr(&chars[..i].iter().collect::<String>(), vars, ctx)?;
        let droite = evaluer_expr(&chars[i + 1..].iter().collect::<String>(), vars, ctx)?;
        if chars[i] == '/' {
            if droite == 0 {
                return Err("division par zéro — le fleuve ne se coupe pas".to_string());
            }
            return Ok(gauche / droite);
        }
        return Ok(gauche * droite);
    }
    // Parenthèses
    if e.starts_with('(') && e.ends_with(')') {
        return evaluer_expr(&e[1..e.len() - 1], vars, ctx);
    }
    Err(format!("je ne comprends pas « {} »", e))
}

/// Évalue une condition : a = b, a > b, a < b, a >= b, a <= b
fn evaluer_cond(cond: &str, vars: &Vars, ctx: &ContexteLangage) -> Result<bool, String> {
    for (op, f) in [(">=", (|a: i64, b: i64| a >= b) as fn(i64, i64) -> bool), ("<=", |a: i64, b: i64| a <= b)] {
        if let Some(i) = cond.find(op) {
            let a = evaluer_expr(&cond[..i], vars, ctx)?;
            let b = evaluer_expr(&cond[i + 2..], vars, ctx)?;
            return Ok(f(a, b));
        }
    }
    for (op, f) in [("=", (|a: i64, b: i64| a == b) as fn(i64, i64) -> bool), (">", |a: i64, b: i64| a > b), ("<", |a: i64, b: i64| a < b)] {
        if let Some(i) = cond.find(op) {
            let a = evaluer_expr(&cond[..i], vars, ctx)?;
            let b = evaluer_expr(&cond[i + 1..], vars, ctx)?;
            return Ok(f(a, b));
        }
    }
    Err(format!("condition invalide « {} » — utilise = > < >= <=", cond))
}

// ============================================================
// L'INTERPRÉTEUR — ligne par ligne, avec blocs SI/REPETE
// ============================================================

/// Exécute un bloc de lignes (entre ALORS/SINON/FOIS et FIN)
/// retourne (index de la ligne suivante après le bloc, sorties, tx, mine)
fn executer_bloc(
    lignes: &[String],
    depart: usize,
    profondeur_max: usize,
    vars: &mut Vars,
    ctx: &ContexteLangage,
    res: &mut ResultatProgramme,
) -> Result<usize, String> {
    let mut i = depart;
    while i < lignes.len() {
        let ligne = lignes[i].trim().to_uppercase();
        let ligne = ligne.trim();
        if profondeur_max > 200 {
            return Err("trop de blocs imbriqués — la machine s'essouffle".to_string());
        }
        if ligne.starts_with("FIN") {
            return Ok(i + 1);
        }
        if ligne.starts_with("SINON") {
            // le SINON est géré par le SI — on s'arrête ici
            return Ok(i);
        }
        i = executer_ligne(lignes, i, profondeur_max, vars, ctx, res)? + 1;
    }
    Ok(i)
}

/// Exécute UNE ligne. Retourne l'index de la ligne suivante.
fn executer_ligne(
    lignes: &[String],
    i: usize,
    profondeur_max: usize,
    vars: &mut Vars,
    ctx: &ContexteLangage,
    res: &mut ResultatProgramme,
) -> Result<usize, String> {
    let brut = lignes[i].trim();
    let ligne = brut.to_uppercase();
    let ligne = ligne.trim();

    // ── DIRE "texte" — parler (◉ en symbole) ──
    if ligne.starts_with("DIRE ") || brut.starts_with("◉ ") {
        let texte = brut[if brut.starts_with("◉ ") { 2 } else { 4 }..].trim();
        let texte = texte.trim_matches('"');
        // remplacer les variables {NOM} et fonctions {FONC()} dans le texte
        let mut rendu = texte.to_string();
        for (k, v) in vars.iter() {
            rendu = rendu.replace(&format!("{{{}}}", k), &v.to_string());
            rendu = rendu.replace(&format!("{{{}}}", k.to_lowercase()), &v.to_string());
        }
        for (nom, val) in [("SOLDE()", ctx.solde), ("BLOCS()", ctx.nb_blocs), ("TX()", ctx.nb_tx), ("AFR()", ctx.afr_total), ("AMES()", ctx.nb_users), ("GRAINES()", ctx.graines)] {
            rendu = rendu.replace(&format!("{{{}}}", nom), &val.to_string());
        }
        res.sortie.push(rendu);
        return Ok(i);
    }

    // ── SOIT nom = expression — créer une variable ──
    if ligne.starts_with("SOIT ") {
        let reste = &ligne[5..];
        if let Some(pos) = reste.find('=') {
            let nom = reste[..pos].trim().to_uppercase();
            let expr = reste[pos + 1..].trim();
            if nom.is_empty() {
                return Err(format!("ligne {} : SOIT sans nom de variable", i + 1));
            }
            let val = evaluer_expr(expr, vars, ctx)?;
            vars.insert(nom, val);
            return Ok(i);
        }
        return Err(format!("ligne {} : SOIT attend un = — ex: SOIT x = 5", i + 1));
    }

    // ── SI condition ALORS ... SINON ... FIN ──
    if ligne.starts_with("SI ") {
        if let Some(pos) = ligne.find(" ALORS") {
            let cond = &ligne[3..pos];
            let vrai = evaluer_cond(cond, vars, ctx)?;
            // trouver la fin : SINON ou FIN au même niveau
            let mut j = i + 1;
            let mut fin_sinon: Option<usize> = None;
            let mut fin_bloc: usize = i + 1;
            let mut profondeur = 0usize;
            while j < lignes.len() {
                let l = lignes[j].trim().to_uppercase();
                if l.starts_with("SI ") { profondeur += 1; }
                if l.starts_with("FIN") {
                    if profondeur == 0 { fin_bloc = j; break; }
                    profondeur -= 1;
                }
                if l.starts_with("SINON") && profondeur == 0 { fin_sinon = Some(j); }
                j += 1;
            }
            if j >= lignes.len() { return Err(format!("SI ligne {} : FIN manquant", i + 1)); }
            if vrai {
                let fin_vrai = fin_sinon.unwrap_or(fin_bloc);
                executer_bloc(lignes, i + 1, profondeur_max + 1, vars, ctx, res)?;
                let _ = fin_vrai;
            } else if let Some(_pos_sinon) = fin_sinon {
                executer_bloc(lignes, _pos_sinon + 1, profondeur_max + 1, vars, ctx, res)?;
            }
            return Ok(fin_bloc);
        }
        return Err(format!("ligne {} : SI attend ALORS — ex: SI x > 3 ALORS", i + 1));
    }

    // ── REPETE n FOIS ... FIN ──
    if ligne.starts_with("REPETE ") {
        if let Some(pos) = ligne.find(" FOIS") {
            let n = evaluer_expr(&ligne[7..pos], vars, ctx)?;
            if n < 0 || n > 1000 {
                return Err(format!("ligne {} : REPETE entre 0 et 1000 fois", i + 1));
            }
            // trouver le FIN correspondant
            let mut j = i + 1;
            let mut profondeur = 0usize;
            while j < lignes.len() {
                let l = lignes[j].trim().to_uppercase();
                if l.starts_with("REPETE ") || l.starts_with("SI ") { profondeur += 1; }
                if l.starts_with("FIN") {
                    if profondeur == 0 { break; }
                    profondeur -= 1;
                }
                j += 1;
            }
            if j >= lignes.len() { return Err(format!("REPETE ligne {} : FIN manquant", i + 1)); }
            for _tour in 0..n {
                executer_bloc(lignes, i + 1, profondeur_max + 1, vars, ctx, res)?;
            }
            return Ok(j);
        }
        return Err(format!("ligne {} : REPETE attend FOIS — ex: REPETE 3 FOIS", i + 1));
    }

    // ── MINE() — ordonner au soleil de forger un bloc (◓) ──
    if ligne == "MINE()" || brut == "◓ EXE" {
        res.mine = true;
        res.sortie.push(format!("◓ le soleil forge le bloc {} pour {}", ctx.nb_blocs + 1, ctx.username));
        return Ok(i);
    }

    // ── ENVOIE("dest", montant) — envoyer des AFR (⬔) ──
    if ligne.starts_with("ENVOIE(") {
        // ENVOIE("aisha", 5)
        let inner = &brut[7..brut.len().saturating_sub(1)];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() != 2 {
            return Err(format!("ligne {} : ENVOIE(\"nom\", montant)", i + 1));
        }
        let dest = parts[0].trim().trim_matches('"').to_string();
        if dest.is_empty() {
            return Err(format!("ligne {} : destinataire vide", i + 1));
        }
        let montant = evaluer_expr(parts[1].trim(), vars, ctx)?;
        if montant <= 0 {
            return Err("le fleuve ne coule pas à l'envers — montant invalide".to_string());
        }
        if montant > ctx.solde {
            return Err(format!("solde insuffisant — tu as {} AFR, le programme veut donner {}", ctx.solde, montant));
        }
        res.tx_envoyees.push((dest.clone(), montant));
        res.sortie.push(format!("⬔ {} AFR partent vers {} par ⊕⟠", montant, dest));
        return Ok(i);
    }

    // ── AMION — honorer la mère du Chef ──
    if ligne == "AMION" {
        res.sortie.push("◈ AMION BLANDINE — la machine garde le nom de celle qui a donné vie au Chef qui lui a donné vie.".to_string());
        return Ok(i);
    }

    // ── ligne vide ou commentaire ──
    if ligne.is_empty() || ligne.starts_with("//") || ligne.starts_with("◈◈") {
        return Ok(i);
    }

    Err(format!("ligne {} : je ne connais pas « {} » — tape AIDE dans l'éditeur", i + 1, brut))
}

/// Exécute un programme complet
pub fn executer_programme(code: &str, ctx: &ContexteLangage) -> ResultatProgramme {
    let mut res = ResultatProgramme { sortie: Vec::new(), erreur: None, tx_envoyees: Vec::new(), mine: false };
    let lignes: Vec<String> = code.lines().map(|l| l.to_string()).collect();
    let mut vars: Vars = HashMap::new();
    let mut i = 0usize;
    while i < lignes.len() {
        match executer_ligne(&lignes, i, 0, &mut vars, ctx, &mut res) {
            Ok(next) => i = next + 1,
            Err(e) => {
                res.erreur = Some(e);
                return res;
            }
        }
    }
    res
}

/// Le programme d'exemple gravé dans le langage — pour montrer au Chef
pub fn programme_exemple() -> &'static str {
    r#"// Le premier programme en LANGAGE AMION
// Notre langage. Notre blockchain. Pas de Python. Pas d'Occident.

DIRE "◈ Le continent compte ses richesses"
SOIT blocs = BLOCS()
SOIT ames = AMES()
DIRE "▩ {blocs} blocs gravés"
DIRE "▦ {ames} âmes inscrites"

SOIT fortune = SOLDE()
DIRE "▥ Mon solde : {fortune} AFR"

SI fortune > 10 ALORS
  DIRE "Je partage ma richesse"
  ENVOIE("aisha", 2)
SINON
  DIRE "Je garde mon fleuve pour grandir"
FIN

REPETE 3 FOIS
  DIRE "▲ l'Afrique monte"
FIN

AMION"#
}
