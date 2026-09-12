// afri_qr.rs — Générateur de QR Code from scratch, zéro dépendance
// Port fidèle de l'algorithme standard (ISO/IEC 18004), versions 1-10,
// correction d'erreur M, masque 0. 100% Rust std. L'Afrique scanne. 💚
//
// Comment ça marche (pour Koffi):
//   1. Texte → bits (mode BYTE: 0100 + longueur 8 bits + données)
//   2. Bits → octets + bourrage 0xEC/0x11
//   3. Reed-Solomon (corps GF(256)) → octets de correction
//   4. Entrelacement data + EC → flux final
//   5. Matrice: motifs de recherche, alignement, temporisation
//   6. Données placées en zigzag + masque 0
//   7. Infos de format (BCH) → le scanner lit le niveau et le masque

pub struct QrMatrix {
    pub size: usize,
    pub modules: Vec<Vec<bool>>, // true = noir
}

// ---- Tables standard ----
// Blocs RS pour correction M, versions 1-10:
// (nb_blocs, total_par_bloc, data_par_bloc) — répétés si 6 valeurs (2 groupes)
const RS_BLOCKS_M: [&[(usize, usize, usize)]; 10] = [
    &[(1, 26, 16)],
    &[(1, 44, 28)],
    &[(1, 70, 44)],
    &[(2, 50, 32)],
    &[(2, 67, 43)],
    &[(4, 43, 27)],
    &[(4, 49, 31)],
    &[(2, 60, 38), (2, 61, 39)],
    &[(3, 58, 36), (2, 59, 37)],
    &[(4, 69, 43), (1, 70, 44)],
];

// Positions d'alignement (versions 1-10)
const PATTERN_POSITIONS: [&[usize]; 10] = [
    &[],
    &[6, 18],
    &[6, 22],
    &[6, 26],
    &[6, 30],
    &[6, 34],
    &[6, 22, 38],
    &[6, 24, 42],
    &[6, 26, 46],
    &[6, 28, 50],
];

// ---- Corps de Galois GF(256), poly 0x11D ----
fn gexp(n: i32) -> u8 {
    static mut TABLE: [u8; 512] = [0; 512];
    static INIT: std::sync::Once = std::sync::Once::new();
    unsafe {
        INIT.call_once(|| {
            let mut x: i32 = 1;
            for i in 0..255 {
                TABLE[i as usize] = x as u8;
                x <<= 1;
                if x & 0x100 != 0 {
                    x ^= 0x11D;
                }
            }
            for i in 255..512 {
                TABLE[i as usize] = TABLE[(i - 255) as usize];
            }
        });
        TABLE[((n % 255 + 255) % 255) as usize]
    }
}

fn glog(n: u8) -> i32 {
    static mut TABLE: [i32; 256] = [0; 256];
    static INIT: std::sync::Once = std::sync::Once::new();
    unsafe {
        INIT.call_once(|| {
            let mut x: i32 = 1;
            for i in 0..255 {
                TABLE[x as usize] = i;
                x <<= 1;
                if x & 0x100 != 0 {
                    x ^= 0x11D;
                }
            }
        });
        TABLE[n as usize]
    }
}

// ---- Polynômes en GF(256) (représentation: coefficient dominant en premier) ----
fn poly_mul(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut num = vec![0u8; a.len() + b.len() - 1];
    for (i, &item) in a.iter().enumerate() {
        for (j, &other) in b.iter().enumerate() {
            num[i + j] ^= gexp(glog(item) + glog(other));
        }
    }
    num
}

fn poly_mod(a: &[u8], b: &[u8]) -> Vec<u8> {
    // a % b, style Python de la référence
    let mut a = a.to_vec();
    loop {
        // retirer les zéros de tête
        while !a.is_empty() && a[0] == 0 {
            a.remove(0);
        }
        let difference = a.len() as i64 - b.len() as i64;
        if difference < 0 {
            return a;
        }
        let ratio = glog(a[0]) - glog(b[0]);
        let mut num = vec![0u8; 0];
        for (i, &other_item) in b.iter().enumerate() {
            let v = if i < a.len() { a[i] } else { 0 };
            num.push(v ^ gexp(glog(other_item) + ratio));
        }
        if difference > 0 {
            num.extend_from_slice(&a[a.len() - difference as usize..]);
        }
        a = num;
    }
}

// ---- BCH pour les infos de format ----
fn bch_digit(data: u32) -> u32 {
    let mut d = data;
    let mut digit = 0;
    while d != 0 {
        digit += 1;
        d >>= 1;
    }
    digit
}

fn bch_type_info(data: u32) -> u32 {
    let g15: u32 = 0b10100110111;
    let g15_mask: u32 = 0b101010000010010;
    let mut d = data << 10;
    while bch_digit(d) >= bch_digit(g15) {
        d ^= g15 << (bch_digit(d) - bch_digit(g15));
    }
    ((data << 10) | d) ^ g15_mask
}

