/// v1.98 — LE NAVIGATEUR AFRI 🌐◉
/// Le client de l'INTERNET AFRI : naviguer dans AfriChain depuis le terminal,
/// en protocole machine ◈⬡◉⬔▤⟠⬠ — sans Chrome, sans HTTP, sans l'Occident.
/// Compilation : rustc navigateur_afri.rs -o navigateur_afri
/// Usage : ./navigateur_afri [hote] [port]   (défaut : 127.0.0.1 8181)
///
/// Commandes :
///   /chemin          → demander une page (◉)
///   /chemin c=a&c=b  → envoyer un formulaire (⬔)
///   brut             → afficher le HTML brut au lieu du texte
///   quitter          → fermer la connexion (⟠⬠)
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

fn en_texte(html: &str) -> String {
    // couper style et script en entier — le texte seul reste
    let sans_style = couper_balises(html, "style");
    let sans_script = couper_balises(&sans_style, "script");
    let mut sortie = String::new();
    let mut dans_tag = false;
    let mut chars = sans_script.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' {
            dans_tag = true;
            if let Some(&n) = chars.peek() {
                if n == '/' || n == '!' || "hpdtl".contains(n) { sortie.push('\n'); }
            }
            continue;
        }
        if c == '>' { dans_tag = false; continue; }
        if !dans_tag { sortie.push(c); }
    }
    let sortie = sortie
        .replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
        .replace("&#x27;", "'").replace("&quot;", "\"").replace("&nbsp;", " ");
    let mut propre = String::new();
    let mut vide = 0;
    for ligne in sortie.lines() {
        let l = ligne.trim();
        if l.is_empty() { vide += 1; if vide > 1 { continue; } }
        else { vide = 0; }
        propre.push_str(l);
        propre.push('\n');
    }
    propre
}

/// Retirer tout le contenu d'une balise (style, script) : <style ...>...</style>
fn couper_balises(html: &str, nom: &str) -> String {
    let mut sortie = String::new();
    let ouvrante = format!("<{}", nom);
    let fermante = format!("</{}", nom);
    let mut reste = html;
    loop {
        match reste.find(&ouvrante) {
            Some(debut) => {
                sortie.push_str(&reste[..debut]);
                // chercher la fermeture </nom
                match reste[debut..].find(&fermante) {
                    Some(fin_rel) => {
                        let fin = debut + fin_rel;
                        // sauter aussi le ">" de la balise fermante
                        let apres = &reste[fin..];
                        match apres.find('>') {
                            Some(g) => reste = &reste[fin + g + 1..],
                            None => { reste = ""; break; }
                        }
                    }
                    None => { reste = ""; break; }
                }
            }
            None => { sortie.push_str(reste); break; }
        }
    }
    sortie
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let hote = args.get(1).cloned().unwrap_or_else(|| "127.0.0.1".to_string());
    let port: u16 = args.get(2).cloned().unwrap_or_else(|| "8181".to_string()).parse().unwrap_or(8181);

    let mut stream = match TcpStream::connect((hote.as_str(), port)) {
        Ok(s) => s,
        Err(e) => { println!("❌ Impossible de rejoindre l'INTERNET AFRI ({}:{}) : {}", hote, port, e); return; }
    };
    let _ = stream.set_nodelay(true);
    let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));

    // handshake ◈⬡ — le serveur répond : intro ×2 + sa carte d'identité
    let _ = writeln!(stream, "◈⬡");
    let _ = stream.flush();
    for _ in 0..3 {
        let mut ligne = String::new();
        match reader.read_line(&mut ligne) {
            Ok(0) | Err(_) => { println!("❌ L'INTERNET AFRI ne répond pas."); return; }
            Ok(_) => println!("{}", ligne.trim()),
        }
    }
    println!();
    println!("🌐 NAVIGATEUR AFRI — tape un chemin (ex: /  /login  /sahara) — « brut » pour le HTML — « quitter » pour partir");

    let stdin = std::io::stdin();
    let mut brut = false;
    loop {
        print!("◉⬔> ");
        let _ = std::io::stdout().flush();
        let mut entree = String::new();
        if stdin.read_line(&mut entree).unwrap_or(0) == 0 { break; }
        let entree = entree.trim().to_string();
        if entree.is_empty() { continue; }
        if entree == "quitter" || entree == "⟠⬠" {
            let _ = writeln!(stream, "⟠⬠");
            let _ = stream.flush();
            let mut l = String::new();
            let _ = reader.read_line(&mut l);
            println!("{}", l.trim());
            break;
        }
        if entree == "brut" { brut = !brut; println!("{} ", if brut { "▤ mode HTML brut" } else { "▤ mode texte lisible" }); continue; }
        if !entree.starts_with('/') {
            println!("◈ commence par / (ex: /account?user=koffi)");
            continue;
        }
        // POST si un corps est fourni après un espace, sinon GET
        let commande = if let Some((chemin, corps)) = entree.split_once(' ') {
            format!("⬔ {} {}", chemin, corps)
        } else {
            format!("◉ {}", entree)
        };
        if writeln!(stream, "{}", commande).is_err() { println!("❌ Connexion perdue."); break; }
        let _ = stream.flush();

        // lire l'en-tête ▤ N octets
        let mut entete = String::new();
        match reader.read_line(&mut entete) {
            Ok(0) | Err(_) => { println!("❌ L'INTERNET AFRI s'est tu."); break; }
            Ok(_) => {}
        }
        let entete = entete.trim();
        let nb: usize = entete
            .strip_prefix("▤ ")
            .and_then(|s| s.split(' ').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        // lire exactement nb octets
        let mut corps = vec![0u8; nb];
        let mut lus = 0;
        while lus < nb {
            match reader.read(&mut corps[lus..]) {
                Ok(0) | Err(_) => break,
                Ok(n) => lus += n,
            }
        }
        let page = String::from_utf8_lossy(&corps[..lus]).to_string();
        if brut {
            println!("{}", page);
        } else {
            println!("{}", en_texte(&page));
        }
        println!("▤ {} octets — {}", lus, entete);
        println!("──────────────────────────────");
    }
}
