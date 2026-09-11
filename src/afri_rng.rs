// ===== AFRI-RNG — Notre propre générateur de nombres aléatoires =====
// 100% souverain — zéro dépendance externe
// Source d'entropie: /dev/urandom (Linux/Termux) + état système
// État interne: seed de 64 bytes, expansion via AfriHash-512

use crate::afri_hash::afrihash_512;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AfriRng {
    state: [u8; 64], // État interne de 512 bits
    counter: u64,
}

impl AfriRng {
    /// Crée un nouveau générateur avec entropie du système
    pub fn new() -> Self {
        let mut seed = [0u8; 64];

        // Source 1: /dev/urandom (available sur Linux/Termux/Android)
        // ATTENTION: urandom est un device INFINI — fs::read() ne se termine jamais!
        // On ouvre le fichier et on lit EXACTEMENT 64 bytes.
        if let Ok(mut f) = fs::File::open("/dev/urandom") {
            use std::io::Read;
            let _ = f.read_exact(&mut seed);
        }

        // Source 2: Temps système (nanosecondes)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let nanos = now.as_nanos();
        for i in 0..8 {
            seed[i] ^= (nanos >> (i * 8)) as u8;
        }

        // Source 3: PID du processus
        let pid = std::process::id();
        for i in 0..4 {
            seed[8 + i] ^= (pid >> (i * 8)) as u8;
        }

        // Source 4: Adresse mémoire (ASLR)
        let stack_addr = &seed as *const _ as u64;
        for i in 0..8 {
            seed[12 + i] ^= (stack_addr >> (i * 8)) as u8;
        }

        // Source 5: Hash de l'état pour mélanger
        let mixed = afrihash_512(&seed);
        seed.copy_from_slice(&mixed);

        AfriRng {
            state: seed,
            counter: 0,
        }
    }

    /// Remplit un buffer avec des bytes aléatoires
    pub fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut offset = 0;
        while offset < dest.len() {
            // Génère 64 bytes via AfriHash-512(state || counter)
            let mut input = Vec::with_capacity(72);
            input.extend_from_slice(&self.state);
            input.extend_from_slice(&self.counter.to_le_bytes());
            let block = afrihash_512(&input);

            // Copie dans dest
            let to_copy = (dest.len() - offset).min(64);
            dest[offset..offset + to_copy].copy_from_slice(&block[..to_copy]);
            offset += to_copy;

            // Met à jour l'état (re-seed partiel)
            self.state = afrihash_512(&self.state);
            self.counter = self.counter.wrapping_add(1);
        }
    }

    /// Génère un u64 aléatoire
    pub fn next_u64(&mut self) -> u64 {
        let mut buf = [0u8; 8];
        self.fill_bytes(&mut buf);
        u64::from_le_bytes(buf)
    }

    /// Génère un usize aléatoire
    pub fn next_usize(&mut self) -> usize {
        self.next_u64() as usize
    }

    /// Génère un nombre aléatoire dans [0, max)
    pub fn next_range(&mut self, max: u64) -> u64 {
        if max == 0 { return 0; }
        self.next_u64() % max
    }

    /// Génère un f64 aléatoire dans [0.0, 1.0)
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }
}

/// Fonction utilitaire — remplace rand::random::<u64>()
pub fn random_u64() -> u64 {
    let mut rng = AfriRng::new();
    rng.next_u64()
}

/// Fonction utilitaire — remplace rand::random::<usize>()
pub fn random_usize() -> usize {
    let mut rng = AfriRng::new();
    rng.next_usize()
}

/// Génère 32 bytes aléatoires (pour clés privées)
pub fn random_32() -> [u8; 32] {
    let mut rng = AfriRng::new();
    let mut buf = [0u8; 32];
    rng.fill_bytes(&mut buf);
    buf
}
