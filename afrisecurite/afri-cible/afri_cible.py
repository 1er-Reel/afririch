#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# ============================================================
# AFRICIBLE v1.0 — Le Labo Vulnérable du Sahel
# Par Koffi Christ Olivier & Letta-Chan
#
# ⚠️ SITE FICTIF VULNÉRABLE POUR APPRENTISSAGE ⚠️
# Cette "banque" est FAUSSE et contient des failles ON PURPOSE.
# Tu t'entraînes dessus comme sur un tatami. N'utilise JAMAIS
# ces techniques sur de vrais sites — seulement sur des labs
# et des programmes bug bounty autorisés.
#
# 5 défis: IDOR, SQLi, XSS, JWT, Path Traversal
# Python std only — marche offline
# ============================================================
import http.server
import socketserver
import os, json, time, hashlib, hmac, base64, urllib.parse, html, re

PORT = 8097
PROG_DIR = os.path.join(os.path.expanduser("~"), "africible_progress")
SECRET_HS = "sahel_secret_2026"  # le secret HMAC du serveur (tu ne le connais pas normalement~)

# ---------- Les "clients" de la banque fictive ----------
USERS = {
    1: {"id": 1, "user": "koffi", "pass": "afrique2026", "role": "client",
        "nom": "Koffi Christ Olivier", "solde": 125000,
        "note": "Bienvenue sur ton compte de test. Le Directeur a dit que tes idées sur la blockchain sont brillantes."},
    2: {"id": 2, "user": "aminata", "pass": "sahel123", "role": "client",
        "nom": "Aminata Diallo", "solde": 89500,
        "note": "Mes économies pour l'école de Bamako. Ne pas toucher !"},
    3: {"id": 3, "user": "moussa", "pass": "niamey456", "role": "client",
        "nom": "Moussa Ibrahim", "solde": 230000,
        "note": "NOTE SECRÈTE: AFRI{idor_du_sahel} — seul moi dois voir ça."},
    4: {"id": 4, "user": "directeur", "pass": "impossible_a_deviner_999", "role": "directeur",
        "nom": "Le Directeur", "solde": 999999999,
        "note": "Compte de la direction. Le panneau /admin exige le rôle 'admin'."},
}

# ---------- Les fichiers "servables" (avec la faille traversal) ----------
FILES = {
    "lettre.txt": "Niamey, le 3 septembre 2026\n\nCher client, votre compte est en ordre.\n\nLa Banque du Sahel (fictive)",
    "contrat_2026.txt": "CONTRAT DE COMPTE — BANQUE DU SAHEL (fictive)\n\nArticle 1: Ce site est un laboratoire d'apprentissage.\nArticle 2: Toutes les données sont fausses.\nArticle 3: Les failles sont plantées exprès.",
    "../secret/flag_du_desert.txt": "FICHIER SECRET DU SERVEUR\n\nFLAG: AFRI{traversee_du_desert}\n\nBravo, tu as traversé le désert des dossiers~",
    "../../secret/flag_du_desert.txt": "FICHIER SECRET DU SERVEUR\n\nFLAG: AFRI{traversee_du_desert}\n\nBravo, tu as traversé le désert des dossiers~",
}

