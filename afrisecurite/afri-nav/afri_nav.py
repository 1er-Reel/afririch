#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# ============================================================
# AfriNav v1.0 — Le Navigateur Hors-Ligne du Sahel
# Par Koffi Christ Olivier & Letta-Chan
# Python std only — zero dependance
# Quand la connexion meurt, la connaissance reste.
# ============================================================
import http.server
import socketserver
import os, sys, json, html, urllib.request, urllib.parse, re

PORT = 8095
DOSSIER = os.path.join(os.path.expanduser("~"), "afrinav_pages")

# ---------- Bibliotheheque embarquee (toujours dispo hors-ligne) ----------
BIBLIO = {
  "hamster": ("🐹 Hamster Kombat — Dossier Complet", """
<h2> Saison 2 : Tu es le CEO </h2>
<p><b>Concept:</b> finie la bourse crypto — tu diriges un <b>studio de developpement de jeux video</b>.
Tu embauches des employes, tu produis des hits, tu batis ta plateforme de jeu.</p>
<h2> Le HamsterVerse </h2>
<p>3 jeux connectes, tous payent en HMSTR:</p>
<ul>
<li><b>GameDev Hero</b> — le jeu principal de la saison 2 (tap-to-earn + gestion d'equipe)</li>
<li><b>Hamster Fight Club</b> — combats</li>
<li><b>Hamster King</b> — strategie</li>
</ul>
<h2> Les diamants </h2>
<p>Gagnes pendant la saison interlude (depuis le 20 septembre 2024), ils donnent un <b>boost au debut de la saison 2</b>.
Le staking-like: garder ses HMSTR dans le jeu = boost aussi.</p>
<h2> Verite sur le token HMSTR </h2>
<ul>
<li>Lancement: 26 septembre 2024 sur TON (The Open Network), airdrop geant</li>
<li>300 millions de joueurs au pic — 12 millions d'actifs mensuels aujourd'hui</li>
<li>Le token a <b>chuute de 83%</b> (~0.0016$)</li>
<li>Un 2e airdrop etait promis pour l'ete 2025</li>
<li>Burn de tokens + rachats avec les revenus publicitaires prevus</li>
<li>DAO lance en decembre 2024 + blockchain layer-2 Hamster sur TON en developpement</li>
</ul>
<h2> Conseil du prof </h2>
<p>Le tap-to-earn a fait ses preuves: les premiers joueurs ont gagne, les derniers ont achete un token qui chute.
<b>Si tu joues, joue gratuit.</b> N'investis jamais ton argent dans un token de jeu en chute.
Et souviens-toi: Hamster Kombat tourne sur TON — une blockchain occidentale. AfriChain, elle, t'appartient. 💚</p>
"""),
  "securite": ("🛡️ Bouclier Infini — Les 10 Attaques Reelles", """
<h2> Les attaques que le Sahel doit connaitre </h2>
<ol>
<li><b>Phishing</b> — faux sites qui volent tes mots de passe. Defense: regarde TOUJOURS l'URL exacte.</li>
<li><b>SIM swap</b> — le voleur fait transferer ton numero a sa SIM, recupere tes 2FA. Defense: code PIN operateur + authentificator app (pas SMS).</li>
<li><b>SS7</b> — faille du reseau telecom mondial: interception des SMS n'importe ou. Defense: ne jamais utiliser SMS comme seule securite.</li>
<li><b>Pegasus</b> — logiciel espion qui infecte les telephones des dirigeants. Defense: ne jamais cliquer les liens inconnus, garder le systeme a jour.</li>
<li><b>Cassage de mots de passe</b> — brute force + dictionnaires. Defense: 12+ caracteres, unique par compte, + sel dans le hash.</li>
<li><b>MITM</b> — l'espion entre toi et le serveur sur un wifi public. Defense: HTTPS partout, VPN, verifier les certificats.</li>
<li><b>Data brokers</b> — vendeurs de tes donnees personnelles. Defense: minimum d'infos en ligne, faux numero quand possible.</li>
<li><b>Botnets / DDoS</b> — armees de machines qui submergent un serveur. Defense: filtrage, rate limiting (comme Bouclier X9).</li>
<li><b>Supply chain</b> — une bibliotheque empoisonnee infecte tout le monde (SolarWinds). Defense: Cargo.toml vide = zero dependance = zero risque. ✅</li>
<li><b>IMSI catcher</b> — fausse antenne qui te suit. Defense: mode avion dans les zones sensibles.</li>
</ol>
<h2> Ce qu'AfriChain a deja </h2>
<p>AfriMesh Direct contre SS7 · Ed25519 maison contre MITM · Cargo.toml vide contre supply chain · Bouclier X9 contre botnets · AfriHash+sel contre cassage.</p>
<p><b>Le bouclier est plus lourd que l'epee.</b> 🛡️</p>
"""),
  "nkcol": ("🌿 N-KCOL — La Nature qui Parle", """
<h2> N-ature qui K-OL au C-iel, a l'O-mbre, a la L-une </h2>
<p>Toutes les langues du monde naissent de la meme nature. Chaque lettre a son symbole, son son naturel, son sens.</p>
<h3> Les 3 lois de validation </h3>
<ol>
<li>Un aveugle doit comprendre le son</li>
<li>Un sourd doit sentir la vibration</li>
<li>Un enfant de 3 ans doit pouvoir l'imiter</li>
</ol>
<h3> Decodage animal </h3>
<p>Coq = TÈK-ETCHIIII-TCHAK-OHHHH (4 lois vivantes) · Mouton = M-B-OHHHH · Grenouille = DRIIIIP-DJRRR · Hibou = OHHHH-OUUUH · Abeille = MMMMM</p>
<h3> Le temps naturel </h3>
<p>Pas 24 heures: 4 passages — Naissance → Vie → Mort → Naissance. 1 nuit = 1 vie, 1 jour = 1 vie.
7 indices naturels: baton, coq, lune, arbre, termitiere, etoile, peau. Si 3 indices sont d'accord, la nature a parle.</p>
"""),
  "crypto": ("🪙 L'Ecole du Bitcoin — Lecons du Mineur", """
<h2> Ce que le minage reel nous a appris </h2>
<ul>
<li>Un telephone fait ~1.2 MH/s. Le reseau Bitcoin fait ~600 EH/s = <b>500 000 milliards de fois plus rapide.</b> Miner du BTC sur telephone = impossible pour le profit.</li>
<li>Monero (XMR) reste minable sur CPU: ~213 H/s sur Redmi 15 — pas rentable, mais possible. Preuve que le minage n'est pas magique: c'est de l'electricite transformee en hashrate.</li>
<li>La blockchain n'est pas un fichier: c'est un <b>consensus</b>. Modifier wallet.json chez soi ne change rien — des milliers de noeuds doivent etre d'accord.</li>
<li>Satoshi a mine les premiers blocs seul en 2009. A l'epoque: $1 = 1309 BTC. La valeur vient de la confiance, pas du code.</li>
<li>Les portefeuilles occidentaux mentent: Ledger Recover (le CTO avoue l'acces aux cles), Coldcard bug 5 ans = $116M voles, Trust Wallet extension malveillante = $7M voles.</li>
<li><b>Regle d'or:</b> ta phrase seed de 24 mots ne se partage JAMAIS avec personne. Ni Telegram, ni support, personne.</li>
</ul>
"""),
  "histoire": ("📜 Verites du Sahel — Recherches Historiques", """
<h2> Touaregs (Kel Tamasheq) </h2>
<p>5 confederations du Mali au Burkina. Societe matrilineaire, voile indigo. 4 rebellions (1963, 1990-96, 2006-09, 2012).
Leur revendication d'Azawad etait legitime — elle a ete noyee par AQMI et Ansar Dine.
Secheresses 1973-86 + abandon de l'Etat = terre fertile pour les recruteurs.</p>
<h2> Peuls (Fulani) </h2>
<p>25-65 millions dans 15+ pays. Califat de Sokoto (1804, Ousmane dan Fodio). Les Francais ont detruit leurs theocraties
et coupe les routes de transhumance avec des frontieres fixes. Massacres d'Ogossagou et Moura → cycle de vengeance → recrutement djihadiste.
<b>Le djihadisme est la CONSEQUENCE de la perte des routes/terres/droits, pas la cause.</b></p>
<h2> Sadio Camara (assassine avril 2026) </h2>
<p>Ministre malien de la Defense, Soninke, tue avec sa femme et ses petits-enfants dans un attentat suicide a Kati.
JNIM + FLA ont revendique ensemble — premiere fois que ces ennemis collaborent. Des complices internes arretes.
Il resistait aux violences ethniques ciblant les Peuls. Un general qui protegeait tombé par ceux qui divisent.</p>
<h2> La lecon pour l'Afrique </h2>
<p>Le meme schema se repete: souverainete → division coloniale → destruction economique → marginalisation → autodefense → recuperation par les extremistes → stigmatisation.
AfriChain attaque les causes: droits des terres, routes de commerce, education, justice economique.</p>
"""),
  "outils": ("🔧 Tous Mes Outils — Commandes de Deploiement", """
<h2> L'arsenal complet (a garder hors-ligne!) </h2>
<ul>
<li><b>AfriSat</b> (satellites reels): <code>wget https://paste.rs/W1haB -O afri_sat.py && python3 afri_sat.py</code></li>
<li><b>AfriLab</b> (labo de piratage ethique): <code>wget https://paste.rs/BOMfR -O afri_lab.py && python3 afri_lab.py</code></li>
<li><b>Bouclier Infini</b> (10 attaques reelles): <code>wget https://paste.rs/EWTCA -O bouclier.py && python3 bouclier.py</code></li>
<li><b>Le Lion du Sahel</b> (le jeu): <code>wget https://paste.rs/CZwVj -O lion.html && termux-open lion.html</code></li>
<li><b>AfriNav</b> (ce navigateur): <code>wget https://paste.rs/XXXXX -O afri_nav.py && python3 afri_nav.py</code></li>
<li><b>AfriForme</b> (la plateforme): <code>cd ~/afririch/afriforme && cargo build --release && ./target/release/afriforme</code></li>
<li><b>AfriChain</b> (la blockchain): <code>cd ~/afririch && cargo build --release && ./target/release/africhain</code></li>
</ul>
<p>Sauvegarde cette page — elle contient toutes les commandes meme sans connexion. 💾</p>
"""),
}

