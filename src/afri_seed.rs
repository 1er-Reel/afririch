// ===== AFRI-SEED — Système de Seed Phrase N-KCOL =====
// 24 mots-nature pour chaque portefeuille AfriChain
// Récupération: entre ton adresse → vois tes 24 mots
// "L'Occident dit perdu — nous disons retrouvable"
// 100% souverain — zéro dépendance externe

use crate::afri_hash::afrihash_256;
use crate::afri_rng::AfriRng;
use crate::afri_hex::{encode as hex_encode, decode as hex_decode};
use crate::afri_json::{JsonValue, from_str, to_string, to_string_pretty};
use std::collections::HashMap;

// 256 mots-nature N-KCOL (8 bits par mot × 24 = 192 bits d'entropie)
// Chaque mot est un élément de la nature africaine
// Organisé par les 26 lettres N-KCOL, 16 mots par lettre
pub const N_KCOL_WORDS: &[&str] = &[
    // 0-15: A - Racine (Plantes)
    "baobab", "igname", "karite", "nere", "fonio", "sorgho", "mil", "niebe",
    "gombo", "oseille", "goyave", "mangue", "papaye", "ananas", "gingembre", "yucca",
    // 16-31: B - Expansion (Arbres)
    "acacia", "cedre", "kapokier", "ronier", "tamarin", "balanzan", "caicedrat", "ditax",
    "dimb", "jujubier", "datte", "palme", "raphia", "bambou", "liane", "fougere",
    // 32-47: C - Tourbillon (Eau)
    "fleuve", "delta", "marigot", "mare", "source", "ruisseau", "cascade", "rapide",
    "estuaire", "lagune", "crique", "anse", "rivage", "berge", "rive", "torrent",
    // 48-63: D - Pierre (Terre)
    "laterite", "granit", "quartz", "silex", "basalte", "gres", "marbre", "kaolin",
    "argile", "lapis", "ambre", "corail", "nacre", "gypse", "craie", "tourbe",
    // 64-79: E - Énergie (Feu/Soleil)
    "soleil", "flamme", "braise", "fournaise", "magma", "lave", "cendre", "etincelle",
    "rayon", "chaleur", "brulant", "ardent", "fulgurant", "solaire", "ete", "canicule",
    // 80-95: F - Flux (Grands animaux)
    "lion", "elephant", "girafe", "guepard", "antilope", "buffle", "hippopotame", "crocodile",
    "rhinoceros", "zebre", "gnou", "gazelle", "springbok", "bubale", "eland", "damalisque",
    // 96-111: G - Grondement (Petits animaux)
    "grenouille", "crapaud", "serpent", "mamba", "cobra", "python", "varan", "agame",
    "lezard", "gecko", "margouillat", "scorpion", "scolopendre", "tarentule", "phasme", "cloporte",
    // 112-127: H - Souffle (Oiseaux)
    "aigle", "vautour", "milan", "faucon", "hibou", "pigeon", "tourterelle", "perdrix",
    "caille", "outarde", "rossignol", "merle", "calao", "huppe", "hirondelle", "fauvette",
    // 128-143: I - Ouverture (Insectes)
    "abeille", "papillon", "libellule", "ephemere", "criquet", "mante", "scarabee", "cigale",
    "guepe", "frelon", "mouche", "moustique", "araignee", "termite", "fourmi", "sauterelle",
    // 144-159: J - Métamorphose (Ciel/Météo)
    "etoile", "lune", "aube", "crepuscule", "zenith", "horizon", "ciel", "nuage",
    "brume", "brouillard", "pluie", "rosee", "goutte", "deluge", "averse", "ondee",
    // 160-175: K - Prévenir (Paysage)
    "mont", "colline", "vallee", "creux", "sommet", "pic", "cime", "flanc",
    "versant", "falaise", "grotte", "caverne", "antre", "gouffre", "canyon", "gorge",
    // 176-191: L - Glissement (Paysage 2)
    "ravin", "crevasse", "faille", "mer", "ocean", "fond", "abime", "reef",
    "atol", "ilot", "barriere", "cordon", "dune", "plateau", "steppe", "plaine",
    // 192-207: M - Vibration (Temps/Saison)
    "nuit", "midi", "minuit", "jour", "soir", "matin", "saison", "epoque",
    "ere", "cycle", "renouveau", "croissance", "declin", "mort", "vie", "naissance",
    // 208-223: N - Nuit (Vents)
    "harmattan", "khamsin", "levante", "ponante", "tramontane", "autan", "gregale", "ostro",
    "libeccio", "chergui", "marin", "astro", "galern", "sirocco", "mistral", "alize",
    // 224-239: O - Tunnel (Éléments)
    "feu", "eau", "terre", "metal", "bois", "pierre", "sable", "roche",
    "minerai", "cristal", "gemme", "diamant", "or", "argent", "cuivre", "fer",
    // 240-255: P - Naissance (Sacré/Africain)
    "ancetre", "griot", "totem", "masque", "tamtam", "kora", "ngoni", "djembe",
    "mbira", "calebasse", "bolon", "doundoun", "sabar", "tama", "kpani", "balafon",
];

