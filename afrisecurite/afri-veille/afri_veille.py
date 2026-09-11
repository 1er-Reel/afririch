#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
AfriVeille v1.0 — L'Oreille du Sahel
Ecoute ce que les leaders du monde disent PUBLIQUEMENT sur l'Afrique et l'AES.
Sources 100% publiques (RSS) — aucune interception, aucune illegalite.
Fonctionne sur Termux: python3 afri_veille.py
"""
import urllib.request
import xml.etree.ElementTree as ET
import html
import re
import os
import time

SOURCES = [
    ("Al Jazeera (Monde)", "https://www.aljazeera.com/xml/rss/all.xml"),
    ("BBC Afrique", "https://feeds.bbci.co.uk/news/world/africa/rss.xml"),
    ("France24 Afrique", "https://www.france24.com/fr/afrique/rss"),
    ("RFI Afrique", "https://www.rfi.fr/fr/afrique/rss"),
    ("Jeune Afrique", "https://www.jeuneafrique.com/feed/"),
]

# Mots-cles surveilles
MOTS_AES = ["aes", "alliance des etats du sahel", "sahel"]
MOTS_PAYS = ["mali", "niger", "burkina", "burkina faso", "traore", "ibrahim traore", "goita", "tiani", "bazoum"]
MOTS_OCCIDENT = ["trump", "macron", "washington", "paris", "bruxelles", "otan", "nato", "union europeenne", "etat-unis", "etats-unis", "france", "sanction", "cese", "occident", "europeenne"]
MOTS_SUJET = ["afrique", "africa", "fcfa", "uranium", "petrole", "lithium", "drone", "militaire", "base militaire", "souverainete", " Wagner", "wagner"]

def fetch(url, timeout=15):
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0 (AfriVeille/1.0)"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return r.read().decode("utf-8", errors="replace")
    except Exception as e:
        return None

def parse_rss(text):
    items = []
    try:
        root = ET.fromstring(text)
    except Exception:
        return items
    # RSS 2.0: <channel><item>
    for item in root.iter("item"):
        title = (item.findtext("title") or "").strip()
        link = (item.findtext("link") or "").strip()
        date = (item.findtext("pubDate") or "").strip()
        desc = html.unescape((item.findtext("description") or "").strip())
        desc = re.sub(r"<[^>]+>", "", desc)[:300]
        if title:
            items.append((title, link, date, desc))
    # Atom: <entry>
    for entry in root.iter("{http://www.w3.org/2005/Atom}entry"):
        title = (entry.findtext("{http://www.w3.org/2005/Atom}title") or "").strip()
        link_el = entry.find("{http://www.w3.org/2005/Atom}link")
        link = link_el.get("href", "") if link_el is not None else ""
        date = (entry.findtext("{http://www.w3.org/2005/Atom}updated") or "").strip()
        desc = ""
        if title:
            items.append((title, link, date, desc))
    return items

def normalize(s):
    s = s.lower()
    s = s.replace("é","e").replace("è","e").replace("ê","e").replace("à","a").replace("ç","c").replace("ô","o").replace("î","i").replace("û","u")
    return s

# Bruit a exclure (sport, people) — ce n'est pas de la geopolitique
EXCLUDE = [" vs ", "champions league", "uefa", "ligue 1", "premier league", "la liga", "serie a", "transfer", "signs", "winger", "striker", "midfielder", "goalless", "match report", "fa cup", "copa", "bundesliga", "leagues cup", "nba", "tennis", "boxer", "ufc"]

def classify(text):
    t = normalize(text)
    if any(re.search(r"\b" + re.escape(normalize(m)) + r"\b", t) for m in EXCLUDE):
        return []
    cats = []
    if any(re.search(r"\b" + re.escape(normalize(m)) + r"\b", t) for m in MOTS_AES): cats.append("AES")
    if any(re.search(r"\b" + re.escape(normalize(m)) + r"\b", t) for m in MOTS_PAYS): cats.append("PAYS")
    if any(re.search(r"\b" + re.escape(normalize(m)) + r"\b", t) for m in MOTS_OCCIDENT): cats.append("OCCIDENT")
    if any(re.search(r"\b" + re.escape(normalize(m)) + r"\b", t) for m in MOTS_SUJET): cats.append("SUJET")
    return cats

def main():
    print("=" * 60)
    print("  AfriVeille v1.0 — L'Oreille du Sahel")
    print("  Ecoute PUBLIQUE des declarations sur l'Afrique et l'AES")
    print("  Sources: RSS publics — zero interception, zero illegalite")
    print("=" * 60)

    all_matches = []
    total_scanned = 0
    sources_ok = 0

    for name, url in SOURCES:
        print(f"\n[Ecoute] {name}...")
        text = fetch(url)
        if not text:
            print(f"  x Source muette (hors ligne ou bloquee)")
            continue
        items = parse_rss(text)
        sources_ok += 1
        print(f"  -> {len(items)} declarations publiques entendues")
        total_scanned += len(items)
        for (title, link, date, desc) in items:
            cats = classify(title + " " + desc)
            if cats:
                all_matches.append((name, title, link, date, desc, cats))
        time.sleep(1)  # politesse envers les serveurs

    print(f"\n{'=' * 60}")
    print(f"  RESULTAT: {len(all_matches)} declarations sur l'Afrique/AES")
    print(f"  parmi {total_scanned} declarations publiques de {sources_ok} sources")
    print(f"{'=' * 60}\n")

    # Rapport HTML
    html_out = []
    html_out.append("""<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>AfriVeille — L'Oreille du Sahel</title><style>