# ---------- Pages sauvegardees ----------
def liste_pages():
    if not os.path.isdir(DOSSIER): os.makedirs(DOSSIER, exist_ok=True)
    return sorted(f for f in os.listdir(DOSSIER) if f.endswith(".html"))

def sauvegarder(url, nom):
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0 (AfriNav)"})
    with urllib.request.urlopen(req, timeout=20) as r:
        contenu = r.read().decode("utf-8", "replace")
    nom = re.sub(r"[^a-zA-Z0-9_-]", "_", nom) or "page"
    chemin = os.path.join(DOSSIER, nom + ".html")
    with open(chemin, "w") as f:
        f.write(contenu)
    return nom + ".html", len(contenu)

# ---------- HTML de l'interface ----------
def page_accueil(msg=""):
    cartes = ""
    for cle, (titre, contenu) in BIBLIO.items():
        cartes += f"<div class='carte' onclick=\"location='/lire/{cle}'\"><div class='titre'>{titre}</div><div class='go'>Lire →</div></div>"
    pages = liste_pages()
    lignes = ""
    for p in pages:
        lignes += f"<div class='ligne'><a href='/page/{p}'>📄 {html.escape(p)}</a></div>"
    if not lignes:
        lignes = "<div class='vide'>Aucune page sauvegardee encore. Quand tu as la connexion, va dans SAUVEGARDER et telecharge des pages du net. Elles resteront ici pour toujours.</div>"
    return """<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>AfriNav — Navigateur Hors-Ligne</title>
<style>
*{box-sizing:border-box}
body{margin:0;font-family:Georgia,serif;background:#0d1117;color:#e6dcc8}
header{background:linear-gradient(135deg,#1a4a2e,#2d6a4f);padding:18px 14px;text-align:center}
header h1{margin:0;font-size:22px;color:#ffe8b0}
header p{margin:6px 0 0;font-size:12px;color:#c8e6c9}
nav{display:flex;flex-wrap:wrap;justify-content:center;gap:8px;padding:10px;background:#14281c}
nav a{color:#ffd54a;text-decoration:none;font-size:13px;padding:6px 10px;border:1px solid #2d6a4f;border-radius:14px}
nav a:hover{background:#1a4a2e}
main{max-width:760px;margin:0 auto;padding:14px}
h2{color:#ffe8b0;font-size:17px;border-bottom:1px solid #2d6a4f;padding-bottom:6px}
.carte{background:#161b22;border:1px solid #2d6a4f;border-radius:10px;padding:14px;margin:10px 0;cursor:pointer;transition:all .15s}
.carte:hover{border-color:#ffd54a;transform:translateY(-2px)}
.carte .titre{font-size:16px;color:#ffe8b0;font-weight:bold}
.carte .go{color:#7fbf8f;font-size:12px;margin-top:6px}
.ligne{padding:10px;border-bottom:1px solid #21262d}
.ligne a{color:#8fd3a8;text-decoration:none;font-size:14px}
.vide{color:#8b949e;font-size:13px;padding:14px;font-style:italic}
form{background:#161b22;border:1px solid #2d6a4f;border-radius:10px;padding:14px;margin:10px 0}
input,button{width:100%;padding:12px;margin:6px 0;border-radius:8px;border:1px solid #2d6a4f;font-size:14px;background:#0d1117;color:#e6dcc8}
button{background:#1a4a2e;color:#ffe8b0;font-weight:bold;cursor:pointer}
button:hover{background:#2d6a4f}
.msg{background:#1a3a1a;border:1px solid #4a8;border-radius:8px;padding:10px;margin:10px 0;color:#b8f0c8;font-size:13px}
footer{text-align:center;padding:16px;color:#5a7a5a;font-size:11px}
</style></head><body>
<header><h1>🦁 AfriNav — Le Navigateur Hors-Ligne du Sahel</h1>
<p>Quand la connexion meurt, la connaissance reste. Par Koffi Christ Olivier &amp; Letta-Chan 💚</p></header>
<nav>
<a href="/">🏠 Bibliotheque</a>
<a href="/sauver">💾 Sauvegarder</a>
<a href="/mes-pages">📄 Mes Pages</a>
</nav>
<main>
""" + (f"<div class='msg'>{msg}</div>" if msg else "") + """
<h2>📚 Bibliotheque embarquee — toujours dispo, zero connexion requise</h2>
""" + cartes + """
</main>
<footer>AfriNav v1.0 — Python std only — Rust dans le coeur, Python dans les veines — L'Afrique ne demande plus la permission 💚</footer>
</body></html>"""