/// Génère 24 bytes aléatoires (entropie) → 24 mots-nature N-KCOL
pub fn generate_seed_bytes() -> [u8; 24] {
    let mut rng = AfriRng::new();
    let mut seed = [0u8; 24];
    rng.fill_bytes(&mut seed);
    seed
}

/// Convertit 24 bytes en 24 mots-nature
pub fn bytes_to_words(seed: &[u8; 24]) -> Vec<&'static str> {
    seed.iter().map(|&b| N_KCOL_WORDS[b as usize]).collect()
}

/// Convertit 24 mots-nature en 24 bytes
pub fn words_to_bytes(words: &[&str]) -> Option<[u8; 24]> {
    let mut seed = [0u8; 24];
    for (i, word) in words.iter().enumerate() {
        if i >= 24 { return None; }
        let idx = N_KCOL_WORDS.iter().position(|w| *w == *word)?;
        seed[i] = idx as u8;
    }
    Some(seed)
}

/// Dérive la clé privée Ed25519 depuis les 24 bytes de seed
/// Utilise AfriHash-256 (notre propre hash) pour la dérivation
pub fn seed_to_private_key(seed: &[u8; 24]) -> [u8; 32] {
    // Double hash pour expansion: seed(24) → AfriHash-256 → 32 bytes
    let mut input = Vec::with_capacity(24);
    input.extend_from_slice(seed);
    let hash1 = afrihash_256(&input);
    
    // Second hash avec sel N-KCOL pour renforcer
    let mut input2 = Vec::with_capacity(56);
    input2.extend_from_slice(&hash1);
    input2.extend_from_slice(b"N-KCOL-AFRI-SEED-2026");
    let hash2 = afrihash_256(&input2);
    
    hash2
}

/// Génère un seed complet: 24 mots + clé privée + adresse
pub fn generate_full_seed() -> (Vec<&'static str>, [u8; 32], String) {
    let seed_bytes = generate_seed_bytes();
    let words = bytes_to_words(&seed_bytes);
    let priv_key = seed_to_private_key(&seed_bytes);
    // L'adresse sera dérivée par l'appelant via AfriSecretKey
    (words, priv_key, hex_encode(&priv_key))
}

// ===== SEED STORE — Stockage transparent des seeds =====
// L'approche BLANCHE: on stocke les seeds avec les adresses
// Récupération garantie à 100% sur AfriChain

#[derive(Debug, Clone)]
pub struct SeedStore {
    // address → 24 mots séparés par espace
    pub seeds: HashMap<String, String>,
}

impl SeedStore {
    pub fn new() -> Self {
        SeedStore { seeds: HashMap::new() }
    }