body{background:#0d1117;color:#c9d1d9;font-family:monospace;padding:20px;max-width:900px;margin:0 auto;}
h1{color:#f59e0b;}h2{color:#58a6ff;border-bottom:1px solid #30363d;padding-bottom:5px;}
.item{background:#161b22;border:1px solid #30363d;border-radius:8px;padding:12px;margin:10px 0;}
.item:hover{border-color:#f59e0b;}
.badge{display:inline-block;padding:2px 8px;border-radius:10px;font-size:0.75em;margin-right:4px;}
.b-aes{background:#238636;color:white;}.b-pays{background:#1f6feb;color:white;}
.b-occ{background:#da3633;color:white;}.b-sujet{background:#8957e5;color:white;}
a{color:#58a6ff;text-decoration:none;}.date{color:#8b949e;font-size:0.8em;}
.desc{color:#8b949e;font-size:0.9em;margin-top:5px;}
.footer{color:#8b949e;text-align:center;margin-top:30px;font-size:0.8em;}
</style></head><body>
<h1>👂 AfriVeille — L'Oreille du Sahel</h1>
<p>Ce que le monde dit <strong>publiquement</strong> sur l'Afrique et l'AES.</p>
<p class="date">Rapport genere le """ + time.strftime("%d/%m/%Y %H:%M") + """ — sources 100% publiques (RSS)</p>
<hr>""")

    # Sections par categorie prioritaire
    sections = [
        ("AES", "b-aes", "🛡️ L'Alliance des Etats du Sahel (AES)"),
        ("PAYS", "b-pays", "🌍 Mali, Niger, Burkina Faso, nos leaders"),
        ("OCCIDENT", "b-occ", "👁️ Ce que l'Occident declare publiquement"),
    ]
    for cat, badge_cls, title in sections:
        entries = [m for m in all_matches if cat in m[5]]
        html_out.append(f"<h2>{title} ({len(entries)})</h2>")
        if not entries:
            html_out.append("<p class='date'>Aucune declaration publique aujourd'hui.</p>")
        for (src, t, link, date, desc, cats) in entries[:25]:
            badges = " ".join(f"<span class='badge {b}'>{c}</span>" for c, b in
                              [("AES","b-aes"),("PAYS","b-pays"),("OCCIDENT","b-occ"),("SUJET","b-sujet")] if c in cats)
            html_out.append(f"<div class='item'>{badges}<a href='{link}'><strong>{html.escape(t)}</strong></a><br><span class='date'>{src} — {html.escape(date)}</span><p class='desc'>{html.escape(desc)}</p></div>")

    html_out.append(f"""<div class="footer">
AfriVeille v1.0 — L'Oreille du Sahel — Par Koffi Christ Olivier<br>
Sources publiques uniquement: ce que les leaders disent OFFICIELLEMENT.<br>
Pas d'ecoute privee. Pas d'interception. La verite publique, organisee.<br>
Python std only — fonctionne sur Termux
</div></body></html>""")

    out = os.path.expanduser("~/afri_veille_rapport.html")
    with open(out, "w", encoding="utf-8") as f:
        f.write("\n".join(html_out))
    print(f"  Rapport: {out}")
    print(f"  Ouvre-le: termux-open {out}")
    print("\n  L'Afrique ecoute ce que le monde lui dit en public. 👂🌍")

if __name__ == "__main__":
    main()