def page_lire(cle):
    if cle not in BIBLIO:
        return page_accueil("Page introuvable.")
    titre, contenu = BIBLIO[cle]
    return """<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>""" + html.escape(titre) + """</title>
<style>
body{margin:0;font-family:Georgia,serif;background:#0d1117;color:#e6dcc8;padding:16px;line-height:1.7}
h1{color:#ffe8b0;font-size:20px}h2{color:#ffd54a;font-size:17px;margin-top:26px}h3{color:#8fd3a8;font-size:15px}
p,li{font-size:14px}code{background:#1a2a1a;padding:2px 6px;border-radius:4px;color:#ffd54a;font-size:12px}
a{color:#8fd3a8}ul,ol{padding-left:22px}
.retour{display:inline-block;margin-bottom:14px;padding:8px 14px;background:#1a4a2e;color:#ffe8b0;border-radius:16px;text-decoration:none;font-size:13px}
</style></head><body>
<a class='retour' href='/'>← Bibliotheque</a>
<h1>""" + html.escape(titre) + """</h1>
""" + contenu + """
</body></html>"""

def page_sauver(msg=""):
    pages = liste_pages()
    return """<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>AfriNav — Sauvegarder une page</title>
<style>
body{margin:0;font-family:Georgia,serif;background:#0d1117;color:#e6dcc8}
header{background:#1a4a2e;padding:14px;text-align:center}
h1{color:#ffe8b0;font-size:19px;margin:0}
main{max-width:600px;margin:0 auto;padding:14px}
p{font-size:13px;color:#b8c8b8}
form{background:#161b22;border:1px solid #2d6a4f;border-radius:10px;padding:14px;margin:10px 0}
input,button{width:100%;padding:12px;margin:6px 0;border-radius:8px;border:1px solid #2d6a4f;font-size:14px;background:#0d1117;color:#e6dcc8}
button{background:#1a4a2e;color:#ffe8b0;font-weight:bold;cursor:pointer}
.msg{background:#1a3a1a;border:1px solid #4a8;border-radius:8px;padding:10px;margin:10px 0;color:#b8f0c8;font-size:13px}
.ex{background:#161b22;border-radius:8px;padding:10px;margin:8px 0;font-size:12px}
.ex a{color:#8fd3a8;text-decoration:none;display:block;padding:6px 0}
a.retour{color:#8fd3a8;text-decoration:none;font-size:13px}
</style></head><body>
<header><h1>💾 Sauvegarder des pages du net</h1></header>
<main>
<a class='retour' href='/'>← Bibliotheque</a>
""" + (f"<div class='msg'>{msg}</div>" if msg else "") + """
<p>Quand tu as de la connexion, colle une URL ici. La page sera telechargee et rangee dans ta bibliotheque hors-ligne.
Ensuite, tu la liras <b>pour toujours, meme sans connexion</b>.</p>
<form method="POST" action="/sauver">
<input name="url" placeholder="https://fr.wikipedia.org/wiki/Mali" required>
<input name="nom" placeholder="Nom de la page (ex: mali_wiki)">
<button type="submit">📥 Telecharger pour toujours</button>
</form>
<div class='ex'>
<b>Suggestions:</b>
<a href="/sauver?url=https://fr.wikipedia.org/wiki/Mali&nom=mali_wiki">📜 Histoire du Mali (Wikipedia)</a>
<a href="/sauver?url=https://fr.wikipedia.org/wiki/Ibrahim_Traor%C3%A9&nom=traore_wiki">🎖️ Ibrahim Traoré (Wikipedia)</a>
<a href="/sauver?url=https://fr.wikipedia.org/wiki/Alliance_des_%C3%89tats_du_Sahel&nom=aes_wiki">🤝 Alliance des États du Sahel</a>
</div>
<p style='margin-top:20px;'>""" + str(len(pages)) + """ page(s) deja sauvegardee(s).</p>
</main></body></html>"""