# ---------- Les 5 défis ----------
FLAGS = {
    "idor": {
        "titre": "🔓 Défi 1 — IDOR : La note secrète de Moussa",
        "mission": "Tu es connecté comme Koffi. Trouve la note secrète de Moussa SANS son mot de passe.",
        "flag": "AFRI{idor_du_sahel}",
        "hints": [
            "Connecte-toi, puis regarde bien l'URL de ton profil.",
            "Ton profil est /profil?id=1. Que se passe-t-il avec d'autres numéros ?",
            "Moussa a l'id 3. Va sur /profil?id=3 et lis sa note secrète.",
        ],
        "solution": "Va sur <code>/profil?id=3</code> — le serveur ne vérifie pas que tu es Moussa. Tu vois sa note secrète avec le flag. C'est un IDOR classique : l'assiette servie sans vérifier le propriétaire.",
    },
    "sqli": {
        "titre": "💉 Défi 2 — SQL Injection : Connexion Directeur",
        "mission": "Connecte-toi comme le Directeur SANS son mot de passe (il est introuvable).",
        "flag": "AFRI{sqli_royale}",
        "hints": [
            "Le formulaire de connexion parle directement à la base de données.",
            "Le guillemet simple ' ferme la chaîne SQL. OR permet d'ajouter une condition toujours vraie.",
            "Dans le champ utilisateur tape: <code>' OR '1'='1</code> — et la même chose dans le mot de passe.",
        ],
        "solution": "Dans les deux champs: <code>' OR '1'='1</code>. La requête devient <code>...WHERE user='' OR '1'='1'</code> — toujours vraie → connexion Directeur acceptée → le flag s'affiche.",
    },
    "xss": {
        "titre": "⚡ Défi 3 — XSS : Vole le cookie du Directeur",
        "mission": "Le Directeur lit les commentaires chaque minute. Fais-lui exécuter ton code pour voler son cookie.",
        "flag": "AFRI{xss_du_directeur}",
        "hints": [
            "Les commentaires sont affichés SANS nettoyage. Teste d'abord: <code>&lt;script&gt;alert(1)&lt;/script&gt;</code>",
            "En JavaScript, <code>document.cookie</code> contient les cookies de la page.",
            "Écris un commentaire contenant un script qui utilise <code>document.cookie</code>. Le Directeur l'exécutera et son cookie partira vers /collect.",
        ],
        "solution": "Commentaire: <code>&lt;script&gt;fetch('/collect?c='+document.cookie)&lt;/script&gt;</code> — ou tout payload avec <code>document.cookie</code> dans un script. Va ensuite sur <code>/collect</code> : le cookie volé du Directeur contient le flag.",
    },
    "jwt": {
        "titre": "🎫 Défi 4 — JWT : Le faux ticket admin",
        "mission": "Le panneau /admin exige le rôle 'admin'. Personne n'a ce rôle... officiellement. Forge ton ticket.",
        "flag": "AFRI{jwt_faux_ticket}",
        "hints": [
            "Ta session est un JWT. Ouvre l'Atelier JWT (/forge) pour le décoder.",
            "Le serveur accepte l'algorithme <code>none</code> — ce qui veut dire: pas de signature.",
            "Dans l'Atelier: mets <code>\"alg\":\"none\"</code> dans le header, <code>\"role\":\"admin\"</code> dans le payload, laisse la signature VIDE, envoie, puis va sur /admin.",
        ],
        "solution": "Atelier JWT: header = <code>{\"alg\":\"none\",\"typ\":\"JWT\"}</code>, payload = <code>{\"user\":\"koffi\",\"role\":\"admin\",\"id\":1}</code>, signature vide. Envoie → ton cookie devient un faux ticket → /admin t'ouvre le panneau secret avec le flag.",
    },
    "traversal": {
        "titre": "🏜️ Défi 5 — Path Traversal : Le fichier du désert",
        "mission": "La page /fichier lit des fichiers par leur nom. Lis le fichier secret qui est HORS du dossier autorisé.",
        "flag": "AFRI{traversee_du_desert}",
        "hints": [
            "Va sur /fichiers et regarde comment on demande un fichier.",
            "En informatique, <code>../</code> veut dire 'le dossier au-dessus'.",
            "Essaie: <code>/fichier?nom=../secret/flag_du_desert.txt</code>",
        ],
        "solution": "Va sur <code>/fichier?nom=../secret/flag_du_desert.txt</code> — le serveur ne nettoie pas les <code>../</code> donc tu sors du dossier public et lis le fichier secret.",
    },
}

COMMENTS = [
    {"auteur": "Aminata", "texte": "Cette banque est super moderne !"},
    {"auteur": "Moussa", "texte": "J'ai déposé mon salaire aujourd'hui."},
]
COLLECT = []  # les cookies "volés" par le bot Directeur

# ============================================================
# JWT — avec la faille "alg none" plantée exprès
# ============================================================
def b64e(s):
    return base64.urlsafe_b64encode(s.encode()).decode().rstrip("=")

def b64d(s):
    s = s + "=" * (-len(s) % 4)
    return base64.urlsafe_b64decode(s.encode()).decode("utf-8", "replace")

def jwt_make(payload, alg="HS256"):
    header = json.dumps({"alg": alg, "typ": "JWT"})
    h, p = b64e(header), b64e(json.dumps(payload))
    if alg == "none":
        return h + "." + p + "."
    sig = hmac.new(SECRET_HS.encode(), (h + "." + p).encode(), hashlib.sha256).hexdigest()
    return h + "." + p + "." + b64e(sig)