// ---- Construction du flux de données ----
pub fn creer_octets(texte: &str, version: usize) -> Vec<u8> {
    // Blocs RS
    let mut blocs: Vec<(usize, usize)> = Vec::new(); // (data_count, ec_count)
    for &(count, total, data) in RS_BLOCKS_M[version - 1] {
        for _ in 0..count {
            blocs.push((data, total - data));
        }
    }
    let bit_limit: usize = blocs.iter().map(|&(d, _)| d * 8).sum();

    // Buffer de bits
    let bytes = texte.as_bytes();
    let mut buffer: Vec<bool> = Vec::new();
    // mode BYTE = 0100
    buffer.extend([false, true, false, false]);
    // longueur: 8 bits (versions 1-9)
    for i in (0..8).rev() {
        buffer.push((bytes.len() >> i) & 1 == 1);
    }
    // données
    for &b in bytes {
        for i in (0..8).rev() {
            buffer.push((b >> i) & 1 == 1);
        }
    }
    // terminateur
    for _ in 0..4.min(bit_limit - buffer.len()) {
        buffer.push(false);
    }
    // alignement octet
    while buffer.len() % 8 != 0 {
        buffer.push(false);
    }
    // bourrage 0xEC / 0x11
    let mut i = 0;
    while buffer.len() < bit_limit {
        let octet: u8 = if i % 2 == 0 { 0xEC } else { 0x11 };
        for k in (0..8).rev() {
            buffer.push((octet >> k) & 1 == 1);
        }
        i += 1;
    }
    // bits → octets
    let mut data_octets: Vec<u8> = Vec::new();
    for chunk in buffer.chunks(8) {
        let mut v = 0u8;
        for &b in chunk {
            v = (v << 1) | (b as u8);
        }
        data_octets.push(v);
    }

    // Reed-Solomon par bloc
    let mut dcdata: Vec<Vec<u8>> = Vec::new();
    let mut ecdata: Vec<Vec<u8>> = Vec::new();
    let mut offset = 0;
    for &(dc_count, ec_count) in &blocs {
        let current_dc = data_octets[offset..offset + dc_count].to_vec();
        offset += dc_count;

        // polynôme générateur de degré ec_count
        let mut rs_poly: Vec<u8> = vec![1];
        for i in 0..ec_count {
            rs_poly = poly_mul(&rs_poly, &[1, gexp(i as i32)]);
        }
        // rawPoly = data avec décalage (degré = len(rs_poly)-1)
        let mut raw = current_dc.clone();
        raw.resize(current_dc.len() + rs_poly.len() - 1, 0);
        let mod_poly = poly_mod(&raw, &rs_poly);
        // ec = derniers ec_count coefficients
        let mut current_ec: Vec<u8> = Vec::new();
        let mod_offset = mod_poly.len() as i64 - ec_count as i64;
        for i in 0..ec_count {
            let idx = i as i64 + mod_offset;
            current_ec.push(if idx >= 0 { mod_poly[idx as usize] } else { 0 });
        }

        dcdata.push(current_dc);
        ecdata.push(current_ec);
    }

    // Entrelacement
    let max_dc = dcdata.iter().map(|d| d.len()).max().unwrap();
    let max_ec = ecdata.iter().map(|e| e.len()).max().unwrap();
    let mut data: Vec<u8> = Vec::new();
    for i in 0..max_dc {
        for dc in &dcdata {
            if i < dc.len() {
                data.push(dc[i]);
            }
        }
    }
    for i in 0..max_ec {
        for ec in &ecdata {
            if i < ec.len() {
                data.push(ec[i]);
            }
        }
    }
    data
}

// ---- Matrice ----
struct MatriceTravail {
    size: usize,
    modules: Vec<Vec<Option<bool>>>, // None = libre
}

fn placer_motif_recherche(m: &mut MatriceTravail, row: i32, col: i32) {
    // Anneau -1..7 inclus: le tour extérieur = séparateur blanc (False)
    for r in -1..8 {
        if row + r <= -1 || m.size as i32 <= row + r {
            continue;
        }
        for c in -1..8 {
            if col + c <= -1 || m.size as i32 <= col + c {
                continue;
            }
            let noir = (r >= 0 && r <= 6 && (c == 0 || c == 6))
                || (c >= 0 && c <= 6 && (r == 0 || r == 6))
                || (r >= 2 && r <= 4 && c >= 2 && c <= 4);
            m.modules[(row + r) as usize][(col + c) as usize] = Some(noir);
        }
    }
}

