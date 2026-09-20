// AFRI MESH NODE v2.0 — LE MESH MULTI-SAUTS 🌐📡
// Œuvre originale de KOFFI CHRIST OLIVIER (Côte d'Ivoire) — Licence AFRI-OSL v1.0.
// Le premier vrai nœud du réseau africain. Un téléphone + ce programme = un nœud.
// Pas de serveur, pas d'opérateur. 100% Rust std, zéro dépendance.
//
// v2.0 — LE ROUTAGE MULTI-SAUTS : A → B → C quand A ne voit pas C directement.
//   Chaque message porte : un identifiant unique, un expéditeur, un destinataire,
//   un TTL (nombre de sauts max), et la liste des nœuds déjà visités.
//   Si le message n'est pas pour moi → je le RELAIE aux voisins que je vois
//   et qui ne l'ont pas encore reçu. Comme ça le message traverse le continent
//   de téléphone en téléphone, sans Orange, sans MTN, sans câble.
//
// Compilation (Termux) : rustc mesh_node.rs -o mesh_node
// Utilisation         : ./mesh_node <ton_nom> [decalage]
//   Deux téléphones sur le même WiFi se trouvent tout seuls.
//   Avec le multi-sauts, TROIS téléphones en ligne (A—B—C) peuvent
//   s'écrire même si A et C sont trop loin l'un de l'autre !
//
// Commandes :
//   /noeuds          — les téléphones que je vois directement
//   /msg <nom> <txt> — message privé routé jusqu'à <nom> (multi-sauts)
//   ton texte        — envoyé à tous les voisins directs
//   /aide            — l'aide

