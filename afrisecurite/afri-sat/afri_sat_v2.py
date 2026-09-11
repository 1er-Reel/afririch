#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
AfriSat v2.0 — L'Oeil qui regarde les satellites
Sources TLE Celestrak correctes (2026): groupes par categorie
"""
import urllib.request
import math
import time
import os
import html as html_esc

CAPITALES = {
    "Bamako": {"lat": 12.6392, "lon": -8.0029},
    "Niamey": {"lat": 13.5137, "lon": 2.1098},
    "Ouagadougou": {"lat": 12.3686, "lon": -1.5319},
}

# Groupes Celestrak actuels (format GP)
TLE_GROUPS = [
    ("Satellites US militaires", "https://celestrak.org/NORAD/elements/gp.php?GROUP=us-military&FORMAT=tle"),
    ("Satellites USA", "https://celestrak.org/NORAD/elements/gp.php?GROUP=usa&FORMAT=tle"),
    ("Satellites Chine", "https://celestrak.org/NORAD/elements/gp.php?GROUP=china&FORMAT=tle"),
    ("Satellites Russie", "https://celestrak.org/NORAD/elements/gp.php?GROUP=russia&FORMAT=tle"),
    ("Satellites France", "https://celestrak.org/NORAD/elements/gp.php?GROUP=france&FORMAT=tle"),
    ("Starlink", "https://celestrak.org/NORAD/elements/gp.php?GROUP=starlink&FORMAT=tle"),
    ("OneWeb", "https://celestrak.org/NORAD/elements/gp.php?GROUP=oneweb&FORMAT=tle"),
]

R_EARTH = 6371.0

def fetch_tle(url, timeout=25):
    req = urllib.request.Request(url, headers={"User-Agent": "AfriSat/2.0"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return r.read().decode("utf-8", errors="replace")
    except Exception as e:
        print(f"    x Erreur: {e}")
        return None

def parse_tle(text):
    lines = text.strip().split("\n")
    satellites = []
    i = 0
    while i < len(lines) - 2:
        nom = lines[i].strip()
        l1 = lines[i+1].strip()
        l2 = lines[i+2].strip()
        if l1.startswith("1 ") and l2.startswith("2 "):
            satellites.append((nom, l1, l2))
        i += 3
    return satellites

def tle_to_orbite(l1, l2):
    try:
        incl = float(l2[8:16])
        raan = float(l2[17:25])
        ecc = float("0." + l2[26:33].strip())
        argp = float(l2[34:42])
        ma = float(l2[43:51])
        mm = float(l2[52:63])
        periode = 1440.0 / mm if mm > 0 else 90.0
        return {"incl": incl, "raan": raan, "ecc": ecc, "argp": argp, "ma": ma, "mm": mm, "periode": periode}
    except Exception:
        return None

def position_sat(orbite, minutes):
    ma = (orbite["ma"] + 360.0 * minutes / orbite["periode"]) % 360.0
    altitude = 500.0
    theta = math.radians(ma)
    raan = (orbite["raan"] - 360.0 * minutes / 1440.0) % 360.0
    incl = math.radians(orbite["incl"])
    lat = math.degrees(math.asin(math.sin(theta) * math.sin(incl)))
    lon = (raan + math.degrees(math.atan2(math.cos(theta), math.tan(incl)))) % 360.0
    lon = lon - 360.0 if lon > 180.0 else lon
    return lat, lon, altitude

def distance_km(lat1, lon1, lat2, lon2):
    lat1, lon1, lat2, lon2 = map(math.radians, [lat1, lon1, lat2, lon2])
    dlat = lat2 - lat1
    dlon = lon2 - lon1
    a = math.sin(dlat/2)**2 + math.cos(lat1) * math.cos(lat2) * math.sin(dlon/2)**2
    return R_EARTH * 2 * math.asin(math.sqrt(a))

def est_visible(cap, lat_s, lon_s, alt_s):
    horizon = math.sqrt(2 * R_EARTH * alt_s + alt_s**2)
    dist = distance_km(cap["lat"], cap["lon"], lat_s, lon_s)
    return dist < horizon, dist

def main():
    print("=" * 65)
    print("  AfriSat v2.0 — Les satellites qui nous regardent, on les voit")
    print("=" * 65)

    tous = []
    for groupe, url in TLE_GROUPS:
        print(f"\n[{groupe}]")
        tle = fetch_tle(url)
        if tle:
            sats = parse_tle(tle)
            print(f"  -> {len(sats)} satellites")
            tous.extend(sats)
        time.sleep(0.8)

    print(f"\n[Total] {len(tous)} satellites charges")

    maintenant = time.time() / 60.0
    passages = []

    for nom, l1, l2 in tous[:200]:
        orb = tle_to_orbite(l1, l2)
        if not orb:
            continue
        for cap_nom, cap_data in CAPITALES.items():
            visible, dist = est_visible(cap_data, *position_sat(orb, maintenant))
            if visible:
                passages.append((nom, cap_nom, dist, orb["periode"]))

    # HTML
    lignes = []
    lignes.append("""<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>AfriSat — Rapport de surveillance spatiale</title><style>