fn construire_matrice(version: usize, data: &[u8]) -> QrMatrix {
    let count = 17 + 4 * version;
    let mut m = MatriceTravail {
        size: count,
        modules: vec![vec![None; count]; count],
    };

    // Motifs de recherche (avec séparateurs blancs autour)
    placer_motif_recherche(&mut m, 0, 0);
    placer_motif_recherche(&mut m, (count - 7) as i32, 0);
    placer_motif_recherche(&mut m, 0, (count - 7) as i32);

    // Motifs d'alignement
    let pos = PATTERN_POSITIONS[version - 1];
    for &row in pos {
        for &col in pos {
            if m.modules[row][col].is_some() {
                continue;
            }
            for r in -2i32..3 {
                for c in -2i32..3 {
                    let noir = r == -2 || r == 2 || c == -2 || c == 2 || (r == 0 && c == 0);
                    m.modules[(row as i32 + r) as usize][(col as i32 + c) as usize] = Some(noir);
                }
            }
        }
    }

    // Temporisation
    for r in 8..count - 8 {
        if m.modules[r][6].is_none() {
            m.modules[r][6] = Some(r % 2 == 0);
        }
    }
    for c in 8..count - 8 {
        if m.modules[6][c].is_none() {
            m.modules[6][c] = Some(c % 2 == 0);
        }
    }

    // Infos de format (réservées) — M = 0b00, masque 0 → data = 0
    let bits = bch_type_info(0); // (0 << 3) | 0
    // vertical
    for i in 0..15 {
        let noir = (bits >> i) & 1 == 1;
        if i < 6 {
            m.modules[i][8] = Some(noir);
        } else if i < 8 {
            m.modules[i + 1][8] = Some(noir);
        } else {
            m.modules[count - 15 + i][8] = Some(noir);
        }
    }
    // horizontal
    for i in 0..15 {
        let noir = (bits >> i) & 1 == 1;
        if i < 8 {
            m.modules[8][count - i - 1] = Some(noir);
        } else if i < 9 {
            m.modules[8][15 - i - 1 + 1] = Some(noir);
        } else {
            m.modules[8][15 - i - 1] = Some(noir);
        }
    }
    // module fixe
    m.modules[count - 8][8] = Some(true);

    // Données en zigzag avec masque 0
    let mut inc: i32 = -1;
    let mut row: i32 = count as i32 - 1;
    let mut bit_index: i32 = 7;
    let mut byte_index: usize = 0;
    let data_len = data.len();

    let mut col_base: i32 = count as i32 - 1;
    while col_base > 0 {
        let col = if col_base <= 6 { col_base - 1 } else { col_base };
        loop {
            for c in [col, col - 1] {
                let cu = c as usize;
                if m.modules[row as usize][cu].is_none() {
                    let mut noir = false;
                    if byte_index < data_len {
                        noir = (data[byte_index] >> bit_index) & 1 == 1;
                    }
                    // masque 0: (i + j) % 2 == 0
                    if (row + c) % 2 == 0 {
                        noir = !noir;
                    }
                    m.modules[row as usize][cu] = Some(noir);
                    bit_index -= 1;
                    if bit_index == -1 {
                        byte_index += 1;
                        bit_index = 7;
                    }
                }
            }
            row += inc;
            if row < 0 || row >= count as i32 {
                row -= inc;
                inc = -inc;
                break;
            }
        }
        col_base -= 2;
    }

    QrMatrix {
        size: count,
        modules: m
            .modules
            .into_iter()
            .map(|row| row.into_iter().map(|c| c.unwrap_or(false)).collect())
            .collect(),
    }
}

/// Génère un QR code (versions 1-10, correction M, masque 0) pour un texte.
pub fn generer(texte: &str) -> Result<QrMatrix, String> {
    let bytes = texte.as_bytes();
    // capacité max data v10-M = 213 octets
    let mut version = 0;
    for v in 1..=10 {
        let capacite: usize = RS_BLOCKS_M[v - 1]
            .iter()
            .map(|&(count, _, data)| count * data)
            .sum();
        if bytes.len() + 2 <= capacite {
            version = v;
            break;
        }
    }
    if version == 0 {
        return Err("Texte trop long (max ~211 octets)".into());
    }
    let data = creer_octets(texte, version);
    Ok(construire_matrice(version, &data))
}

/// Rendu SVG du QR code avec bordure blanche (quiet zone obligatoire)
pub fn vers_svg(m: &QrMatrix, taille_px: usize) -> String {
    let n = m.size;
    let bordure = 4; // quiet zone standard
    let total = n + bordure * 2;
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{w}\" viewBox=\"0 0 {t} {t}\" shape-rendering=\"crispEdges\">",
        w = taille_px, t = total
    );
    svg.push_str(&format!("<rect width=\"{t}\" height=\"{t}\" fill=\"#ffffff\"/>", t = total));
    for i in 0..n {
        for j in 0..n {
            if m.modules[i][j] {
                svg.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"1\" height=\"1\" fill=\"#000000\"/>",
                    j + bordure, i + bordure
                ));
            }
        }
    }
    svg.push_str("</svg>");
    svg
}
