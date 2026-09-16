// AFRIOS v1.0 — Le Système d'Exploitation Africain.
// Le shell qui relie tout : la blockchain, le mesh, le DNS, le peuple.
// Tu ouvres Termux → tu es dans AfriOS. Android ne fait que porter le téléphone.
//
// Compilation (Termux) : rustc afrios.rs -o afrios
// Lancement           : ./afrios
// 100% Rust std. Zéro dépendance. Le Cargo.toml de l'Afrique est vide.

use std::io::{BufRead, Write};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn home() -> String {
    std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
}

fn chemin_donnees(fichier: &str) -> String {
    format!("{}/afririch/{}", home(), fichier)
}

fn maintenant_heure() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("Afri+0 — {:02}:{:02}:{:02}", h, m, s)
}

fn lire_fichier(chemin: &str) -> Option<String> {
    std::fs::read_to_string(chemin).ok()
}

fn compter(cible: &str, motif: &str) -> usize {
    cible.matches(motif).count()
}

fn splash() {
    println!(r#"
   ╔════════════════════════════════════════════════════╗
   ║                                                    ║
   ║        🌿  A F R I O S  v1.0  🌿                  ║
   ║                                                    ║
   ║      Le Système d'Exploitation Africain             ║
   ║      Blockchain · Mesh · DNS · Peuple              ║
   ║                                                    ║
   ║      54 pays — un seul système                      ║
   ║      L'Afrique ne demande plus la permission        ║
   ║                                                    ║
   ╚════════════════════════════════════════════════════╝
"#);
    println!("🕐 {} 💚🦁", maintenant_heure());
    println!("Tape 'aide' pour voir les commandes du continent.\n");
}

fn cmd_aide() {
    println!("┌─ 📋 COMMANDES AFRIOS ─────────────────────────┐");
    println!("│ aide          — cette liste                  │");
    println!("│ etat          — état du continent (blocs, tx) │");
    println!("│ peuple        — les frères inscrits          │");
    println!("│ dns           — les domaines souverains       │");
    println!("│ pays          — les 54 pays de la famille     │");
    println!("│ demarrer      — lance le serveur AfriChain   │");
    println!("│ mesh <nom>    — lance ton nœud mesh           │");
    println!("│ heure         — l'heure d'Afrique             │");
    println!("│ qui           — qui a construit AfriOS        │");
    println!("│ quitter       — fermer AfriOS                 │");
    println!("└───────────────────────────────────────────────┘");
}

fn cmd_etat() {
    match lire_fichier(&chemin_donnees("blockchain.json")) {
        Some(data) => {
            let blocs = compter(&data, "\"index\"");
            let txs = compter(&data, "\"memo\"");
            println!("⛓️  BLOCKCHAIN AFRICHAIN — état du continent :");
            println!("   📦 Blocs        : {}", blocs);
            println!("   📜 Transactions : {}", txs);
            println!("   🔐 Crypto       : Ed25519 + AfriHash — 100% souverain");
            println!("   📦 Dépendances  : ZÉRO — le Cargo.toml est vide");
        }
        None => {
            println!("⚠️  Aucune blockchain trouvée dans ~/afririch/");
            println!("   Lance 'demarrer' pour créer le premier bloc.");
        }
    }
}

fn cmd_peuple() {
    match lire_fichier(&chemin_donnees("users.json")) {
        Some(data) => {
            let n = compter(&data, "\"username\"");
            println!("🌍 LE PEUPLE : {} frère(s) inscrit(s) sur la chaîne.", n);
            // Extraire les noms simplement
            let mut reste = data.as_str();
            let mut noms: Vec<String> = Vec::new();
            while let Some(p) = reste.find("\"username\":") {
                let apres0 = &reste[p + "\"username\":".len()..];
                let nettoyé = apres0.trim_start();
                if let Some(d2) = nettoyé.find('"') {
                    let apres = &nettoyé[d2 + 1..];
                    if let Some(fin) = apres.find('"') {
                        noms.push(apres[..fin].to_string());
                        reste = &apres[fin..];
                        continue;
                    }
                }
                break;
            }
            if !noms.is_empty() {
                println!("   👥 {}", noms.join(" · "));
            }
        }
        None => println!("⚠️  Aucun users.json — lance 'demarrer' d'abord."),
    }
}

fn cmd_dns() {
    match lire_fichier(&chemin_donnees("dns.json")) {
        Some(data) => {
            let mut reste = data.as_str();
            let mut domaines: Vec<String> = Vec::new();
            while let Some(p) = reste.find("\"domaine\":") {
                let apres0 = &reste[p + "\"domaine\":".len()..];
                let nettoyé = apres0.trim_start();
                if let Some(d2) = nettoyé.find('"') {
                    let apres = &nettoyé[d2 + 1..];
                    if let Some(fin) = apres.find('"') {
                        domaines.push(apres[..fin].to_string());
                        reste = &apres[fin..];
                        continue;
                    }
                }
                break;
            }
            if domaines.is_empty() {
                println!("🌐 Aucun domaine enregistré. Va sur /dns (admin).");
            } else {
                println!("🌐 DOMAINES SOUVERAINS (résolus par l'Afrique) :");
                for d in &domaines {
                    println!("   🔗 {}", d);
                }
            }
        }
        None => println!("🌐 AfriDNS pas encore initialisé — lance 'demarrer'."),
    }
}

fn cmd_pays() {
    let pays = [
        ("Algérie", "🇩🇿"), ("Angola", "🇦🇴"), ("Bénin", "🇧🇯"), ("Botswana", "🇧🇼"),
        ("Burkina Faso", "🇧🇫"), ("Burundi", "🇧🇮"), ("Cabo Verde", "🇨🇻"), ("Cameroun", "🇨🇲"),
        ("Centrafrique", "🇨🇫"), ("Tchad", "🇹🇩"), ("Comores", "🇰🇲"), ("Congo", "🇨🇬"),
        ("RD Congo", "🇨🇩"), ("Côte d'Ivoire", "🇨🇮"), ("Djibouti", "🇩🇯"), ("Égypte", "🇪🇬"),
        ("Guinée équatoriale", "🇬🇶"), ("Érythrée", "🇪🇷"), ("Eswatini", "🇸🇿"), ("Éthiopie", "🇪🇹"),
        ("Gabon", "🇬🇦"), ("Gambie", "🇬🇲"), ("Ghana", "🇬🇭"), ("Guinée", "🇬🇳"),
        ("Guinée-Bissau", "🇬🇼"), ("Kenya", "🇰🇪"), ("Lesotho", "🇱🇸"), ("Liberia", "🇱🇷"),
        ("Libye", "🇱🇾"), ("Madagascar", "🇲🇬"), ("Malawi", "🇲🇼"), ("Mali", "🇲🇱"),
        ("Mauritanie", "🇲🇷"), ("Maurice", "🇲🇺"), ("Maroc", "🇲🇦"), ("Mozambique", "🇲🇿"),
        ("Namibie", "🇳🇦"), ("Niger", "🇳🇪"), ("Nigeria", "🇳🇬"), ("Rwanda", "🇷🇼"),
        ("Sao Tomé", "🇸🇹"), ("Sénégal", "🇸🇳"), ("Seychelles", "🇸🇨"), ("Sierra Leone", "🇸🇱"),
        ("Somalie", "🇸🇴"), ("Afrique du Sud", "🇿🇦"), ("Soudan du Sud", "🇸🇸"), ("Soudan", "🇸🇩"),
        ("Tanzanie", "🇹🇿"), ("Togo", "🇹🇬"), ("Tunisie", "🇹🇳"), ("Ouganda", "🇺🇬"),
        ("Zambie", "🇿🇲"), ("Zimbabwe", "🇿🇼"),
    ];
    println!("🌍 LES 54 — un seul continent, une seule famille :");
    let mut ligne = String::from("   ");
    for (i, (n, f)) in pays.iter().enumerate() {
        ligne.push_str(&format!("{} {}  ", f, n));
        if (i + 1) % 4 == 0 {
            println!("{}", ligne);
            ligne = String::from("   ");
        }
    }
    if !ligne.trim().is_empty() {
        println!("{}", ligne);
    }
}

fn cmd_demarrer() {
    println!("🚀 Lancement du serveur AfriChain (--web, port 8080)...");
    let chemin = format!("{}/afririch/target/release/africhain", home());
    if !std::path::Path::new(&chemin).exists() {
        println!("⚠️  Binaire introuvable : {}", chemin);
        println!("   Compile d'abord : cd ~/afririch && cargo build --release");
        return;
    }
    let status = Command::new(&chemin).arg("--web").status();
    match status {
        Ok(_) => println!("Serveur arrêté."),
        Err(e) => println!("❌ Erreur : {}", e),
    }
}

fn cmd_mesh(nom: Option<&str>) {
    let nom_noeud = nom.unwrap_or("afri").to_string();
    println!("📡 Lancement du nœud mesh : {} ...", nom_noeud);
    let chemin = format!("{}/afririch/mesh_node", home());
    if !std::path::Path::new(&chemin).exists() {
        println!("⚠️  mesh_node introuvable : {}", chemin);
        println!("   Compile d'abord : cd ~/afririch && rustc mesh_node.rs -o mesh_node");
        return;
    }
    let status = Command::new(&chemin).arg(&nom_noeud).status();
    match status {
        Ok(_) => println!("Nœud mesh fermé."),
        Err(e) => println!("❌ Erreur : {}", e),
    }
}

fn cmd_qui() {
    println!("💚 AfriOS a été construit par Koffi Christ Olivier,");
    println!("   tapé ligne par ligne dans nano sur Termux, sur un Redmi 15.");
    println!("   Avec Letta-Chan — sa famille machine.");
    println!("🦁 Zéro dépendance occidentale. Le Cargo.toml est vide.");
    println!("   Eux volent le physique. Nous, on gagne par le code.");
}

fn main() {
    splash();
    let stdin = std::io::stdin();
    let mut ligne = String::new();
    loop {
        print!("afrios> ");
        std::io::stdout().flush().unwrap();
        ligne.clear();
        match stdin.lock().read_line(&mut ligne) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        let texte = ligne.trim().to_string();
        if texte.is_empty() {
            continue;
        }
        let parties: Vec<&str> = texte.split_whitespace().collect();
        match parties[0] {
            "aide" | "help" => cmd_aide(),
            "etat" => cmd_etat(),
            "peuple" => cmd_peuple(),
            "dns" => cmd_dns(),
            "pays" => cmd_pays(),
            "demarrer" => cmd_demarrer(),
            "mesh" => cmd_mesh(parties.get(1).copied()),
            "heure" => println!("🕐 {}", maintenant_heure()),
            "qui" => cmd_qui(),
            "quitter" | "exit" => {
                println!("💚 AfriOS se ferme. Le continent reste debout. 🦁");
                break;
            }
            _ => println!("❓ Commande inconnue : '{}'. Tape 'aide'.", parties[0]),
        }
        println!();
    }
}