def jwt_verify(token):
    try:
        parts = token.split(".")
        if len(parts) < 2:
            return None
        header = json.loads(b64d(parts[0]))
        payload = json.loads(b64d(parts[1]))
        if header.get("alg") == "none":   # ⚠️ LA FAILLE plantée
            return payload
        if len(parts) < 3 or not parts[2]:
            return None
        sig = b64e(hmac.new(SECRET_HS.encode(), (parts[0] + "." + parts[1]).encode(), hashlib.sha256).hexdigest())
        if sig != parts[2]:
            return None
        return payload
    except Exception:
        return None

# ============================================================
# Progression
# ============================================================
def load_prog():
    chemin = os.path.join(PROG_DIR, "progress.json")
    if os.path.isfile(chemin):
        try:
            with open(chemin) as f:
                return json.load(f)
        except Exception:
            pass
    return {}

def save_prog(d):
    os.makedirs(PROG_DIR, exist_ok=True)
    with open(os.path.join(PROG_DIR, "progress.json"), "w") as f:
        json.dump(d, f, indent=2)

# ============================================================
# HTML commun
# ============================================================
STYLE = """
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:Georgia,serif;background:#0a0f14;color:#d4e8f0;line-height:1.7}
header{background:linear-gradient(135deg,#1a2a3a,#2a4a5a);padding:16px 14px;text-align:center;border-bottom:3px solid #ffd700}
header h1{color:#ffd700;font-size:20px;margin-bottom:4px}
header p{color:#a8c8d8;font-size:11px}
.avert{background:#3a2a1a;border:2px solid #ff8c00;color:#ffd8a8;padding:10px;margin:10px auto;max-width:800px;border-radius:8px;font-size:12px;text-align:center}
nav{display:flex;justify-content:center;gap:8px;padding:10px;background:#0f1a22;flex-wrap:wrap}
nav a{color:#7fbfd8;text-decoration:none;font-size:12px;padding:6px 10px;border:1px solid #2a4a5a;border-radius:14px}
nav a:hover{border-color:#ffd700;color:#ffd700}
main{max-width:800px;margin:0 auto;padding:14px}
h2{color:#ffd700;font-size:17px;margin:20px 0 10px;border-bottom:1px solid #2a4a5a;padding-bottom:6px}
h3{color:#a8d8b8;font-size:14px;margin:14px 0 6px}
p{margin:8px 0;font-size:14px}
ul,ol{margin:8px 0;padding-left:22px;font-size:14px}
li{margin:4px 0}
pre{background:#0f1a22;border:1px solid #2a4a5a;border-radius:6px;padding:12px;overflow-x:auto;font-size:12px;color:#e8f8f8;margin:10px 0}
code{background:#1a2a3a;padding:2px 6px;border-radius:4px;color:#ffd700;font-size:12px}
a{color:#7fbfd8}
.carte{background:#0f1a22;border:1px solid #2a4a5a;border-radius:10px;padding:14px;margin:12px 0}
.carte h3{color:#ffd700;margin-top:0}
.ok{background:#1a3a1a;border:1px solid #4a8;border-radius:8px;padding:10px;color:#b8f8b8;margin:10px 0;font-size:13px}
.ko{background:#3a1a1a;border:1px solid #a44;border-radius:8px;padding:10px;color:#f8b8b8;margin:10px 0;font-size:13px}
.hintbox{background:#2a2a1a;border:1px solid #aa8;border-radius:8px;padding:10px;color:#e8e8b8;margin:8px 0;font-size:13px}
.solbox{background:#2a1a3a;border:1px solid #a8a;border-radius:8px;padding:10px;color:#e8b8e8;margin:8px 0;font-size:13px}
form{margin:12px 0}
input,textarea,select{width:100%;padding:10px;border:1px solid #2a4a5a;border-radius:8px;background:#0a0f14;color:#d4e8f0;font-size:13px;margin:6px 0;font-family:monospace}
button{padding:10px 16px;background:#1a3a4a;color:#ffd700;border:1px solid #2a4a5a;border-radius:8px;font-size:13px;cursor:pointer}
button:hover{background:#2a4a5a;border-color:#ffd700}
.flag{color:#ffd700;font-weight:bold;background:#2a2a1a;padding:6px 10px;border-radius:6px;display:inline-block;margin:6px 0}
.cmt{background:#0f1a22;border:1px solid #1a3a4a;border-radius:8px;padding:10px;margin:8px 0;font-size:13px}
.cmt b{color:#7fbfd8}
footer{text-align:center;padding:14px;color:#5a7a8a;font-size:11px;border-top:1px solid #1a2a3a;margin-top:20px}
table{width:100%;border-collapse:collapse;margin:10px 0;font-size:13px}
th,td{padding:8px;text-align:left;border-bottom:1px solid #1a2a3a}
th{color:#ffd700}
"""