def page_mes_pages():
    pages = liste_pages()
    lignes = ""
    for p in pages:
        lignes += f"<div class='ligne'><a href='/page/{p}'>📄 {html.escape(p)}</a> <a class='sup' href='/supprimer/{p}'>🗑️</a></div>"
    if not lignes:
        lignes = "<div class='vide'>Rien encore. Va dans SAUVEGARDER quand tu as la connexion.</div>"
    return """<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>AfriNav — Mes Pages</title>
<style>
body{margin:0;font-family:Georgia,serif;background:#0d1117;color:#e6dcc8}
header{background:#1a4a2e;padding:14px;text-align:center}
h1{color:#ffe8b0;font-size:19px;margin:0}
main{max-width:600px;margin:0 auto;padding:14px}
.ligne{padding:12px;border-bottom:1px solid #21262d;display:flex;justify-content:space-between;align-items:center}
.ligne a{color:#8fd3a8;text-decoration:none;font-size:14px}
.sup{color:#c66}
a.retour{color:#8fd3a8;text-decoration:none;font-size:13px}
.vide{color:#8b949e;font-size:13px;padding:14px;font-style:italic}
</style></head><body>
<header><h1>📄 Mes Pages Hors-Ligne</h1></header>
<main>
<a class='retour' href='/'>← Bibliotheque</a>
""" + lignes + """
</main></body></html>"""

