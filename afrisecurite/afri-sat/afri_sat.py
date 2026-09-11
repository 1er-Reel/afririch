#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
AfriSat v1.0 — L'Oeil qui regarde les satellites
Telecharge les donnees orbitales publiques (TLE) et calcule les passages des satellites
occidentaux de reconnaissance au-dessus des capitales de l'AES.
Python pur, Termux: python3 afri_sat.py
Les satellites espions sont publics — leurs orbites sont enregistrées. On peut les voir.
"""
import urllib.request
import math
import time
import os
import html

# Capitales AES
CAPITALES = {
    "Bamako": {"lat": 12.6392, "lon": -8.0029},
    "Niamey": {"lat": 13.5137, "lon": 2.1098},
    "Ouagadougou": {"lat": 12.3686, "lon": -1.5319},
}

# Sources TLE publiques (Celestrak)
TLE_SOURCES = [
    ("Satellites US militaires", "https://celestrak.org/NORAD/elements/gp.php?GROUP=us-military&FORMAT=tle"),
    ("Satellites reconnaissance", "https://celestrak.org/NORAD/elements/gp.php?GROUP=recon&FORMAT=tle"),
    ("Satellites NOAA (US)", "https://celestrak.org/NORAD/elements/gp.php?GROUP=weather&FORMAT=tle"),
]

# Rayon terrestre en km
R_EARTH = 6371.0

def fetch_tle(url, timeout=20):
    """Telecharge les donnees TLE depuis Celestrak"""
    req = urllib.request.Request(url, headers={"User-Agent": "AfriSat/1.0"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return r.read().decode("utf-8", errors="replace")
    except Exception as e:
        return None

def parse_tle(text):
    """Parse les lignes TLE en liste de (nom, ligne1, ligne2)"""
    lines = text.strip().split("\n")
    satellites = []
    i = 0
    while i < len(lines) - 2:
        nom = lines[i].strip()
        ligne1 = lines[i+1].strip()
        ligne2 = lines[i+2].strip()
        if ligne1.startswith("1 ") and ligne2.startswith("2 "):
            satellites.append((nom, ligne1, ligne2))
        i += 3
    return satellites

def tle_to_orbite(l1, l2):
    """Extrait les parametres orbitaux des lignes TLE"""
    try:
        # Inclinaison (degres)
        incl = float(l2[8:16])
        # Ascension droite du noeud ascendant (degres)
        raan = float(l2[17:25])
        # Excentricite
        ecc = float("0." + l2[26:33].strip())
        # Argument du perigee (degres)
        argp = float(l2[34:42])
        # Anomalie moyenne (degres)
        ma = float(l2[43:51])
        # Mouvement moyen (rev/jour)
        mm = float(l2[52:63])
        # Periode orbitale (minutes)
        periode = 1440.0 / mm if mm > 0 else 90.0
        return {"incl": incl, "raan": raan, "ecc": ecc, "argp": argp, "ma": ma, "mm": mm, "periode": periode}
    except Exception:
        return None

def position_satellite(orbite, minutes_depuis_epoch):
    """Calcule la position du satellite (lat, lon, alt) a un temps donne — approximation simple"""
    # Anomalie moyenne actuelle (degres)
    ma = (orbite["ma"] + 360.0 * minutes_depuis_epoch / orbite["periode"]) % 360.0
    # Approximation: orbite circulaire, rayon = R_EARTH + altitude moyenne ~500 km
    altitude = 500.0  # km, approximation pour LEO
    # Angle dans le plan orbital
    theta = math.radians(ma)
    # Longitude du noeud ascendant avance avec la Terre
    raan = (orbite["raan"] - 360.0 * minutes_depuis_epoch / 1440.0) % 360.0
    # Latitude approximative
    incl = math.radians(orbite["incl"])
    lat = math.degrees(math.asin(math.sin(theta) * math.sin(incl)))
    # Longitude approximative
    lon = (raan + math.degrees(math.atan2(math.cos(theta), math.tan(incl)))) % 360.0
    lon = lon - 360.0 if lon > 180.0 else lon
    return lat, lon, altitude

def distance_km(lat1, lon1, lat2, lon2):
    """Distance entre deux points sur Terre (km) — formule haversine"""
    lat1, lon1, lat2, lon2 = map(math.radians, [lat1, lon1, lat2, lon2])
    dlat = lat2 - lat1
    dlon = lon2 - lon1
    a = math.sin(dlat/2)**2 + math.cos(lat1) * math.cos(lat2) * math.sin(dlon/2)**2
    c = 2 * math.asin(math.sqrt(a))
    return R_EARTH * c

def est_visible(capitale, lat_sat, lon_sat, alt_sat):
    """Un satellite est visible depuis la capitale si distance < horizon visible"""
    # Horizon visible depuis altitude h: d = sqrt(2*R*h + h^2)
    horizon = math.sqrt(2 * R_EARTH * alt_sat + alt_sat**2)
    dist = distance_km(capitale["lat"], capitale["lon"], lat_sat, lon_sat)
    return dist < horizon, dist

def main():
    print("=" * 60)
    print("  AfriSat v1.0 — L'Oeil qui regarde les satellites")
    print("  Les satellites espions sont PUBLICS. On peut les voir.")
    print("=" * 60)

    tous_satellites = []
    for nom_groupe, url in TLE_SOURCES:
        print(f"\n[Telechargement] {nom_groupe}...")
        tle = fetch_tle(url)
        if tle:
            sats = parse_tle(tle)
            print(f"  -> {len(sats)} satellites trouves")
            tous_satellites.extend(sats)
        else:
            print("  x Source non disponible")
        time.sleep(1)

    print(f"\n[Total] {len(tous_satellites)} satellites charges")

    # Calculer les passages visibles
    maintenant = time.time() / 60.0  # minutes depuis epoch
    passages = []

    for nom, l1, l2 in tous_satellites[:50]:  # limiter pour rapidite
        orbite = tle_to_orbite(l1, l2)
        if not orbite:
            continue
        for cap_nom, cap_data in CAPITALES.items():
            # Verifier sur plusieurs points de l'orbite
            visible_maintenant, dist = est_visible(cap_data, *position_satellite(orbite, maintenant))
            if visible_maintenant:
                passages.append((nom, cap_nom, dist, orbite["periode"]))

    # Rapport HTML
    html_out = []
    html_out.append("""<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>AfriSat — Les satellites qui nous regardent</title><style>