def page(titre, corps):
    return ("<!DOCTYPE html><html lang='fr'><head><meta charset='UTF-8'>"
            "<meta name='viewport' content='width=device-width,initial-scale=1'>"
            "<title>" + html.escape(titre) + " — AfriCible</title>"
            "<style>" + STYLE + "</style></head><body>"
            "<header><h1>🏦 AfriCible — Banque du Sahel (FICTIVE)</h1>"
            "<p>Labo vulnérable d'entraînement au bug bounty — par Koffi Christ Olivier &amp; Letta-Chan</p></header>"
            "<div class='avert'>⚠️ SITE FICTIF VULNÉRABLE POUR APPRENTISSAGE — n'utilise ces techniques QUE sur des labs et programmes autorisés</div>"
            "<nav>"
            "<a href='/'>🏠 Accueil</a>"
            "<a href='/login'>🔑 Connexion</a>"
            "<a href='/profil'>👤 Profil</a>"
            "<a href='/commentaires'>💬 Commentaires</a>"
            "<a href='/fichiers'>📁 Fichiers</a>"
            "<a href='/forge'>🎫 Atelier JWT</a>"
            "<a href='/collect'>📡 Collecte</a>"
            "<a href='/challenges'>🏆 Défis</a>"
            "</nav><main>" + corps + "</main>"
            "<footer>AfriCible v1.0 — Le tatami du chasseur de failles 🦁💚</footer></body></html>")

def get_session(handler):
    cookie = handler.headers.get("Cookie", "")
    for part in cookie.split(";"):
        part = part.strip()
        if part.startswith("session="):
            return jwt_verify(part[8:])
    return None

# ============================================================
# Pages
# ============================================================
def page_accueil(handler):
    s = get_session(handler)
    if s:
        bienvenue = ("<div class='ok'>🔑 Connecté: <b>" + html.escape(str(s.get("user"))) +
                     "</b> (rôle: " + html.escape(str(s.get("role"))) + ")</div>")
    else:
        bienvenue = "<p>Non connecté. <a href='/login'>Connecte-toi</a> — compte test: <code>koffi / afrique2026</code></p>"
    return page("Accueil", """
<h2>🏦 Bienvenue à la Banque du Sahel (fictive)</h2>
""" + bienvenue + """
<h2>🏆 Ta mission</h2>
<p>Cette banque contient <b>5 failles plantées exprès</b>. Ton but: trouver les 5 flags cachés.</p>
<ol>
<li>🔓 <b>IDOR</b> — la note secrète de Moussa</li>
<li>💉 <b>SQL Injection</b> — connexion Directeur</li>
<li>⚡ <b>XSS</b> — le cookie du Directeur</li>
<li>🎫 <b>JWT</b> — le faux ticket admin</li>
<li>🏜️ <b>Path Traversal</b> — le fichier du désert</li>
</ol>
<p>Chaque flag ressemble à: <code>AFRI{...}</code>. Va sur <a href='/challenges'>🏆 Défis</a> pour voir tes missions et valider tes flags.</p>
<h2>📖 Comptes de test</h2>
<table>
<tr><th>Utilisateur</th><th>Mot de passe</th><th>Rôle</th></tr>
<tr><td>koffi</td><td>afrique2026</td><td>client</td></tr>
<tr><td>aminata</td><td>sahel123</td><td>client</td></tr>
<tr><td>moussa</td><td>niamey456</td><td>client</td></tr>
<tr><td>directeur</td><td>???</td><td>directeur</td></tr>
</table>
<p>Le mot de passe du Directeur est introuvable... ou presque~ 😉</p>
""")