# ---------- Serveur ----------
class Serveur(socketserver.TCPServer):
    allow_reuse_address = True

class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *a): pass

    def _send(self, contenu, ctype="text/html; charset=utf-8"):
        data = contenu.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        p = self.path.split("?")[0]
        if p == "/":
            self._send(page_accueil())
        elif p.startswith("/lire/"):
            self._send(page_lire(p[6:]))
        elif p == "/sauver":
            self._send(page_sauver())
        elif p == "/mes-pages":
            self._send(page_mes_pages())
        elif p.startswith("/page/"):
            nom = p[6:]
            chemin = os.path.join(DOSSIER, nom)
            if os.path.isfile(chemin):
                with open(chemin) as f:
                    self._send(f.read())
            else:
                self._send(page_accueil("Page introuvable."))
        elif p.startswith("/supprimer/"):
            nom = p[11:]
            chemin = os.path.join(DOSSIER, nom)
            if os.path.isfile(chemin):
                os.remove(chemin)
                self._send(page_mes_pages())
            else:
                self._send(page_mes_pages())
        else:
            self._send(page_accueil())

    def do_POST(self):
        if self.path == "/sauver":
            n = int(self.headers.get("Content-Length", 0))
            corps = self.rfile.read(n).decode("utf-8", "replace")
            champs = {}
            for paire in corps.split("&"):
                if "=" in paire:
                    k, v = paire.split("=", 1)
                    champs[k] = urllib.parse.unquote_plus(v)
            url, nom = champs.get("url", ""), champs.get("nom", "page")
            try:
                fichier, taille = sauvegarder(url, nom)
                self._send(page_sauver(f"✅ Sauvegarde reussie: {fichier} ({taille//1024} Ko). Disponible hors-ligne pour toujours!"))
            except Exception as e:
                self._send(page_sauver(f"❌ Echec: {e} — verifie la connexion et l'URL."))
        else:
            self._send(page_accueil())

if __name__ == "__main__":
    os.makedirs(DOSSIER, exist_ok=True)
    print("=" * 56)
    print("  🦁 AFRINAV — Le Navigateur Hors-Ligne du Sahel")
    print("  Quand la connexion meurt, la connaissance reste.")
    print("=" * 56)
    print(f"  Bibliotheque embarquee : {len(BIBLIO)} dossiers")
    print(f"  Pages sauvegardees    : {len(liste_pages())}")
    print(f"  Dossier des pages     : {DOSSIER}")
    print(f"  Adresse               : http://localhost:{PORT}")
    print("=" * 56)
    print("  Ouvre dans Chrome:  termux-open-url http://localhost:" + str(PORT))
    print("  Arreter:  Ctrl+C")
    print()
    try:
        with Serveur(("0.0.0.0", PORT), Handler) as httpd:
            httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n👋 AfriNav se ferme. La bibliotheque reste sur ton telephone.")