body{background:#0d1117;color:#c9d1d9;font-family:monospace;padding:20px;max-width:950px;margin:0 auto;}
h1{color:#f59e0b;border-bottom:2px solid #f59e0b;padding-bottom:8px;}
h2{color:#58a6ff;margin-top:28px;}
.card{background:#161b22;border:1px solid #30363d;border-radius:10px;padding:18px;margin:14px 0;}
.sat{background:#1a1d2b;border-left:4px solid #da3633;padding:12px;margin:10px 0;border-radius:6px;}
.visible{background:#0d1f0d;border-left:4px solid #238636;}
.cap{color:#f59e0b;font-weight:bold;font-size:1.1em;}
.dist{color:#8b949e;font-size:0.9em;}
.stat{display:inline-block;background:#21262d;padding:8px 14px;border-radius:6px;margin:6px 4px;}
.stat span{color:#f59e0b;font-weight:bold;}
.foot{color:#8b949e;text-align:center;margin-top:35px;font-size:0.85em;}
</style></head><body>
<h1>🛰️ AfriSat — Rapport de surveillance spatiale</h1>
<p>Les satellites militaires et commerciaux sont publics. Leurs orbites sont enregistrees par le NORAD (reseau americain). On peut voir quand ils passent au-dessus de l'AES.</p>
<hr>""")

    for cap_nom, cap_data in CAPITALES.items():
        lignes.append(f"<h2>🌍 {cap_nom}</h2>")
        caps = [p for p in passages if p[1] == cap_nom]
        if not caps:
            lignes.append("<p class='dist'>Aucun satellite visible a l'instant T (les passages durent 5-10 min).</p>")
        else:
            for nom, _, dist, per in caps[:8]:
                lignes.append(f"<div class='sat visible'><span class='cap'>{html_esc.escape(nom)}</span><br><span class='dist'>Distance: {int(dist)} km — Periode: {int(per)} min</span></div>")

    lignes.append(f"""<div class="card"><h2>👁️ Ce que ca signifie pour l'AES</h2>
<p><strong>Les satellites espions sont publics.</strong> Le NORAD publie leurs orbites pour eviter les collisions. C'est le systeme americain lui-meme qui rend ces donnees disponibles.</p>
<p><strong>On ne peut pas les faire tomber.</strong> Mais on peut savoir QUAND ils passent. Pendant leurs fenetres de passage, on protege ce qui doit l'etre.</p>
<p><strong>Exemples de satellites:</strong></p>
<ul>
<li><strong>Starlink</strong> — 6000+ satellites, couverture mondiale, donnees transitees vers les USA</li>
<li><strong>OneWeb</strong> — constellation concurrente, couverture polaire</li>
<li><strong>Satellites militaires USA</strong> — reconnaissance, communications, GPS</li>
<li><strong>Satellites France</strong> — CSO (reconnaissance optique), Syracuse (communications militaires)</li>
</ul>
<p><em>"Ils ont les yeux dans le ciel. Maintenant on sait quand ils ouvrent les paupieres."</em></p>
</div>
<div class="foot">
AfriSat v2.0 — Par Koffi Christ Olivier<br>
Donnees: Celestrak/NORAD (publiques, gratuites)<br>
Python std only — fonctionne sur Termux/Android<br>
Genere le """ + time.strftime("%d/%m/%Y %H:%M UTC") + """</div></body></html>""")

    path = os.path.expanduser("~/afri_sat_rapport.html")
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lignes))
    print(f"\n  Rapport: {path}")
    print(f"  termux-open {path}")

if __name__ == "__main__":
    main()