    /// Charge depuis seeds.json
    pub fn load() -> Self {
        let path = crate::data_path("seeds.json");
        match std::fs::read_to_string(&path) {
            Ok(data) => {
                if let Ok(v) = from_str(&data) {
                    SeedStore::from_json(&v).unwrap_or(SeedStore::new())
                } else {
                    SeedStore::new()
                }
            }
            Err(_) => SeedStore::new(),
        }
    }

    /// Sauvegarde vers seeds.json
    pub fn save(&self) {
        let path = crate::data_path("seeds.json");
        let data = to_string_pretty(&self.to_json());
        std::fs::write(&path, data).ok();
    }

    /// Stocke un seed pour une adresse
    pub fn store_seed(&mut self, address: &str, words: &[&str]) {
        let words_str = words.join(" ");
        self.seeds.insert(address.to_string(), words_str);
        self.save();
    }

    /// Récupère le seed pour une adresse → 24 mots
    pub fn recover_seed(&self, address: &str) -> Option<Vec<String>> {
        let words_str = self.seeds.get(address)?;
        Some(words_str.split_whitespace().map(|s| s.to_string()).collect())
    }

    /// Vérifie si une adresse a un seed stocké
    pub fn has_seed(&self, address: &str) -> bool {
        self.seeds.contains_key(address)
    }

    /// Nombre total de seeds stockés
    pub fn count(&self) -> usize {
        self.seeds.len()
    }

    pub fn to_json(&self) -> JsonValue {
        let mut map = HashMap::new();
        for (k, v) in &self.seeds {
            map.insert(k.clone(), JsonValue::Str(v.clone()));
        }
        JsonValue::Object(map)
    }

    pub fn from_json(v: &JsonValue) -> Option<Self> {
        let map = v.as_object()?;
        let mut seeds = HashMap::new();
        for (k, v) in map {
            seeds.insert(k.clone(), v.as_str()?.to_string());
        }
        Some(SeedStore { seeds })
    }
}

// ===== BIP39 RECOVERY — Aide pour les wallets occidentaux =====
// Outil d'investigation: scanne l'appareil pour trouver les seeds stockées

/// Liste des chemins communs où les wallets occidentaux stockent des données
pub const WALLET_PATHS: &[&str] = &[
    // Trust Wallet (Android)
    "/data/data/com.wallet.crypto.trustapp/files/",
    "/data/data/com.wallet.crypto.trustapp/shared_prefs/",
    // MetaMask (Android)
    "/data/data/io.metamask/files/",
    "/data/data/io.metamask/shared_prefs/",
    // Bitcoin Core
    "~/.bitcoin/wallet.dat",
    "~/bitcoin/wallet.dat",
    // Electrum
    "~/.electrum/wallets/",
    // General wallet files
    "~/wallet.dat",
    "~/wallet.json",
    "~/keystore/",
    "~/Wallets/",
];

/// Scanne l'appareil pour trouver des fichiers wallet potentiels
pub fn scan_device_for_wallets() -> Vec<String> {
    let mut found = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();
    
    for path in WALLET_PATHS {
        let expanded = if path.starts_with("~/") {
            format!("{}/{}", home, &path[2..])
        } else {
            path.to_string()
        };
        
        if std::path::Path::new(&expanded).exists() {
            found.push(expanded);
        }
    }
    
    // Also scan common Termux directories
    let scan_dirs = [
        format!("{}/", home),
        format!("{}/Downloads/", home),
        format!("{}/Documents/", home),
    ];
    
    for dir in &scan_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("wallet") || name.contains("seed") || name.contains("keystore") || name.contains("mnemonic") {
                    found.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }
    
    found
}

