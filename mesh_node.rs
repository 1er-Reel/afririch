// AFRI MESH NODE v1.0 — Le premier vrai nœud du réseau africain.
// Un téléphone + ce programme = un nœud du réseau. Pas de serveur, pas d'opérateur.
// Découverte UDP broadcast + messages directs TCP. 100% Rust std, zéro dépendance.
//
// Compilation (Termux) : rustc mesh_node.rs -o mesh_node
// Utilisation         : ./mesh_node <ton_nom> [decalage]
//   Deux téléphones sur le même WiFi, chacun lance le programme → ils se trouvent
//   tout seuls et peuvent s'écrire. AUCUNE donnée ne quitte le réseau local.
//   Decalage (multiple de 100) : pour lancer plusieurs nœuds sur la MÊME machine
//   en test — ./mesh_node bamako 0 et ./mesh_node lagos 100.

use std::io::{BufRead, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const PORT_BASE: u16 = 7171; // UDP broadcast : "je suis là"

struct Noeud {
    nom: String,
    ip: String,
    port: u16, // son port TCP messages
    vu_a: u64,
}

fn maintenant() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mon_nom = if args.len() > 1 {
        args[1].clone()
    } else {
        "afri".to_string()
    };
    let decalage: u16 = if args.len() > 2 {
        args[2].parse().unwrap_or(0)
    } else {
        0
    };
    let port_decouverte = PORT_BASE + decalage;
    let port_messages = PORT_BASE + decalage + 1;

    println!("╔════════════════════════════════════════════════╗");
    println!("║   🌿 AFRI MESH NODE v1.0 — Réseau de l'Afrique  ║");
    println!("╚════════════════════════════════════════════════╝");
    println!("👤 Toi : {}", mon_nom);
    println!("📡 Découverte : UDP port {} (broadcast)", port_decouverte);
    println!("💬 Messages  : TCP port {}", port_messages);
    println!("{}", "─".repeat(50));

    // Les nœuds découverts, partagés entre threads
    let noeuds: Arc<Mutex<Vec<Noeud>>> = Arc::new(Mutex::new(Vec::new()));
    let messages_recus: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // ─── 1. DÉCOUVERTE : UDP broadcast "je suis là" toutes les 3 secondes ───
    let sock_annonce = UdpSocket::bind("0.0.0.0:0").unwrap();
    sock_annonce.set_broadcast(true).unwrap();
    let sock_ecoute = UdpSocket::bind(("0.0.0.0", port_decouverte)).unwrap();
    sock_ecoute.set_read_timeout(Some(Duration::from_millis(500))).unwrap();

    // Thread : envoyer mon annonce — vers TOUS les ports de découverte possibles,
    // pour que des nœuds avec des décalages différents s'entendent quand même.
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
                    if nom != mon_nom2 { // pas moi-même
                        let ip = src.ip().to_string();
                        let mut liste = noeuds_clone.lock().unwrap();
                        match liste.iter_mut().find(|nd| nd.nom == nom) {
                            Some(nd) => {
                                nd.vu_a = maintenant();
                                nd.ip = ip;
                                nd.port = port;
                            }
                            None => {
                                println!("✨ NŒUD DÉCOUVERT : {} ({}:{})", nom, ip, port);
                                liste.push(Noeud { nom, ip, port, vu_a: maintenant() });
                            }
                        }
                    }
                }
            }
        }
    });

    // ─── 2. MESSAGES : serveur TCP — quand un message arrive, on l'affiche ───
    let listener = match TcpListener::bind(("0.0.0.0", port_messages)) {
        Ok(l) => l,
        Err(e) => {
            println!("❌ Port {} occupé ({}). Essaie un autre décalage : ./mesh_node {} 200", port_messages, e, mon_nom);
            return;
        }
    };
    let messages_clone = Arc::clone(&messages_recus);
    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buf = [0u8; 1024];
                if let Ok(n) = std::io::Read::read(&mut stream, &mut buf) {
                    let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                    println!("\r📩 REÇU : {}\n> ", msg);
                    std::io::stdout().flush().unwrap();
                    messages_clone.lock().unwrap().push(msg);
                }
            }
        }
    });

    // ─── 3. TOI : tape un message, il part vers tous les nœuds découverts ───
    println!("📝 Tape ton message + Entrée → il part vers tous les nœuds découverts.");
    println!("   Commandes : /noeuds (liste) | /aide");
    println!("{}", "─".repeat(50));

    let stdin = std::io::stdin();
    let mut ligne = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        ligne.clear();
        match stdin.lock().read_line(&mut ligne) {
            Ok(0) => break, // EOF — plus rien à lire
            Ok(_) => {}
            Err(_) => break,
        }
        let texte = ligne.trim().to_string();
        if texte.is_empty() { continue; }

        if texte == "/noeuds" {
            let liste = noeuds.lock().unwrap();
            // Nettoyer : les nœuds silencieux depuis +30s sont partis
            let maintenant = maintenant();
            if liste.is_empty() {
                println!("  (aucun nœud pour l'instant — attends, ils arrivent toutes les 3s)");
            }
            for nd in liste.iter() {
                let age = maintenant - nd.vu_a;
                let etat = if age < 10 { "🟢 actif" } else if age < 30 { "🟡 loin" } else { "🔴 parti" };
                println!("  {} → {} ({})", nd.nom, nd.ip, etat);
            }
            continue;
        }
        if texte == "/aide" {
            println!("  /noeuds — liste des téléphones découverts");
            println!("  ton texte — envoyé à tous les nœuds");
            continue;
        }

        // Envoyer à tous les nœuds actifs
        let cibles: Vec<(String, String, u16)> = {
            let liste = noeuds.lock().unwrap();
            liste.iter()
                .filter(|nd| maintenant() - nd.vu_a < 30)
                .map(|nd| (nd.nom.clone(), nd.ip.clone(), nd.port))
                .collect()
        };

        if cibles.is_empty() {
            println!("  ⚠️ Aucun nœud actif. L'autre téléphone doit lancer ./mesh_node son_nom sur le MÊME WiFi.");
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
            println!("  ❌ Échec d'envoi — le nœud est peut-être parti. Réessaie.");
        }
    }
}
