// ===== AFRI-TIME — Notre propre gestion du temps from scratch =====
// 100% souverain — zéro dépendance externe
// Utilise std::time (bibliothèque standard) + calculs manuels
// Pas de Greenwich — le temps de l'Afrique
// UTC est MORT. Afri+0 (Afrique de l'Ouest: Mali, Niger, Burkina Faso = AES)
// Afri+1 (Nigeria, Cameroun), Afri+2 (Égypte, Afrique du Sud), Afri+3, Afri+4

use std::time::{SystemTime, UNIX_EPOCH};

/// Retourne le timestamp Unix actuel en secondes
pub fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Retourne le timestamp Unix actuel en millisecondes
pub fn now_timestamp_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Retourne l'heure Afri+0 actuelle (0-23)
/// Afri+0 = Afrique de l'Ouest (Mali, Niger, Burkina Faso, Sénégal, Ghana)
pub fn now_hour() -> u32 {
    let ts = now_timestamp() as u64;
    ((ts / 3600) % 24) as u32
}

/// Convertit un timestamp Unix en date lisible: "YYYY-MM-DD HH:MM:SS Afri+0"
pub fn format_timestamp(ts: i64) -> String {
    let (year, month, day, hour, min, sec) = timestamp_to_components(ts);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} Afri+0", year, month, day, hour, min, sec)
}

/// Convertit un timestamp Unix en date courte: "YYYY-MM-DD HH:MM Afri+0"
pub fn format_timestamp_short(ts: i64) -> String {
    let (year, month, day, hour, min, _sec) = timestamp_to_components(ts);
    format!("{:04}-{:02}-{:02} {:02}:{:02} Afri+0", year, month, day, hour, min)
}

/// Convertit un timestamp avec fuseau horaire africain
/// offset: 0 (Afri+0: Mali, Niger, BF), 1 (Afri+1: Nigeria), 2 (Afri+2: Egypte), 3, 4
pub fn format_timestamp_afri(ts: i64, offset: i32) -> String {
    let adjusted = ts + (offset as i64 * 3600);
    let (year, month, day, hour, min, sec) = timestamp_to_components(adjusted);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} Afri+{}", year, month, day, hour, min, sec, offset)
}

/// Retourne le nom du fuseau horaire africain
pub fn afri_zone_name(offset: i32) -> &'static str {
    match offset {
        0 => "Afri+0 — Afrique de l'Ouest (Mali, Niger, Burkina Faso, Sénégal, Ghana, Côte d'Ivoire)",
        1 => "Afri+1 — Afrique Centrale (Nigeria, Cameroun, Tchad, Gabon, Congo)",
        2 => "Afri+2 — Afrique de l'Est (Égypte, Afrique du Sud, Rwanda, Libye)",
        3 => "Afri+3 — Corne de l'Afrique (Djibouti, Somalie, Érythrée, Kenya)",
        4 => "Afri+4 — Océan Indien (Maurice, Seychelles, Comores)",
        _ => "Afri+0 — Afrique de l'Ouest",
    }
}

/// Décompose un timestamp Unix en (year, month, day, hour, minute, second)
/// Calcul manuel — pas de chrono, pas de Greenwich
fn timestamp_to_components(ts: i64) -> (i32, u32, u32, u32, u32, u32) {
    let secs = ts;
    let days = secs.div_euclid(86400);
    let time_secs = secs.rem_euclid(86400);

    let hour = (time_secs / 3600) as u32;
    let min = ((time_secs % 3600) / 60) as u32;
    let sec = (time_secs % 60) as u32;

    // Calcul de la date (jour → année/mois/jour)
    // Algorithme: conversion jours → date calendaire
    // Référence: 1970-01-01 = jour 0
    let (year, month, day) = days_to_date(days);

    (year, month, day, hour, min, sec)
}

/// Convertit un nombre de jours depuis 1970-01-01 en (year, month, day)
fn days_to_date(days: i64) -> (i32, u32, u32) {
    // Algorithme de conversion jour → date
    // Basé sur l'algorithme de Howard Hinnant (date library)
    let z = days + 719468; // Décalage vers l'ère proleptique
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097; // Day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // Year of era [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // Day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // Month [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // Day of month [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // Month [1, 12]
    let year = if m <= 2 { y + 1 } else { y };

    (year as i32, m as u32, d as u32)
}