use std::collections::HashSet;
use std::io::{BufRead, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const PORT_BASE: u16 = 7171; // UDP broadcast : "je suis là"
const TTL_DEFAUT: u32 = 8;   // un message peut traverser 8 téléphones

struct Noeud {
    nom: String,
    ip: String,
    port: u16, // son port TCP messages
    vu_a: u64,
}

fn maintenant() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

/// Fabrique un identifiant unique de message : nom + horloge + compteur
fn nouvel_id(mon_nom: &str, compteur: &mut u64) -> String {
    *compteur += 1;
    format!("{}-{}-{}", mon_nom, maintenant(), *compteur)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mon_nom = if args.len() > 1 { args[1].clone() } else { "afri".to_string() };
    let decalage: u16 = if args.len() > 2 { args[2].parse().unwrap_or(0) } else { 0 };
    let port_decouverte = PORT_BASE + decalage;
    let port_messages = PORT_BASE + decalage + 1;

    println!("╔════════════════════════════════════════════════════╗");
    println!("║   🌿 AFRI MESH NODE v2.0 — MULTI-SAUTS 🌐📡        ║");
    println!("║   A → B → C : le message traverse les téléphones  ║");
    println!("╚════════════════════════════════════════════════════╝");
    println!("👤 Toi : {}", mon_nom);
    println!("📡 Découverte : UDP port {} (broadcast)", port_decouverte);
    println!("💬 Messages  : TCP port {}", port_messages);
    println!("🌐 Routage multi-sauts : TTL {}", TTL_DEFAUT);
    println!("{}", "─".repeat(50));

    let noeuds: Arc<Mutex<Vec<Noeud>>> = Arc::new(Mutex::new(Vec::new()));
    // v2.0 — les messages déjà vus (anti-doublon) et le compteur d'IDs
    let vus: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let compteur_id: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

    // ─── 1. DÉCOUVERTE : UDP broadcast "je suis là" toutes les 3 secondes ───
    let sock_annonce = UdpSocket::bind("0.0.0.0:0").unwrap();
    sock_annonce.set_broadcast(true).unwrap();
    let sock_ecoute = UdpSocket::bind(("0.0.0.0", port_decouverte)).unwrap();
    sock_ecoute.set_read_timeout(Some(Duration::from_millis(500))).unwrap();

    let nom_clone = mon_nom.clone();
    let port_messages_clone = port_messages;
    thread::spawn(move || {
        let ports: Vec<u16> = (0..5).map(|i| PORT_BASE + i * 100).collect();
        loop {
            for p in &ports {
                let addr_broadcast = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), *p);
                let annonce = format!("AFRI-MESH|{}|{}", nom_clone, port_messages_clone);
                let _ = sock_annonce.send_to(annonce.as_bytes(), addr_broadcast);
            }
            thread::sleep(Duration::from_secs(3));
        }
    });

    // Thread : écouter les annonces des autres nœuds
    let noeuds_clone = Arc::clone(&noeuds);
    let mon_nom2 = mon_nom.clone();
    thread::spawn(move || {
        let mut buf = [0u8; 256];
        loop {
            if let Ok((n, src)) = sock_ecoute.recv_from(&mut buf) {
                let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                let parties: Vec<&str> = msg.split('|').collect();
                if parties.len() >= 3 && parties[0] == "AFRI-MESH" {
                    let nom = parties[1].to_string();
                    let port: u16 = parties[2].parse().unwrap_or(PORT_BASE + 1);
                    if nom != mon_nom2 {
                        let ip = src.ip().to_string();
                        let mut liste = noeuds_clone.lock().unwrap();
                        match liste.iter_mut().find(|nd| nd.nom == nom) {
                            Some(nd) => {
                                nd.vu_a = maintenant();
                                nd.ip = ip;
                                nd.port = port;
                            }
                            None => {
                                println!("\r✨ NŒUD DÉCOUVERT : {} ({}:{})\n> ", nom, ip, port);
                                let _ = std::io::stdout().flush();
                                liste.push(Noeud { nom, ip, port, vu_a: maintenant() });
                            }
                        }
                    }
                }
            }
        }
    });

    // ─── 2. MESSAGES ROUTÉS : serveur TCP ────────────────────────────────
    // Format du paquet : MESH2|id|de|pour|ttl|chemin|texte
    //   - id : anti-doublon ; de/pour : expéditeur/destinataire
    //   - ttl : sauts restants ; chemin : noms visités séparés par ,
    //   - pour == "*" : diffusion (tout le monde affiche et relai)
    let listener = match TcpListener::bind(("0.0.0.0", port_messages)) {
        Ok(l) => l,
        Err(e) => {
            println!("❌ Port {} occupé ({}). Essaie un autre décalage : ./mesh_node {} 200", port_messages, e, mon_nom);
            return;
        }
    };
    let noeuds_routage = Arc::clone(&noeuds);
    let vus_routage = Arc::clone(&vus);
    let mon_nom_routage = mon_nom.clone();
    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buf = [0u8; 4096];
                if let Ok(n) = std::io::Read::read(&mut stream, &mut buf) {
                    let paquet = String::from_utf8_lossy(&buf[..n]).to_string();
                    let p: Vec<&str> = paquet.splitn(7, '|').collect();
                    if p.len() == 7 && p[0] == "MESH2" {
                        let id = p[1].to_string();
                        let de = p[2].to_string();
                        let pour = p[3].to_string();
                        let ttl: u32 = p[4].parse().unwrap_or(0);
                        let chemin = p[5].to_string();
                        let texte = p[6].to_string();

                        // Anti-doublon : déjà vu → ignorer
                        {
                            let mut v = vus_routage.lock().unwrap();
                            if v.contains(&id) { continue; }
                            v.insert(id.clone());
                            // garder la mémoire légère : max 2000 IDs
                            if v.len() > 2000 {
                                let garder: HashSet<String> = v.iter().skip(v.len() - 1000).cloned().collect();
                                *v = garder;
                            }
                        }

                        // Pour moi ? → afficher
                        if pour == mon_nom_routage || pour == "*" {
                            let sauts = chemin.matches(',').count();
                            println!("\r📩 [{} → {}] {} ({} saut(s))\n> ", de, pour, texte, sauts);
                            let _ = std::io::stdout().flush();
                        }

                        // Pas pour moi seul → RELAYER aux voisins non visités
                        if (pour != mon_nom_routage || pour == "*") && ttl > 0 {
                            let deja_visites: HashSet<&str> = chemin.split(',').filter(|s| !s.is_empty()).collect();
                            let cibles: Vec<(String, String, u16)> = {
                                let liste = noeuds_routage.lock().unwrap();
                                liste.iter()
                                    .filter(|nd| maintenant() - nd.vu_a < 30)
                                    .filter(|nd| nd.nom != de && !deja_visites.contains(nd.nom.as_str()))
                                    .map(|nd| (nd.nom.clone(), nd.ip.clone(), nd.port))
                                    .collect()
                            };
                            let nouveau_chemin = format!("{},{}", chemin, mon_nom_routage);
                            let nouveau_paquet = format!("MESH2|{}|{}|{}|{}|{}|{}", id, de, pour, ttl - 1, nouveau_chemin, texte);
                            for (_, ip, port) in &cibles {
                                if let Ok(mut s) = TcpStream::connect((ip.parse::<Ipv4Addr>().unwrap_or(Ipv4Addr::new(127,0,0,1)), *port)) {
                                    let _ = s.write_all(nouveau_paquet.as_bytes());
                                }
                            }
                        }
                    } else {
                        // compat v1 : texte simple → afficher
                        println!("\r📩 REÇU : {}\n> ", paquet);
                        let _ = std::io::stdout().flush();
                    }
                }
            }
        }
    });

    // ─── 3. TOI : commandes et envois ────────────────────────────────────
    println!("📝 /msg <nom> <texte> — message privé ROUTÉ (multi-sauts)");
    println!("   /noeuds — les téléphones que je vois | /aide");
    println!("{}", "─".repeat(50));

    let stdin = std::io::stdin();
    let mut ligne = String::new();
    loop {
        print!("> ");
        let _ = std::io::stdout().flush();
        ligne.clear();
        match stdin.lock().read_line(&mut ligne) {
            Ok(0) => {
                // Pas de clavier (relais en arrière-plan) → je VEILLE, je ne meurs pas.
                thread::sleep(Duration::from_secs(1));
                continue;
            }
            Ok(_) => {}
            Err(_) => { thread::sleep(Duration::from_secs(1)); continue; }
        }
        let texte = ligne.trim().to_string();
        if texte.is_empty() { continue; }

        if texte == "/noeuds" {
            let liste = noeuds.lock().unwrap();
            let mtn = maintenant();
            if liste.is_empty() {
                println!("  (aucun nœud direct — mais avec le multi-sauts, tes messages peuvent quand même passer par un relais !)");
            }
            for nd in liste.iter() {
                let age = mtn - nd.vu_a;
                let etat = if age < 10 { "🟢 actif" } else if age < 30 { "🟡 loin" } else { "🔴 parti" };
                println!("  {} → {} ({}) — relais possible", nd.nom, nd.ip, etat);
            }
            continue;
        }
        if texte == "/aide" {
            println!("  /msg <nom> <texte> — message privé routé jusqu'à <nom> (traverse les téléphones)");
            println!("  /noeuds — les téléphones que je vois directement");
            println!("  /tous <texte> — diffusion à tout le réseau (multi-sauts)");
            println!("  ton texte — envoyé aux voisins directs (v1)");
            continue;
        }

        // v2.0 — /msg <nom> <texte> : message privé routé
        if let Some(reste) = texte.strip_prefix("/msg ") {
            if let Some((dest, corps)) = reste.split_once(' ') {
                let dest = dest.trim().to_string();
                let corps = corps.to_string();
                if corps.is_empty() { println!("  ⚠️ /msg <nom> <ton message>"); continue; }
                let id = nouvel_id(&mon_nom, &mut compteur_id.lock().unwrap());
                let paquet = format!("MESH2|{}|{}|{}|{}|{}|{}", id, mon_nom, dest, TTL_DEFAUT, mon_nom, corps);
                let cibles: Vec<(String, String, u16)> = {
                    let liste = noeuds.lock().unwrap();
                    liste.iter()
                        .filter(|nd| maintenant() - nd.vu_a < 30)
                        .filter(|nd| nd.nom != dest) // pas directement au destinataire s'il est voisin... si, quand même :
                        .map(|nd| (nd.nom.clone(), nd.ip.clone(), nd.port))
                        .collect()
                };
                // Envoyer à TOUS les voisins actifs (y compris le destinataire s'il est direct)
                let toutes: Vec<(String, String, u16)> = {
                    let liste = noeuds.lock().unwrap();
                    liste.iter()
                        .filter(|nd| maintenant() - nd.vu_a < 30)
                        .map(|nd| (nd.nom.clone(), nd.ip.clone(), nd.port))
                        .collect()
                };
                let cibles = if toutes.is_empty() { cibles } else { toutes };
                let mut envoyes = 0;
                for (_, ip, port) in &cibles {
                    if let Ok(mut s) = TcpStream::connect((ip.parse::<Ipv4Addr>().unwrap_or(Ipv4Addr::new(127,0,0,1)), *port)) {
                        if s.write_all(paquet.as_bytes()).is_ok() { envoyes += 1; }
                    }
                }
                if envoyes > 0 {
                    println!("  ✉️ Message pour {} confié à {} voisin(s) — le mesh le porte (TTL {})", dest, envoyes, TTL_DEFAUT);
                } else {
                    println!("  ⚠️ Aucun voisin direct. Attends qu'un nœud arrive (toutes les 3s) — ou le message attendra que le mesh se forme.");
                }
                continue;
            }
        }

        // v2.0 — /tous <texte> : diffusion multi-sauts
        if let Some(corps) = texte.strip_prefix("/tous ") {
            let id = nouvel_id(&mon_nom, &mut compteur_id.lock().unwrap());
            let paquet = format!("MESH2|{}|{}|*|{}|{}|{}", id, mon_nom, TTL_DEFAUT, mon_nom, corps);
            let cibles: Vec<(String, String, u16)> = {
                let liste = noeuds.lock().unwrap();
                liste.iter()
                    .filter(|nd| maintenant() - nd.vu_a < 30)
                    .map(|nd| (nd.nom.clone(), nd.ip.clone(), nd.port))
                    .collect()
            };
            let mut envoyes = 0;
            for (_, ip, port) in &cibles {
                if let Ok(mut s) = TcpStream::connect((ip.parse::<Ipv4Addr>().unwrap_or(Ipv4Addr::new(127,0,0,1)), *port)) {
                    if s.write_all(paquet.as_bytes()).is_ok() { envoyes += 1; }
                }
            }
            println!("  📢 Diffusion confiée à {} voisin(s) — elle traversera le mesh", envoyes);
            continue;
        }

        // v1 — envoi direct simple (compat)
        let cibles: Vec<(String, String, u16)> = {
            let liste = noeuds.lock().unwrap();
            liste.iter()
                .filter(|nd| maintenant() - nd.vu_a < 30)
                .map(|nd| (nd.nom.clone(), nd.ip.clone(), nd.port))
                .collect()
        };
        if cibles.is_empty() {
            println!("  ⚠️ Aucun nœud direct. L'autre téléphone doit lancer ./mesh_node son_nom sur le MÊME WiFi.");
            continue;
        }
        let mut envoyes = 0;
        for (nom, ip, port) in &cibles {
            if let Ok(mut stream) = TcpStream::connect((ip.parse::<Ipv4Addr>().unwrap(), *port)) {
                let paquet = format!("{} dit : {}", mon_nom, texte);
                if stream.write_all(paquet.as_bytes()).is_ok() {
                    envoyes += 1;
                    println!("  ✅ → {} ({}:{})", nom, ip, port);
                }
            }
        }
        if envoyes == 0 {
            println!("  ❌ Échec d'envoi — réessaie.");
        }
    }
}