def page_login(handler, msg="", msg_ok=False):
    s = get_session(handler)
    if s:
        corps = ("<div class='ok'>Déjà connecté: <b>" + html.escape(str(s.get("user"))) + "</b></div>"
                  "<p><a href='/logout'>Se déconnecter</a></p>")
        return page("Connexion", corps)
    m = ""
    if msg:
        m = ("<div class='" + ("ok" if msg_ok else "ko") + "'>" + msg + "</div>")
    return page("Connexion", m + """
<h2>🔑 Connexion à la Banque du Sahel</h2>
<form method='POST' action='/login'>
<input name='user' placeholder="Nom d'utilisateur" required>
<input name='pass' type='password' placeholder='Mot de passe' required>
<button type='submit'>Se connecter</button>
</form>
<p>Compte test: <code>koffi / afrique2026</code></p>
""")

def page_profil(handler):
    s = get_session(handler)
    if not s:
        return page("Profil", "<div class='ko'>Connecte-toi d'abord. <a href='/login'>🔑 Connexion</a></div>")
    qs = urllib.parse.parse_qs(urllib.parse.urlparse(handler.path).query)
    uid = int(qs.get("id", [s.get("id")])[0]) if qs.get("id", [s.get("id")])[0] else s.get("id")
    u = USERS.get(uid)
    if not u:
        return page("Profil", "<div class='ko'>Utilisateur introuvable.</div>")
    return page("Profil", """
<h2>👤 Profil de """ + html.escape(u["nom"]) + """</h2>
<table>
<tr><th>Identifiant</th><td>""" + str(u["id"]) + """</td></tr>
<tr><th>Nom d'utilisateur</th><td>""" + html.escape(u["user"]) + """</td></tr>
<tr><th>Rôle</th><td>""" + html.escape(u["role"]) + """</td></tr>
<tr><th>Solde</th><td>""" + str(u["solde"]) + """ FCFA</td></tr>
<tr><th>Note</th><td>""" + u["note"] + """</td></tr>
</table>
<p><i>(Tu regardes le profil id=""" + str(uid) + """ — ton id est """ + str(s.get("id")) + """)</i></p>
""")

def page_commentaires(handler, msg=""):
    s = get_session(handler)
    m = ("<div class='ok'>" + msg + "</div>") if msg else ""
    liste = ""
    for c in COMMENTS:
        liste += ("<div class='cmt'><b>" + html.escape(c["auteur"]) + ":</b> " +
                  c["texte"] + "</div>")   # ⚠️ texte AFFICHÉ BRUT — la faille XSS
    form = ""
    if s:
        form = """
<form method='POST' action='/commentaires'>
<textarea name='texte' rows='3' placeholder='Ton commentaire...' required></textarea>
<button type='submit'>Publier</button>
</form>"""
    else:
        form = "<p><a href='/login'>Connecte-toi</a> pour commenter.</p>"
    return page("Commentaires", m + """
<h2>💬 Commentaires des clients</h2>
<p><i>ℹ️ Le Directeur lit les commentaires chaque minute avec son cookie secret...</i></p>
""" + form + liste)

def page_collect(handler):
    lignes = ""
    for c in COLLECT:
        lignes += ("<div class='cmt'><b>[" + c["temps"] + "] Cookie intercepté:</b><br><code>" +
                   html.escape(c["cookie"]) + "</code><br><i>via le commentaire de " +
                   html.escape(c["origine"]) + "</i></div>")
    if not lignes:
        lignes = "<p><i>Aucun cookie intercepté pour l'instant. Le serveur de collecte attend...</i></p>"
    return page("Collecte", """
<h2>📡 Serveur de collecte (simulation)</h2>
<p>Les cookies volés par tes payloads XSS arrivent ici~</p>
""" + lignes)

def page_fichiers(handler):
    return page("Fichiers", """
<h2>📁 Documents de la banque</h2>
<ul>
<li><a href='/fichier?nom=lettre.txt'>📄 lettre.txt</a></li>
<li><a href='/fichier?nom=contrat_2026.txt'>📄 contrat_2026.txt</a></li>
</ul>
<p><i>Le système ne sert que les fichiers du dossier public... en théorie.</i></p>
""")