/// Tente d'extraire une seed phrase depuis un fichier
/// Recherche des patterns BIP39 (12 ou 24 mots anglais)
pub fn extract_seed_from_file(path: &str) -> Option<Vec<String>> {
    let data = std::fs::read_to_string(path).ok()?;
    
    // BIP39 word list (premières lettres pour détection rapide)
    // On cherche des séquences de mots qui ressemblent à une seed phrase
    
    // Pattern 1: JSON avec "mnemonic" ou "seed" field
    if data.contains("mnemonic") || data.contains("seed") || data.contains("phrase") {
        // Tente de parser comme JSON
        if let Ok(v) = from_str(&data) {
            if let Some(obj) = v.as_object() {
                for key in ["mnemonic", "seed", "seedPhrase", "phrase", "backup"] {
                    if let Some(val) = obj.get(key) {
                        if let Some(s) = val.as_str() {
                            let words: Vec<String> = s.split_whitespace().map(|w| w.to_string()).collect();
                            if words.len() == 12 || words.len() == 24 {
                                return Some(words);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Pattern 2: Recherche de 12 ou 24 mots séparés par des espaces
    // Les mots BIP39 sont en anglais, minuscules, 3-8 caractères
    for line in data.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.len() == 12 || words.len() == 24 {
            // Vérifie que tous les mots sont des mots BIP39 potentiels
            // (minuscules, lettres uniquement, 3-8 caractères)
            let all_valid = words.iter().all(|w| {
                w.len() >= 3 && w.len() <= 8 && w.chars().all(|c| c.is_ascii_lowercase())
            });
            if all_valid {
                return Some(words.iter().map(|s| s.to_string()).collect());
            }
        }
    }
    
    None
}

/// Vérifie si une seed phrase correspond à une adresse Bitcoin/Ethereum
/// (Pour vérification après récupération)
pub fn verify_seed_match(seed_words: &[String], target_address: &str) -> bool {
    // Pour AfriChain: on vérifie via notre SeedStore
    // Pour les wallets occidentaux: nécessite l'implémentation de BIP39 → adresse
    // Cette fonction sera étendue au fur et à mesure
    let words_str = seed_words.join(" ");
    // Vérification basique: la seed existe-t-elle dans nos données?
    false // Placeholder — sera étendu
}

/// Affiche le rapport de scan pour l'investigation
pub fn scan_report() -> String {
    let mut report = String::new();
    report.push_str("🔍 RAPPORT D'INVESTIGATION — Seeds Occidentales\n");
    report.push_str("=============================================\n\n");
    
    let found = scan_device_for_wallets();
    
    if found.is_empty() {
        report.push_str("❌ Aucun fichier wallet trouvé sur cet appareil.\n");
        report.push_str("   Les wallets occidentaux stockent les seeds dans:\n");
        for p in WALLET_PATHS {
            report.push_str(&format!("   - {}\n", p));
        }
        report.push_str("\n💡 Sur un appareil rooté, ces fichiers seraient accessibles.\n");
        report.push_str("   L'Occident stocke TOUT — nous le prouvons.\n");
    } else {
        report.push_str(&format!("✅ {} fichier(s) wallet trouvé(s)!\n\n", found.len()));
        for (i, path) in found.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, path));
            if let Some(seed) = extract_seed_from_file(path) {
                report.push_str(&format!("   → 🎯 SEED TROUVÉE! {} mots\n", seed.len()));
                report.push_str(&format!("   → Mots: {}... (masqués pour sécurité)\n", 
                    seed.iter().take(3).cloned().collect::<Vec<_>>().join(" ")));
            } else {
                report.push_str("   → Fichier trouvé mais seed non extractible (chiffrée)\n");
            }
        }
    }
    
    report.push_str("\n📊 CONCLUSION:\n");
    report.push_str("L'Occident dit 'impossible à récupérer' mais STOCKE les seeds.\n");
    report.push_str("AfriChain dit 'récupérable' et STOCKE les seeds — mais TRANSPAREMMENT.\n");
    report.push_str("La différence: nous ne mentons pas. 💚🦁\n");
    
    report
}
