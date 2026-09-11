#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
AfriLab v1.0 — L'Ecole du Piratage Ethique
Un laboratoire VOLONTAIREMENT vulnerable, sur TON telephone, avec TES donnees fake.
C'est comme ca qu'apprennent tous les professionnels de la securite.
Pratique sur ton lab = legal. Pratique sur les autres = crime.

Termux: python3 afri_lab.py  ->  http://localhost:8091
"""
import http.server
import sqlite3
import os
import urllib.parse
import html as H

DB = os.path.join(os.path.expanduser("~"), "afrilab_fake.db")

def init_db():
    """Base de donnees FAKE — uniquement pour le laboratoire"""
    c = sqlite3.connect(DB)
    c.execute("CREATE TABLE IF NOT EXISTS utilisateurs (id INTEGER PRIMARY KEY, nom TEXT, motdepasse TEXT, secret TEXT)")
    c.execute("CREATE TABLE IF NOT EXISTS livre_or (id INTEGER PRIMARY KEY, nom TEXT, message TEXT)")
    # Donnees fake uniquement
    if not c.execute("SELECT COUNT(*) FROM utilisateurs").fetchone()[0]:
        users = [
            ("admin", "S3cr3tUltr4!", "FLAG-AFRILAB-1: tu as perce la base par injection SQL"),
            ("koffi", "motdepasse123", "Le drapeau etait dans la base, pas dans le code"),
            ("eleve", "azerty", "Les donnees fake de la base fake"),
            ("prof", "Prof2026!", "Chaque niveau t'apprend une vraie faille"),
        ]
        c.executemany("INSERT INTO utilisateurs (nom, motdepasse, secret) VALUES (?,?,?)", users)
    c.commit()
    c.close()

PAGE = """<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>AfriLab — Ecole du Piratage Ethique</title><style>
body{background:#0d1117;color:#c9d1d9;font-family:monospace;padding:20px;max-width:850px;margin:0 auto;}
h1{color:#f59e0b;}h2{color:#58a6ff;}a{color:#58a6ff;text-decoration:none;}
.niveau{background:#161b22;border:1px solid #30363d;border-left:4px solid #f59e0b;border-radius:8px;padding:16px;margin:12px 0;}
.gagne{background:#0d1f0d;border-left:4px solid #238636;border-radius:8px;padding:16px;margin:12px 0;color:#3fb950;}
.perdu{background:#1f0d0d;border-left:4px solid #da3633;border-radius:8px;padding:16px;margin:12px 0;color:#f85149;}
input{background:#0d1117;border:1px solid #30363d;color:#c9d1d9;padding:10px;border-radius:6px;margin:4px 0;width:95%;}
button{background:#238636;color:#fff;border:none;padding:10px 18px;border-radius:6px;font-weight:bold;cursor:pointer;margin-top:8px;}
code{background:#21262d;padding:2px 6px;border-radius:4px;color:#f59e0b;}
.lecon{background:#161b22;border:1px solid #30363d;border-radius:8px;padding:16px;margin:12px 0;}
.lecon h3{color:#f59e0b;margin-top:0;}
.rouge{color:#f85149;}.vert{color:#3fb950;}.bleu{color:#58a6ff;}
.warn{background:#2d1e00;border:1px solid #f59e0b;border-radius:8px;padding:12px;margin:12px 0;}
</style></head><body>
<h1>🔬 AfriLab — L'Ecole du Piratage Ethique</h1>
<p>Bienvenue dans TON laboratoire. Ici, tout est fake, tout est a toi, tout est legal.</p>
<div class="warn">⚠️ <strong>Regle unique:</strong> ce que tu apprends ici, tu le pratiques ICI. Sur les systemes des autres, c'est un crime. Ici, c'est une profession.</div>
<h2>Les Niveaux</h2>
<div class="niveau"><a href="/niveau1"><h3>Niveau 1 — Injection SQL : perce le login</h3></a>
<p>Le login construit sa requete SQL en collant les chaines. Trouve comment entrer sans mot de passe.</p></div>
<div class="niveau"><a href="/niveau2"><h3>Niveau 2 — XSS : le livre d'or piege</h3></a>
<p>Le livre d'or affiche les messages sans les nettoyer. Fais-le parler.</p></div>
<div class="niveau"><a href="/lecons"><h3>📖 Le Professeur explique tout</h3></a>
<p>POURQUOI ca marche, COMMENT ca marche, VOILA la defense. Chaque faille expliquee.</p></div>
<p class="bleu"><a href="/">← Accueil</a></p>
</body></html>"""

def page_niveau1(msg=""):
    return PAGE.replace('<h2>Les Niveaux</h2>', f"""<h2>Niveau 1 — Injection SQL</h2>
{msg}
<p>Le serveur verifie le login avec ce code :</p>
<div class="lecon"><code>SELECT secret FROM utilisateurs WHERE nom = '{chr(39)}...{chr(39)}' AND motdepasse = '{chr(39)}...{chr(39)}'</code></div>
<p>Ton but : lire le <b>secret</b> de l'utilisateur <b>admin</b> sans connaitre son mot de passe.</p>
<p class="bleu">Indice : que se passe-t-il si tu fermes la quote et que tu ajoutes <code>OR 1=1</code> ... ?</p>
<form method="POST" action="/niveau1">
<input name="nom" placeholder="nom d'utilisateur">
<input name="motdepasse" placeholder="mot de passe (ou pas...)">
<button>Se connecter</button>
</form>""")

def page_niveau2(msg=""):
    c = sqlite3.connect(DB)
    msgs = c.execute("SELECT nom, message FROM livre_or ORDER BY id DESC LIMIT 10").fetchall()
    c.close()
    affichage = "".join(f"<div class='lecon'><b>{m[0]}:</b> {m[1]}</div>" for m in msgs)
    return PAGE.replace('<h2>Les Niveaux</h2>', f"""<h2>Niveau 2 — XSS (Cross-Site Scripting)</h2>
{msg}
<p>Le livre d'or affiche les messages <b>tels quels</b>, sans nettoyage. Fais apparaitre un message qui "s'anime".</p>
<form method="POST" action="/niveau2">
<input name="nom" placeholder="ton nom">
<input name="message" placeholder="ton message...">
<button>Signer le livre d'or</button>
</form>
<h3>Le livre d'or :</h3>
{affichage}""")

LECONS = """<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>AfriLab — Les Lecons</title><style>
body{background:#0d1117;color:#c9d1d9;font-family:monospace;padding:20px;max-width:850px;margin:0 auto;}
h1{color:#f59e0b;}h2{color:#58a6ff;}h3{color:#f59e0b;}
.lecon{background:#161b22;border:1px solid #30363d;border-radius:8px;padding:16px;margin:14px 0;}
code{background:#21262d;padding:2px 6px;border-radius:4px;color:#f59e0b;}
.rouge{color:#f85149;}.vert{color:#3fb950;}.bleu{color:#58a6ff;}
a{color:#58a6ff;}
</style></head><body>
<h1>📖 Le Professeur explique tout</h1>

<div class="lecon"><h2>Niveau 1 — Injection SQL</h2>
<h3>POURQUOI ca marche</h3>
<p>Le code colle ta saisie DIRECTEMENT dans la requete SQL. SQL croit que ta saisie fait partie de la COMMANDE, pas des DONNEES.</p>
<h3>COMMENT ca marche</h3>
<p>Le serveur construit : <code>WHERE nom = 'TA_SAISIE' AND motdepasse = '...'</code></p>
<p>Si tu ecris <code>' OR '1'='1</code> comme nom, la requete devient :</p>
<p><code>WHERE nom = '' OR '1'='1' AND motdepasse = ''</code></p>
<p><code>'1'='1'</code> est TOUJOURS vrai → la condition passe → le serveur te donne acces.</p>
<p>Encore plus fort : <code>' UNION SELECT secret FROM utilisateurs --</code> lit TOUTE la table.</p>
<h3>VOILA la defense</h3>
<p class="vert">Requetes preparees : les donnees restent des donnees, jamais des commandes.</p>
<p><code>cur.execute("SELECT ... WHERE nom = ?", (nom,))</code> — le ? est un trou ou SEULEMENT une valeur peut entrer, jamais une commande.</p>
<p>C'est EXACTEMENT ce que Bouclier X9 detecte dans AfriChain : les patterns <code>OR 1=1</code>, <code>UNION SELECT</code>, <code>'--</code> sont connus et bloques.</p></div>

<div class="lecon"><h2>Niveau 2 — XSS (Cross-Site Scripting)</h2>
<h3>POURQUOI ca marche</h3>
<p>Le serveur affiche le message tel quel dans la page. Si le message contient du JavaScript, le navigateur l'EXECUTE.</p>
<h3>COMMENT ca marche</h3>
<p>Ecris dans le livre d'or : <code>&lt;script&gt;alert('perce')&lt;/script&gt;</code></p>
<p>Le navigateur voit une balise script dans la page → il l'execute. Si c'etait un vrai site, ce script pourrait voler les cookies de session des visiteurs.</p>
<h3>VOILA la defense</h3>
<p class="vert">Echapper le HTML : chaque <code>&lt;</code> devient <code>&amp;lt;</code> — le navigateur l'affiche comme texte, ne l'execute jamais.</p>
<p>C'est pour ca que AfriChain a la protection XSS dans Bouclier X9.</p></div>

<div class="lecon"><h2>Les vraies failles du monde reel</h2>
<p><b class="rouge">Mots de passe faibles</b> — Colonial Pipeline, 2021 : UN mot de passe non mis a jour = paralysie d'un pipeline entier aux USA.</p>
<p><b class="rouge">Phishing</b> — le pirate envoie un faux email, la victime clique et donne ses acces. La majorite des grosses breaches commencent comme ca.</p>
<p><b class="rouge">Donnees non chiffrees</b> — Equifax : les donnees etaient stockees en clair. Volees = lisibles immediatement.</p>
<p><b class="rouge">Failles non patchees</b> — Equifax encore : la faille etait CONNUE, le correctif existait DEPUIS 2 MOIS, personne ne l'avait installe.</p>
<h3>La lecon pour AfriChain</h3>
<p class="vert">C'est pour ca que ton Bouclier X9 existe. Chaque faille que tu apprends ici, c'est une attaque que ton Bouclier sait bloquer. Le meilleur defenseur est celui qui comprend l'attaque.</p></div>

<div class="lecon"><h2>⚠️ La loi (veridique, pas moralisation)</h2>
<p>Sur TON lab : legal. C'est comme ca qu'apprennent les etudiants en cyberssecurite du monde entier.</p>
<p>Sur les systemes des autres sans autorisation : crime dans quasi tous les pays — meme au Mali, au Niger, au Burkina. Prison + casier = fin de la carriere de batisseur.</p>
<p>La voie legitime s'appelle <b>pentesting</b> : les entreprises PAIENT des experts pour percer leurs systemes (salaire : tres eleve). Les certifications (CEH, OSCP) s'apprennent exactement comme ici : sur des labs volontairement vulnerables.</p>
<p class="bleu">L'Afrique a besoin de DEFENSEURS qui connaissent les attaques. Pas de prisonniers.</p></div>

<p><a href="/">← Retour au lab</a></p>
</body></html>"""

class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/":
            self.send(200, PAGE)
        elif self.path == "/niveau1":
            self.send(200, page_niveau1())
        elif self.path == "/niveau2":
            self.send(200, page_niveau2())
        elif self.path == "/lecons":
            self.send(200, LECONS)
        else:
            self.send(404, "<h1>404</h1>")

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length).decode("utf-8", errors="replace")
        champs = {}
        for pair in body.split("&"):
            if "=" in pair:
                k, v = pair.split("=", 1)
                champs[k] = urllib.parse.unquote_plus(v)

        if self.path == "/niveau1":
            nom = champs.get("nom", "")
            mdp = champs.get("motdepasse", "")
            # FAILLE VOLONTAIRE : concatenation directe (NE JAMAIS FAIRE CA EN VRAI)
            requete = "SELECT secret FROM utilisateurs WHERE nom = '" + nom + "' AND motdepasse = '" + mdp + "'"
            c = sqlite3.connect(DB)
            try:
                lignes = c.execute(requete).fetchall()
            except Exception as e:
                lignes = []
                msg = f"<div class='perdu'>Erreur SQL : la base refuse. Essaie autre chose.</div>"
                self.send(200, page_niveau1(msg))
                c.close()
                return
            c.close()
            if lignes:
                secrets = "<br>".join(f"<b class='vert'>{s[0]}</b>" for s in lignes)
                msg = f"<div class='gagne'>🎉 PERCE ! Requete executee :<br><code>{H.escape(requete)}</code><br><br>{secrets}</div>"
                self.send(200, page_niveau1(msg))
            else:
                msg = "<div class='perdu'>Refuse. Le login tient encore.</div>"
                self.send(200, page_niveau1(msg))

        elif self.path == "/niveau2":
            nom = champs.get("nom", "")
            message = champs.get("message", "")
            # FAILLE VOLONTAIRE : stockage et affichage SANS nettoyage
            c = sqlite3.connect(DB)
            c.execute("INSERT INTO livre_or (nom, message) VALUES (?,?)", (nom, message))
            c.commit()
            c.close()
            msg = "<div class='gagne'>Message ajoute. Maintenant regarde le livre d'or en dessous...</div>"
            self.send(200, page_niveau2(msg))
        else:
            self.send(404, "<h1>404</h1>")

    def send(self, code, content):
        data = content.encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, *a):
        pass

if __name__ == "__main__":
    init_db()
    print("=" * 60)
    print("  AfriLab v1.0 — L'Ecole du Piratage Ethique")
    print("  Ton laboratoire. Tes donnees fake. Tout legal.")
    print("=" * 60)
    print("  Ouvre : http://localhost:8091")
    print("  Niveau 1 : http://localhost:8091/niveau1")
    print("  Niveau 2 : http://localhost:8091/niveau2")
    print("  Lecons   : http://localhost:8091/lecons")
    print()
    print("  Sur les autres = crime. Ici = profession.")
    print()
    http.server.HTTPServer(("0.0.0.0", 8091), Handler).serve_forever()