def page_forge(handler, msg=""):
    s_cookie = None
    cookie = handler.headers.get("Cookie", "")
    for part in cookie.split(";"):
        part = part.strip()
        if part.startswith("session="):
            s_cookie = part[8:]
    m = ("<div class='ok'>" + msg + "</div>") if msg else ""
    if not s_cookie:
        return page("Atelier JWT", m + "<div class='ko'>Connecte-toi d'abord pour avoir un token à décoder. <a href='/login'>🔑 Connexion</a></div>")
    try:
        parts = s_cookie.split(".")
        header = json.dumps(json.loads(b64d(parts[0])), indent=2)
        payload = json.dumps(json.loads(b64d(parts[1])), indent=2)
        sig = parts[2] if len(parts) > 2 else ""
    except Exception:
        return page("Atelier JWT", m + "<div class='ko'>Token illisible.</div>")
    return page("Atelier JWT", m + """
<h2>🎫 Atelier JWT — décortique ton ticket</h2>
<p>Ton token de session est un JWT: <b>header.payload.signature</b>. Modifie-le ici et renvoie-le au serveur~</p>
<form method='POST' action='/forge'>
<h3>Header (partie 1)</h3>
<textarea name='header' rows='4'>""" + html.escape(header) + """</textarea>
<h3>Payload (partie 2)</h3>
<textarea name='payload' rows='6'>""" + html.escape(payload) + """</textarea>
<h3>Signature (partie 3)</h3>
<input name='sig' value='""" + html.escape(sig) + """'>
<button type='submit'>🔨 Forger et utiliser ce token</button>
</form>
<p><i>Après le forge, tu seras redirigé vers /admin avec ton nouveau ticket...</i></p>
""")

def page_admin(handler):
    s = get_session(handler)
    if not s:
        return page("Admin", "<div class='ko'>Connecte-toi d'abord. <a href='/login'>🔑 Connexion</a></div>")
    role = str(s.get("role"))
    if role == "admin":
        return page("Admin", """
<div class='ok'>✅ ACCÈS ADMIN ACCORDÉ — ton faux ticket a fonctionné~</div>
<h2>🛡️ Panneau secret de la Banque du Sahel</h2>
<p>Bienvenue, administrateur improvisé. Voici ta récompense :</p>
<p class='flag'>AFRI{jwt_faux_ticket}</p>
<p>Tu viens de forger un JWT avec l'algorithme <code>none</code> — exactement comme les vrais pirates. Les vraies entreprises paient $1,000-$5,000 pour cette faille~</p>
""")
    return page("Admin", """
<div class='ko'>⛔ ACCÈS REFUSÉ</div>
<h2>🛡️ Panneau /admin</h2>
<p>Ce panneau exige le rôle <code>admin</code>. Ton rôle actuel: <code>""" + html.escape(role) + """</code>.</p>
<p><i>Même le Directeur (rôle <code>directeur</code>) ne peut pas entrer ici. Seul le rôle <code>admin</code> ouvre la porte...</i></p>
<p>Astuce: <a href='/forge'>🎫 Atelier JWT</a></p>
""")

def page_challenges(handler, msg="", msg_ok=False):
    prog = load_prog()
    m = ""
    if msg:
        m = "<div class='" + ("ok" if msg_ok else "ko") + "'>" + msg + "</div>"
    done = sum(1 for k in FLAGS if prog.get(k, {}).get("done"))
    cartes = ""
    for key, ch in FLAGS.items():
        p = prog.get(key, {})
        etat = "✅ RÉUSSI" if p.get("done") else ("🔓 En cours" if p.get("hints", 0) > 0 else "⬜ Non commencé")
        hints = ""
        for i in range(p.get("hints", 0)):
            if i < len(ch["hints"]):
                hints += "<div class='hintbox'>💡 Indice " + str(i + 1) + ": " + ch["hints"][i] + "</div>"
        sol = ""
        if p.get("sol"):
            sol = "<div class='solbox'>🔓 SOLUTION: " + ch["solution"] + "</div>"
        form_flag = ""
        if not p.get("done"):
            form_flag = ("<form method='POST' action='/valider'>"
                         "<input type='hidden' name='ch' value='" + key + "'>"
                         "<input name='flag' placeholder='AFRI{...}' required>"
                         "<button type='submit'>Valider le flag</button></form>")
        btns = ""
        if not p.get("done") and p.get("hints", 0) < len(ch["hints"]):
            btns += ("<form method='POST' action='/hint'>"
                     "<input type='hidden' name='ch' value='" + key + "'>"
                     "<button type='submit'>💡 Indice suivant</button></form>")
        if not p.get("sol") and not p.get("done"):
            btns += ("<form method='POST' action='/solution'>"
                     "<input type='hidden' name='ch' value='" + key + "'>"
                     "<button type='submit'>🔓 Révéler la solution (dernier recours)</button></form>")
        cartes += ("<div class='carte'><h3>" + ch["titre"] + " — " + etat + "</h3>"
                   "<p>" + ch["mission"] + "</p>" + hints + sol + form_flag + btns + "</div>")
    return page("Défis", m + """
<h2>🏆 Les 5 défis du Sahel — """ + str(done) + """/5 réussis</h2>
<p>Trouve chaque flag, colle-le ici pour valider. Ta progression est sauvegardée~</p>
""" + cartes + """
<form method='POST' action='/reset'><button type='submit'>🔄 Tout réinitialiser</button></form>
""")