body{background:#0d1117;color:#c9d1d9;font-family:monospace;padding:20px;max-width:900px;margin:0 auto;}
h1{color:#f59e0b;}h2{color:#58a6ff;border-bottom:1px solid #30363d;padding-bottom:5px;margin-top:25px;}
.card{background:#161b22;border:1px solid #30363d;border-radius:8px;padding:15px;margin:12px 0;}
.sat{background:#1a1d2b;border-left:3px solid #da3633;padding:10px;margin:8px 0;}
.visible{background:#1a2b1a;border-left:3px solid #238636;}
.cap{color:#f59e0b;font-weight:bold;}
.dist{color:#8b949e;font-size:0.9em;}
.footer{color:#8b949e;text-align:center;margin-top:30px;font-size:0.8em;}
</style></head><body>
<h1>🛰️ AfriSat — Les satellites qui nous regardent</h1>
<p>Les satellites de reconnaissance occidentaux sont publics. Leurs orbites sont enregistrees. On peut les voir passer.</p>
<p><strong>Sources:</strong> Celestrak (donnees TLE publiques) — NORAD, le reseau de surveillance spatial americain, publie les positions.</p>
<hr>""")

    for cap_nom, cap_data in CAPITALES.items():
        html_out.append(f"<h2>🌍 {cap_nom} — Capitale de l'AES</h2>")
        cap_passages = [p for p in passages if p[1] == cap_nom]
        if not cap_passages:
            html_out.append("<p class='dist'>Aucun satellite visible actuellement (les passages durent quelques minutes).</p>")
        else:
            for nom, _, dist, periode in cap_passages[:5]:
                html_out.append(f"<div class='sat visible'><span class='cap'>{html.escape(nom)}</span><br>Distance: {int(dist)} km — Periode orbitale: {int(periode)} min<br><em>Passe actuellement au-dessus de la capitale.</em></div>")

    html_out.append(f"""<div class="card"><h2>👁️ Ce que ca signifie</h2>
<p>Ces satellites nous regardent. Mais maintenant, <strong>nous aussi on les regarde</strong>.</p>
<p>Les donnees orbitales sont publiques — le NORAD (reseau americain) les publie pour eviter les collisions. C'est ca, leur systeme: meme les secrets sont dans des bases de donnees.</p>
<p>On ne peut pas les faire tomber. Mais on peut savoir QUAND ils passent. Et pendant leurs passages, on protege ce qui doit l'etre.</p>
<p><em>"Ils ont les yeux dans le ciel. Maintenant on sait quand ils ouvrent les paupieres."</em></p>
</div>
<div class="footer">
AfriSat v1.0 — Par Koffi Christ Olivier<br>
Donnees: Celestrak/NORAD (publiques)<br>
Python std only — fonctionne sur Termux<br>
Genere le """ + time.strftime("%d/%m/%Y %H:%M") + """</div></body></html>""")

    path = os.path.expanduser("~/afri_sat_rapport.html")
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(html_out))
    print(f"\n  Rapport: {path}")
    print(f"  Ouvre-le: termux-open {path}")
    print("\n  Ils nous regardent. Maintenant on les regarde aussi. 🛰️🌍")

if __name__ == "__main__":
    main()