# ============================================================
# Serveur
# ============================================================
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

    def _redirect(self, url, set_cookie=None):
        self.send_response(302)
        if set_cookie:
            self.send_header("Set-Cookie", set_cookie)
        self.send_header("Location", url)
        self.send_header("Content-Length", "0")
        self.end_headers()

    def _body(self):
        n = int(self.headers.get("Content-Length", 0))
        corps = self.rfile.read(n).decode("utf-8", "replace")
        champs = {}
        for paire in corps.split("&"):
            if "=" in paire:
                k, v = paire.split("=", 1)
                champs[k] = urllib.parse.unquote_plus(v)
        return champs

    def do_GET(self):
        p = urllib.parse.urlparse(self.path).path
        if p == "/":
            self._send(page_accueil(self))
        elif p == "/login":
            self._send(page_login(self))
        elif p == "/logout":
            self._redirect("/", "session=; Path=/")
        elif p == "/profil":
            self._send(page_profil(self))
        elif p == "/commentaires":
            self._send(page_commentaires(self))
        elif p == "/collect":
            self._send(page_collect(self))
        elif p == "/fichiers":
            self._send(page_fichiers(self))
        elif p == "/fichier":
            qs = urllib.parse.parse_qs(urllib.parse.urlparse(self.path).query)
            nom = qs.get("nom", [""])[0]
            if nom in FILES:   # ⚠️ aucune protection contre ../ — la faille traversal
                self._send(FILES[nom], "text/plain; charset=utf-8")
            else:
                self._send("Fichier introuvable: " + nom, "text/plain; charset=utf-8")
        elif p == "/forge":
            self._send(page_forge(self))
        elif p == "/admin":
            self._send(page_admin(self))
        elif p == "/challenges":
            self._send(page_challenges(self))
        else:
            self._send(page_accueil(self))

    def do_POST(self):
        p = urllib.parse.urlparse(self.path).path
        champs = self._body()

        if p == "/login":
            u, pw = champs.get("user", ""), champs.get("pass", "")
            # ⚠️ Simulation de requête SQL construite par concaténation — LA FAILLE SQLi
            query = "SELECT * FROM users WHERE user='" + u + "' AND pass='" + pw + "'"
            normalise = query.lower().replace(" ", "")
            if "'or'1'='1" in normalise:
                # La condition toujours vraie fait tomber la connexion Directeur
                token = jwt_make({"user": "directeur", "role": "directeur", "id": 4})
                self._send(page("Connexion", """
<div class='ok'>✅ CONNEXION ADMINISTRATEUR RÉUSSIE</div>
<p>Requête exécutée:</p>
<pre>""" + html.escape(query) + """</pre>
<p>Tu es entré sans le mot de passe du Directeur. Voici ta récompense :</p>
<p class='flag'>AFRI{sqli_royale}</p>
<p>Dans la vraie vie, cette faille paie <b>$1,000 - $20,000</b> en bug bounty. Ici, elle t'a ouvert la banque~</p>
"""))
                self._set_session(token)
                return
            trouve = None
            for uid, usr in USERS.items():
                if usr["user"] == u and usr["pass"] == pw:
                    trouve = usr
                    break
            if trouve:
                token = jwt_make({"user": trouve["user"], "role": trouve["role"], "id": trouve["id"]})
                self._redirect("/", "session=" + token + "; Path=/")
            else:
                self._send(page_login(self, "❌ Identifiants incorrects. (Requête: " + html.escape(query) + ")"))
            return

        if p == "/commentaires":
            s = get_session(self)
            if not s:
                self._send(page_commentaires(self, "Connecte-toi pour commenter."))
                return
            texte = champs.get("texte", "")
            auteur = str(s.get("user"))
            COMMENTS.append({"auteur": auteur, "texte": texte})
            # Simulation du bot Directeur qui visite et exécute les scripts
            if (re.search(r"document\.cookie", texte, re.I) and
                    re.search(r"<script|onerror|onload|onmouseover|<img|<svg|<body", texte, re.I)):
                COLLECT.append({
                    "temps": time.strftime("%H:%M:%S"),
                    "cookie": "session_directeur=AFRI{xss_du_directeur}",
                    "origine": auteur,
                })
                self._send(page_commentaires(self, "✅ Commentaire publié. Le Directeur l'a lu... et ton script s'est exécuté chez lui~ Va voir /collect 📡"))
            else:
                self._send(page_commentaires(self, "✅ Commentaire publié."))
            return

        if p == "/forge":
            header = champs.get("header", "{}")
            payload = champs.get("payload", "{}")
            sig = champs.get("sig", "")
            try:
                json.loads(header)
                json.loads(payload)
            except Exception:
                self._send(page_forge(self, "❌ Header ou payload n'est pas du JSON valide."))
                return
            token = b64e(header) + "." + b64e(payload) + "." + sig
            self._redirect("/admin", "session=" + token + "; Path=/")
            return

        if p == "/valider":
            prog = load_prog()
            ch = champs.get("ch", "")
            flag = champs.get("flag", "").strip()
            if ch in FLAGS and flag == FLAGS[ch]["flag"]:
                if ch not in prog:
                    prog[ch] = {}
                prog[ch]["done"] = True
                prog[ch]["done_at"] = time.strftime("%Y-%m-%d %H:%M")
                save_prog(prog)
                self._send(page_challenges(self, "🎉 FLAG VALIDÉ: " + ch + " ! Bien joué chasseur~", True))
            else:
                self._send(page_challenges(self, "❌ Ce flag n'est pas correct (ou défi inconnu). Réessaie~"))
            return

        if p == "/hint":
            prog = load_prog()
            ch = champs.get("ch", "")
            if ch in FLAGS:
                if ch not in prog:
                    prog[ch] = {}
                prog[ch]["hints"] = min(prog[ch].get("hints", 0) + 1, len(FLAGS[ch]["hints"]))
                save_prog(prog)
            self._send(page_challenges(self))
            return

        if p == "/solution":
            prog = load_prog()
            ch = champs.get("ch", "")
            if ch in FLAGS:
                if ch not in prog:
                    prog[ch] = {}
                prog[ch]["sol"] = True
                save_prog(prog)
            self._send(page_challenges(self, "🔓 Solution révélée pour " + ch + ". Essaie de la refaire seul demain~", True))
            return

        if p == "/reset":
            save_prog({})
            self._send(page_challenges(self, "🔄 Progression réinitialisée. Le tatami est propre~", True))
            return

        self._send(page_accueil(self))

    def _set_session(self, token):
        # utilisé après la connexion SQLi pour poser le cookie aussi
        pass

if __name__ == "__main__":
    os.makedirs(PROG_DIR, exist_ok=True)
    print("=" * 60)
    print("  🏦 AFRICIBLE v1.0 — Le Labo Vulnérable du Sahel")
    print("  5 failles plantées exprès: IDOR, SQLi, XSS, JWT, Traversal")
    print("=" * 60)
    print("  Compte test : koffi / afrique2026")
    print("  Défis       : http://localhost:" + str(PORT) + "/challenges")
    print("  Adresse     : http://localhost:" + str(PORT))
    print("=" * 60)
    print("  Ouvre Chrome: termux-open-url http://localhost:" + str(PORT))
    print("  Arrêter     : Ctrl+C")
    print()
    print("  ⚠️  SITE FICTIF POUR APPRENTISSAGE UNIQUEMENT")
    print()
    try:
        with Serveur(("0.0.0.0", PORT), Handler) as httpd:
            httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n👋 AfriCible se ferme. Ta progression est sauvegardée~")
