#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# ============================================================
# AFRIHUNTER v1.0 — L'École du Chasseur de Failles
# Par Koffi Christ Olivier & Letta-Chan
# Python std only — Apprendre le bug bounty depuis zéro
# "Le lion apprend à chasser en chassant." 🦁
# ============================================================
import http.server
import socketserver
import os, json, html, urllib.parse, hashlib, random, time

PORT = 8096
DOSSIER = os.path.join(os.path.expanduser("~"), "afrihunter_progress")

# ============================================================
# LEÇONS COMPLÈTES — De zéro à chasseur
# ============================================================
LECONS = {
    "1": {
        "titre": "🌐 Qu'est-ce qu'un site web ?",
        "niveau": "Débutant total",
        "temps": "15 min",
        "contenu": """
<h2>🌐 Comment fonctionne un site web ?</h2>
<p>Avant de pirater, tu dois comprendre ce que tu attaques. Un site web, c'est comme un restaurant :</p>

<h3>🍽️ L'analogie du restaurant</h3>
<ul>
<li><b>Toi (le client)</b> = Le navigateur (Chrome)</li>
<li><b>Le serveur</b> = Le serveur web (nginx, Apache)</li>
<li><b>Ta commande</b> = La requête HTTP (GET, POST)</li>
<li><b>L'assiette</b> = La réponse (HTML, JSON)</li>
<li><b>Le menu</b> = Les URLs disponibles</li>
</ul>

<h3>📡 HTTP — Le langage du web</h3>
<p>HTTP = HyperText Transfer Protocol. C'est le langage que ton navigateur parle aux serveurs.</p>

<p><b>Exemple de requête HTTP :</b></p>
<pre>GET /profil?id=123 HTTP/1.1
Host: exemple.com
Cookie: session=abc123
User-Agent: Chrome Mobile</pre>

<p><b>Exemple de réponse HTTP :</b></p>
<pre>HTTP/1.1 200 OK
Content-Type: text/html
Set-Cookie: session=xyz789

&lt;html&gt;&lt;body&gt;Bienvenue Koffi&lt;/body&gt;&lt;/html&gt;</pre>

<h3>🎯 Les méthodes importantes</h3>
<ul>
<li><b>GET</b> = Demander une page (comme lire le menu)</li>
<li><b>POST</b> = Envoyer des données (comme passer commande)</li>
<li><b>PUT</b> = Modifier quelque chose</li>
<li><b>DELETE</b> = Supprimer quelque chose</li>
</ul>

<h3>🍪 Les cookies — Ton bracelet d'entrée</h3>
<p>Quand tu te connectes à un site, le serveur te donne un <b>cookie</b> — comme un bracelet de festival. À chaque requête, tu montres ton bracelet, et le serveur sait qui tu es.</p>
<p><b>Cookie volé = Identité volée.</b> C'est pour ça que les pirates adorent voler les cookies.</p>

<h3>🔧 Exercice pratique</h3>
<p>Ouvre Chrome et appuie sur <b>F12</b> (ou menu → Plus d'outils → Outils de développement). Clique sur l'onglet <b>Network</b> (Réseau). Actualise la page. Tu vois TOUTES les requêtes HTTP que ton navigateur envoie.</p>
<p>C'est là que le chasseur regarde. C'est son microscope~ 🔬</p>
""",
        "exercice": "Dans Chrome, ouvre les DevTools (F12), va sur Network, et trouve une requête. Quel est le 'Status Code' ?",
        "reponse_attendue": ["200", "404", "301", "302", "403", "500"],
    },
    "2": {
        "titre": "🔓 IDOR — La faille qui paie le plus",
        "niveau": "Débutant",
        "temps": "20 min",
        "contenu": """
<h2>🔓 IDOR — Insecure Direct Object Reference</h2>
<p><b>C'est la faille la plus rentable pour les débutants.</b> Pourquoi ? Parce qu'elle est simple à comprendre et les entreprises paient cher pour la corriger.</p>

<h3>🎯 Le problème</h3>
<p>Imagine un site avec des profils utilisateurs. Chaque profil a un numéro :</p>
<ul>
<li><b>/profil?id=1</b> = Profil de Koffi</li>
<li><b>/profil?id=2</b> = Profil de Aminata</li>
<li><b>/profil?id=3</b> = Profil de Moussa</li>
</ul>

<p>Tu es connecté comme Koffi (id=1). Tu changes l'URL en <b>/profil?id=2</b>...</p>
<p><b>Si tu vois le profil d'Aminata → C'EST UNE FAILLE IDOR !</b></p>

<h3>🍽️ L'analogie du restaurant</h3>
<p>Tu commandes au serveur : "Je veux l'assiette #5". Le serveur t'apporte l'assiette #5 sans vérifier si c'est TON assiette. Tu peux manger l'assiette de n'importe qui.</p>
<p><b>Le serveur devrait vérifier : "Est-ce que l'assiette #5 appartient à ce client ?"</b></p>

<h3>💰 Combien ça paie ?</h3>
<ul>
<li>IDOR basique (voir le profil d'un autre) : <b>$500 - $2,000</b></li>
<li>IDOR critique (modifier les données d'un autre) : <b>$2,000 - $10,000</b></li>
<li>IDOR massif (accéder à TOUS les utilisateurs) : <b>$10,000+</b></li>
</ul>

<h3>🔍 Comment la trouver ?</h3>
<ol>
<li>Connecte-toi au site</li>
<li>Ouvre DevTools → Network</li>
<li>Cherche les URLs avec des <b>numéros</b> : <code>?id=123</code>, <code>/user/456</code>, <code>?user_id=789</code></li>
<li>Change le numéro pour celui d'un autre utilisateur</li>
<li>Si tu vois ses données → <b>RAPPORT !</b></li>
</ol>

<h3>🎯 Types d'IDOR courants</h3>
<ul>
<li><b>URL parameters</b> : <code>/api/user/123</code></li>
<li><b>Query parameters</b> : <code>/profil?user_id=123</code></li>
<li><b>POST data</b> : <code>user_id=123</code> dans le corps</li>
<li><b>Headers</b> : <code>X-User-ID: 123</code></li>
<li><b>Cookies</b> : <code>user_id=123</code></li>
</ul>

<h3>🔧 Exercice pratique</h3>
<p>Sur n'importe quel site où tu as un compte, essaie de trouver une URL avec ton ID. Change-le pour voir ce qui se passe.</p>
""",
        "exercice": "Nomme UN endroit où un ID peut être caché dans une requête HTTP",
        "reponse_attendue": ["url", "query", "post", "header", "cookie", "parametre", "paramètre", "corps", "body"],
    },
    "3": {
        "titre": "💉 SQL Injection — La faille légendaire",
        "niveau": "Intermédiaire",
        "temps": "30 min",
        "contenu": """
<h2>💉 SQL Injection — Le couteau dans la base de données</h2>
<p>C'est la faille la plus célèbre de l'histoire. Elle a fait perdre des <b>millions de dollars</b> aux entreprises. Et elle existe encore aujourd'hui~</p>

<h3>🗃️ Qu'est-ce que SQL ?</h3>
<p>SQL = Structured Query Language. C'est le langage pour parler aux bases de données.</p>
<pre>SELECT * FROM users WHERE id = 1</pre>
<p>Cette commande dit : "Donne-moi TOUTES les infos de l'utilisateur numéro 1."</p>

<h3>🎯 Le problème</h3>
<p>Imagine un site avec une recherche :</p>
<pre>SELECT * FROM products WHERE name = 'iphone'</pre>
<p>Le site prend ce que tu tapes et le met directement dans la requête. Que se passe-t-il si tu tapes :</p>
<pre>' OR '1'='1</pre>
<p>La requête devient :</p>
<pre>SELECT * FROM products WHERE name = '' OR '1'='1'</pre>
<p><b>'1'='1' est toujours VRAI !</b> Donc la requête renvoie TOUS les produits — ou pire, TOUS les utilisateurs.</p>

<h3>🍽️ L'analogie du restaurant</h3>
<p>Tu dis au serveur : "Je veux l'assiette du client qui s'appelle 'N'IMPORTE QUI'". Le serveur ne comprend pas que c'est une blague — il va chercher dans toute la base de données.</p>

<h3>💀 Les dégâts possibles</h3>
<ul>
<li><b>Vol de données</b> : mots de passe, emails, numéros de carte</li>
<li><b>Modification</b> : changer les prix, les soldes</li>
<li><b>Suppression</b> : effacer toute la base</li>
<li><b>Authentification bypass</b> : se connecter sans mot de passe</li>
</ul>

<h3>💰 Combien ça paie ?</h3>
<ul>
<li>SQL Injection basique : <b>$1,000 - $5,000</b></li>
<li>SQL Injection avec vol de données : <b>$5,000 - $20,000</b></li>
<li>SQL Injection critique (base de données entière) : <b>$20,000 - $100,000+</b></li>
</ul>

<h3>🔍 Comment la trouver ?</h3>
<ol>
<li>Cherche les <b>champs de saisie</b> : recherche, login, formulaire</li>
<li>Tape un <b>guillemet simple</b> : <code>'</code></li>
<li>Si tu vois une <b>erreur SQL</b> → C'EST VULNÉRABLE !</li>
<li>Si pas d'erreur, essaie : <code>' OR '1'='1' --</code></li>
</ol>

<h3>🛡️ Comment les développeurs corrigent ?</h3>
<p>Ils utilisent des <b>requêtes préparées</b> (prepared statements) — les données sont séparées du code SQL. Mais beaucoup de sites oublient...</p>

<h3>🔧 Exercice pratique</h3>
<p>Dans AfriLab, tu as déjà vu SQL Injection en action. Relance-le et essaie de te connecter sans mot de passe~</p>
""",
        "exercice": "Quel caractère simple suffit pour tester une SQL Injection ?",
        "reponse_attendue": ["'", "guillemet", "quote", "apostrophe"],
    },
    "4": {
        "titre": "⚡ XSS — Voler les cookies",
        "niveau": "Intermédiaire",
        "temps": "25 min",
        "contenu": """
<h2>⚡ XSS — Cross-Site Scripting</h2>
<p>XSS = Injecter du code JavaScript malveillant dans un site web. C'est comme si tu écrivais ton propre menu dans le restaurant, et que le serveur le servait à tous les clients~</p>

<h3>🎯 Le problème</h3>
<p>Imagine un site avec des commentaires. Tu écris :</p>
<pre>&lt;script&gt;alert('Salut!')&lt;/script&gt;</pre>
<p>Si le site affiche ce commentaire <b>sans le nettoyer</b>, ton code s'exécute dans le navigateur de TOUS les visiteurs.</p>

<h3>💀 Les dégâts possibles</h3>
<ul>
<li><b>Vol de cookies</b> : <code>document.cookie</code> → tu envoies les cookies à ton serveur</li>
<li><b>Keylogging</b> : enregistrer tout ce que la victime tape</li>
<li><b>Phishing</b> : afficher un faux formulaire de connexion</li>
<li><b>Redirection</b> : envoyer la victime sur un site malveillant</li>
</ul>

<h3>🍽️ L'analogie du restaurant</h3>
<p>Tu écris sur ta commande : "ET AUSSI, donne-moi le portefeuille du client à côté". Le serveur lit tout haut ta commande, et le client à côté obéit~</p>

<h3>💰 Combien ça paie ?</h3>
<ul>
<li>XSS réfléchi (Reflected) : <b>$500 - $3,000</b></li>
<li>XSS stocké (Stored) : <b>$1,000 - $10,000</b></li>
<li>XSS basé sur DOM : <b>$500 - $5,000</b></li>
</ul>

<h3>🔍 Les trois types de XSS</h3>
<ol>
<li><b>Reflected XSS</b> : Le code malveillant est dans l'URL. Exemple : <code>?q=&lt;script&gt;...&lt;/script&gt;</code></li>
<li><b>Stored XSS</b> : Le code est SAUVEGARDÉ dans la base de données (commentaires, profils). C'est le plus dangereux.</li>
<li><b>DOM-based XSS</b> : Le code s'exécute côté client, sans aller au serveur.</li>
</ol>

<h3>🧪 Comment tester ?</h3>
<p>Dans n'importe quel champ de saisie, tape :</p>
<pre>&lt;script&gt;alert('XSS')&lt;/script&gt;</pre>
<p>Si une alerte apparaît → C'EST VULNÉRABLE !</p>

<p>Si ça ne marche pas, essaie les variantes :</p>
<pre>&lt;img src=x onerror="alert('XSS')"&gt;
&lt;svg onload="alert('XSS')"&gt;
&lt;body onload="alert('XSS')"&gt;</pre>

<h3>🔧 Exercice pratique</h3>
<p>Sur un site avec des commentaires ou un profil, essaie d'injecter un script. Attention : teste seulement sur des sites de bug bounty ou des labs~</p>
""",
        "exercice": "Quelle fonction JavaScript permet de voler les cookies ?",
        "reponse_attendue": ["document.cookie", "cookie", "cookies"],
    },
    "5": {
        "titre": "🎫 JWT — Les faux tickets",
        "niveau": "Avancé",
        "temps": "30 min",
        "contenu": """
<h2>🎫 JWT — JSON Web Tokens</h2>
<p>JWT = Un "ticket d'identité" numérique que le serveur donne au client. C'est très utilisé pour les connexions modernes.</p>

<h3>🎯 Structure d'un JWT</h3>
<p>Un JWT ressemble à ça :</p>
<pre>eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyIjoiS29mZmkiLCJyb2xlIjoiYWRtaW4ifQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c</pre>

<p>C'est <b>3 parties</b> séparées par des points :</p>
<ol>
<li><b>Header</b> (algorithme) — Toujours lisible</li>
<li><b>Payload</b> (données) — Toujours lisible : <code>{"user":"Koffi","role":"user"}</code></li>
<li><b>Signature</b> — Empêche la modification... en théorie~</li>
</ol>

<h3>🔓 Les failles JWT courantes</h3>

<h4>1. Algorithme "none"</h4>
<p>Certains serveurs acceptent l'algorithme "none" — ce qui signifie <b>pas de signature</b> !</p>
<pre>{"alg":"none","typ":"JWT"}</pre>
<p>Tu peux modifier le payload sans être détecté.</p>

<h4>2. Changer RS256 en HS256</h4>
<p>RS256 = Signature avec clé privée/publique. HS256 = Signature avec secret. Si le serveur utilise la clé publique comme secret HS256... tu peux forger des tokens !</p>

<h4>3. Secret faible</h4>
<p>Si le secret est simple (ex: "secret123"), tu peux le deviner avec une attaque par force brute.</p>

<h4>4. Pas de vérification</h4>
<p>Certains serveurs vérifient seulement le format, pas la signature~</p>

<h3>💰 Combien ça paie ?</h3>
<ul>
<li>JWT avec algorithme "none" : <b>$1,000 - $5,000</b></li>
<li>JWT avec secret faible : <b>$2,000 - $10,000</b></li>
<li>JWT avec confusion RS256/HS256 : <b>$5,000 - $20,000</b></li>
</ul>

<h3>🔧 Outils</h3>
<p><b>jwt.io</b> — Pour décoder et modifier les JWT</p>
<p><b>jwt_tool</b> — Pour les attaques automatiques</p>

<h3>🔧 Exercice pratique</h3>
<p>Si tu vois un JWT dans tes cookies (DevTools → Application → Cookies), copie-le sur jwt.io et regarde ce qu'il contient~</p>
""",
        "exercice": "Combien de parties a un JWT (séparées par des points) ?",
        "reponse_attendue": ["3", "trois", "three"],
    },
    "6": {
        "titre": "🎯 PortSwigger Academy — L'école gratuite",
        "niveau": "Pratique",
        "temps": "1 heure par lab",
        "contenu": """
<h2>🎯 PortSwigger Web Security Academy</h2>
<p><b>C'est la MEILLEURE ressource gratuite au monde.</b> PortSwigger est la compagnie qui fait Burp Suite, l'outil des professionnels.</p>

<h3>📚 Ce que tu apprends</h3>
<ul>
<li>SQL Injection (tous les niveaux)</li>
<li>XSS (Reflected, Stored, DOM-based)</li>
<li>CSRF</li>
<li>IDOR</li>
<li>JWT</li>
<li>SSRF</li>
<li>File Upload</li>
<li>Path Traversal</li>
<li>...et des dizaines d'autres !</li>
</ul>

<h3>🎮 Comment ça marche ?</h3>
<ol>
<li>Va sur <b>portswigger.net/web-security</b></li>
<li>Choisis un sujet (commence par SQL Injection)</li>
<li>Lis l'explication</li>
<li>Fais le lab — c'est un vrai site vulnérable</li>
<li>Trouve la faille et valide</li>
</ol>

<h3>🏆 La preuve de tes compétences</h3>
<p>Quand tu complètes un lab, tu peux le montrer sur ton profil HackerOne. Les entreprises voient que tu as de l'expérience~</p>

<h3>📱 Sur Termux</h3>
<p>Tu peux faire les labs directement dans Chrome. Pas besoin d'outils complexes au début~</p>

<h3>🔧 Exercice</h3>
<p>Inscris-toi sur PortSwigger et fais ton premier lab SQL Injection~</p>
""",
        "exercice": "Quel est le site de PortSwigger pour apprendre ?",
        "reponse_attendue": ["portswigger", "portswigger.net", "portswigger.net/web-security"],
    },
    "7": {
        "titre": "🦁 HackerOne — Ton premier bounty",
        "niveau": "Action",
        "temps": "1 heure pour s'inscrire",
        "contenu": """
<h2>🦁 HackerOne — La plateforme de bug bounty</h2>
<p><b>C'est là que tu vas gagner de l'argent.</b> HackerOne connecte les chasseurs aux entreprises qui paient.</p>

<h3>📋 Comment s'inscrire</h3>
<ol>
<li>Va sur <b>hackerone.com</b></li>
<li>Crée un compte avec ton email</li>
<li>Remplis ton profil — mets tes labs PortSwigger</li>
<li>Active les notifications par email</li>
</ol>

<h3>🎯 Les programmes pour débutants</h3>
<p>Cherche les programmes avec le label <b>"New"</b> ou <b>"Good for new hackers"</b>. Ils sont plus indulgents~</p>

<h3>⚠️ Les règles à ne JAMAIS casser</h3>
<ul>
<li><b>Ne teste que les sites autorisés</b> — HackerOne liste les domaines</li>
<li><b>Ne cause pas de dégâts</b> — tu es un chasseur, pas un vandale</li>
<li><b>Ne vole pas de données</b> — rapporte juste la faille</li>
<li><b>Ne parle pas publiquement</b> — garde le secret jusqu'à correction</li>
</ul>

<h3>💰 Comment tu reçois l'argent</h3>
<p>HackerOne paie en <b>USD</b> via PayPal ou virement bancaire. Tu peux convertir en FCFA~</p>

<h3>🔧 Ton premier objectif</h3>
<p>Trouve UN programme avec "Good for new hackers", lis leurs règles, et cherche une IDOR~</p>
""",
        "exercice": "Quel label cherche-tu pour les programmes débutants sur HackerOne ?",
        "reponse_attendue": ["new", "good for new hackers", "beginner", "débutant", "good for new"],
    },
    "8": {
        "titre": "📝 Écrire un rapport qui paie",
        "niveau": "Crucial",
        "temps": "30 min",
        "contenu": """
<h2>📝 Comment écrire un rapport de bug bounty</h2>
<p><b>Ton rapport = Ton chèque.</b> Un mauvais rapport peut être rejeté. Un bon rapport montre que tu es professionnel~</p>

<h3>📋 Structure d'un bon rapport</h3>

<h4>1. Titre clair</h4>
<pre>IDOR sur /api/user/{id} permet d'accéder aux données d'autres utilisateurs</pre>

<h4>2. Description</h4>
<p>Explique le problème en 2-3 phrases. <b>Qu'est-ce qui ne va pas ?</b></p>

<h4>3. Étapes pour reproduire</h4>
<p>C'est le plus important ! Sois précis~</p>
<pre>1. Connecte-toi avec le compte A
2. Va sur /profil?id=1
3. Note les données affichées
4. Connecte-toi avec le compte B
5. Va sur /profil?id=1 (le profil de A)
6. Tu vois les données de A → FAILLE IDOR</pre>

<h4>4. Preuves</h4>
<p><b>Captures d'écran</b> avec les DevTools ouverts. Montre la requête et la réponse~</p>

<h4>5. Impact</h4>
<p>Qu'est-ce qu'un attaquant peut faire ?</p>
<pre>- Voler les informations personnelles de TOUS les utilisateurs
- Accéder aux emails, numéros de téléphone, adresses
- Potentiellement prendre le contrôle des comptes</pre>

<h4>6. Recommandation</h4>
<p>Comment corriger ?</p>
<pre>Vérifier que l'utilisateur connecté a le droit d'accéder à l'ID demandé avant de renvoyer les données.</pre>

<h3>⚠️ Erreurs à éviter</h3>
<ul>
<li>Rapport trop court → Rejeté</li>
<li>Pas de preuves → Impossible à vérifier</li>
<li>Rapport en colère → Tu passes pour un amateur</li>
<li>Duplique → Vérifie que personne n'a déjà rapporté~</li>
</ul>

<h3>🔧 Exercice</h3>
<p>Écris un rapport fictif pour une IDOR imaginaire. Entraîne-toi~</p>
""",
        "exercice": "Quelle est la partie la plus importante d'un rapport ?",
        "reponse_attendue": ["étapes", "etape", "reproduire", "steps", "preuves", "proof"],
    },
}

# ============================================================
# INTERFACE HTML
# ============================================================
def page_accueil(progress=None):
    msg = ""
    if progress:
        msg = f"<div class='msg'>{progress}</div>"
    
    cartes = ""
    for num, lecon in LECONS.items():
        status = ""
        if progress and progress.get(num, {}).get("complete"):
            status = "✅"
        cartes += f"""
<div class='carte' onclick="location='/lecon/{num}'">
<div class='num'>{status}</div>
<div class='titre'>{lecon['titre']}</div>
<div class='meta'>{lecon['niveau']} • {lecon['temps']}</div>
</div>"""
    
    return f"""<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>AfriHunter — École du Chasseur de Failles</title>
<style>
*{{box-sizing:border-box;margin:0;padding:0}}
body{{font-family:Georgia,serif;background:#0a0f0a;color:#d4e8d4;line-height:1.6}}
header{{background:linear-gradient(135deg,#1a3a1a,#2a5a2a);padding:18px 14px;text-align:center;border-bottom:3px solid #ffd700}}
header h1{{color:#ffd700;font-size:22px;margin-bottom:6px}}
header p{{color:#b8d8b8;font-size:12px}}
nav{{display:flex;justify-content:center;gap:10px;padding:12px;background:#0f1f0f;flex-wrap:wrap}}
nav a{{color:#7fbf7f;text-decoration:none;font-size:13px;padding:6px 12px;border:1px solid #2a5a2a;border-radius:16px}}
nav a:hover{{background:#1a3a1a;border-color:#ffd700}}
main{{max-width:800px;margin:0 auto;padding:14px}}
h2{{color:#ffd700;font-size:18px;margin:20px 0 12px;border-bottom:1px solid #2a5a2a;padding-bottom:8px}}
.carte{{background:#0f1f0f;border:1px solid #2a5a2a;border-radius:10px;padding:14px;margin:10px 0;cursor:pointer;transition:all .15s;display:flex;align-items:center;gap:14px}}
.carte:hover{{border-color:#ffd700;transform:translateY(-2px);background:#1a2a1a}}
.carte .num{{font-size:24px;min-width:32px}}
.carte .titre{{color:#e8f8e8;font-size:15px;font-weight:bold;flex:1}}
.carte .meta{{color:#8fa88f;font-size:11px}}
.msg{{background:#1a3a1a;border:1px solid #4a8;border-radius:8px;padding:12px;margin:10px 0;color:#b8f8b8;font-size:13px}}
.stats{{display:flex;gap:12px;margin:14px 0;flex-wrap:wrap}}
.stat{{background:#0f1f0f;border:1px solid #2a5a2a;border-radius:8px;padding:12px;flex:1;min-width:100px;text-align:center}}
.stat .val{{color:#ffd700;font-size:24px;font-weight:bold}}
.stat .lab{{color:#8fa88f;font-size:11px;margin-top:4px}}
footer{{text-align:center;padding:16px;color:#5a7a5a;font-size:11px;border-top:1px solid #1a3a1a;margin-top:20px}}
</style></head><body>
<header><h1>🦁 AfriHunter — L'École du Chasseur de Failles</h1>
<p>Apprends à trouver les bugs qui paient. De zéro à bounty en 8 leçons. Par Koffi Christ Olivier & Letta-Chan 💚</p></header>
<nav>
<a href="/">🏠 Accueil</a>
<a href="/progress">📊 Ma progression</a>
<a href="/outils">🔧 Outils</a>
</nav>
<main>
{msg}
<div class='stats'>
<div class='stat'><div class='val'>{len(LECONS)}</div><div class='lab'>Leçons</div></div>
<div class='stat'><div class='val'>{sum(1 for k in LECONS if progress and progress.get(k, {}).get("complete"))}</div><div class='lab'>Complétées</div></div>
<div class='stat'><div class='val'>∞</div><div class='lab'>Potentiel $</div></div>
</div>
<h2>📚 Programme complet — De zéro à chasseur</h2>
{cartes}
</main>
<footer>AfriHunter v1.0 — Le lion apprend à chasser en chassant 🦁💚</footer>
</body></html>"""

def page_lecon(num, progress=None):
    if num not in LECONS:
        return page_accueil("Leçon introuvable.")
    
    lecon = LECONS[num]
    complete = progress and progress.get(num, {}).get("complete", False)
    
    btn = ""
    if not complete:
        btn = f"""<form method="POST" action="/complete/{num}"><button type="submit">✅ Marquer comme terminée</button></form>"""
    else:
        btn = f"""<div class='success'>✅ Leçon terminée ! Tu peux passer à la suite~</div>"""
    
    exercice = ""
    if lecon.get("exercice"):
        exercice = f"""
<h2>🎯 Exercice pratique</h2>
<p>{lecon['exercice']}</p>
<form method="POST" action="/repondre/{num}">
<input type="text" name="reponse" placeholder="Ta réponse..." required>
<button type="submit">Envoyer</button>
</form>
"""
    
    return f"""<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{lecon['titre']} — AfriHunter</title>
<style>
*{{box-sizing:border-box;margin:0;padding:0}}
body{{font-family:Georgia,serif;background:#0a0f0a;color:#d4e8d4;line-height:1.7;padding:14px;max-width:800px;margin:0 auto}}
a{{color:#7fbf7f}}a:hover{{color:#ffd700}}
h1{{color:#ffd700;font-size:20px;margin-bottom:6px}}
h2{{color:#ffd700;font-size:17px;margin:24px 0 12px;border-bottom:1px solid #2a5a2a;padding-bottom:8px}}
h3{{color:#b8d8b8;font-size:15px;margin:18px 0 8px}}
h4{{color:#a8c8a8;font-size:13px;margin:14px 0 6px}}
p{{margin:10px 0}}
ul,ol{{margin:10px 0;padding-left:24px}}
li{{margin:6px 0}}
pre{{background:#0f1f0f;border:1px solid #2a5a2a;border-radius:6px;padding:12px;overflow-x:auto;font-size:13px;color:#e8f8e8;margin:12px 0}}
.retour{{display:inline-block;margin:0 0 14px;padding:8px 14px;background:#1a3a1a;color:#b8d8b8;border-radius:16px;text-decoration:none;font-size:13px}}
.retour:hover{{background:#2a5a2a;color:#ffd700}}
.meta{{color:#8fa88f;font-size:12px;margin-bottom:14px}}
form{{margin:16px 0}}
input{{width:100%;padding:12px;border:1px solid #2a5a2a;border-radius:8px;background:#0f1f0f;color:#d4e8d4;font-size:14px;margin:8px 0}}
button{{padding:12px 20px;background:#1a3a1a;color:#ffd700;border:1px solid #2a5a2a;border-radius:8px;font-size:14px;cursor:pointer}}
button:hover{{background:#2a5a2a;border-color:#ffd700}}
.success{{background:#1a3a1a;border:1px solid #4a8;border-radius:8px;padding:12px;color:#b8f8b8;margin:12px 0}}
.error{{background:#3a1a1a;border:1px solid #a44;border-radius:8px;padding:12px;color:#f8b8b8;margin:12px 0}}
</style></head><body>
<a class='retour' href='/'>← Accueil</a>
<h1>{lecon['titre']}</h1>
<p class='meta'>{lecon['niveau']} • {lecon['temps']}</p>
{lecon['contenu']}
{exercice}
{btn}
</body></html>"""

def page_progress(progress=None):
    if not progress:
        progress = {}
    
    lignes = ""
    total = 0
    complete = 0
    
    for num, lecon in LECONS.items():
        total += 1
        status = "⬜"
        if progress.get(num, {}).get("complete"):
            status = "✅"
            complete += 1
        score = progress.get(num, {}).get("score", 0)
        lignes += f"<tr><td>{status}</td><td>{lecon['titre']}</td><td>{score}/1</td></tr>"
    
    pct = int((complete / total) * 100) if total > 0 else 0
    
    return f"""<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Ma progression — AfriHunter</title>
<style>
*{{box-sizing:border-box;margin:0;padding:0}}
body{{font-family:Georgia,serif;background:#0a0f0a;color:#d4e8d4;padding:14px;max-width:800px;margin:0 auto}}
h1{{color:#ffd700;font-size:20px;margin-bottom:14px}}
h2{{color:#b8d8b8;font-size:16px;margin:20px 0 10px}}
table{{width:100%;border-collapse:collapse;margin:14px 0}}
th,td{{padding:10px;text-align:left;border-bottom:1px solid #1a3a1a}}
th{{color:#ffd700;font-size:13px}}
td{{font-size:13px}}
.bar{{background:#0f1f0f;border-radius:10px;height:24px;overflow:hidden;margin:14px 0}}
.fill{{background:linear-gradient(90deg,#2a5a2a,#4a8a4a);height:100%;width:{pct}%;display:flex;align-items:center;justify-content:center;color:#ffd700;font-size:12px;font-weight:bold}}
.retour{{display:inline-block;margin:0 0 14px;padding:8px 14px;background:#1a3a1a;color:#b8d8b8;border-radius:16px;text-decoration:none;font-size:13px}}
</style></head><body>
<a class='retour' href='/'>← Accueil</a>
<h1>📊 Ma progression</h1>
<div class='bar'><div class='fill'>{pct}%</div></div>
<p>{complete}/{total} leçons terminées</p>
<h2>📋 Détail par leçon</h2>
<table><tr><th>✓</th><th>Leçon</th><th>Score</th></tr>
{lignes}
</table>
</body></html>"""

def page_outils():
    return """<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Outils — AfriHunter</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:Georgia,serif;background:#0a0f0a;color:#d4e8d4;padding:14px;max-width:800px;margin:0 auto}
h1{color:#ffd700;font-size:20px;margin-bottom:14px}
h2{color:#b8d8b8;font-size:16px;margin:20px 0 10px}
.tool{background:#0f1f0f;border:1px solid #2a5a2a;border-radius:10px;padding:14px;margin:10px 0}
.tool h3{color:#ffd700;font-size:14px;margin-bottom:8px}
.tool p{font-size:13px;margin:6px 0}
.tool a{color:#7fbf7f}
code{background:#1a2a1a;padding:2px 6px;border-radius:4px;color:#ffd700;font-size:12px}
.retour{display:inline-block;margin:0 0 14px;padding:8px 14px;background:#1a3a1a;color:#b8d8b8;border-radius:16px;text-decoration:none;font-size:13px}
</style></head><body>
<a class='retour' href='/'>← Accueil</a>
<h1>🔧 Outils du chasseur</h1>
<h2>📱 Sur Termux (ton téléphone)</h2>
<div class='tool'>
<h3>Chrome DevTools</h3>
<p>Déjà installé~ Appuie sur F12 ou menu → Plus d'outils → Outils de développement.</p>
<p><b>Network</b> = voir les requêtes | <b>Application</b> = voir les cookies et JWT</p>
</div>
<div class='tool'>
<h3>curl</h3>
<p>Envoyer des requêtes HTTP manuelles.</p>
<code>curl https://exemple.com/api/users</code>
</div>
<div class='tool'>
<h3>python3</h3>
<p>Scripts personnalisés pour tester les failles.</p>
</div>
<h2>🌐 Sur le web</h2>
<div class='tool'>
<h3>jwt.io</h3>
<p>Décoder et modifier les JSON Web Tokens.</p>
<a href="https://jwt.io">https://jwt.io</a>
</div>
<div class='tool'>
<h3>PortSwigger Academy</h3>
<p>Labs pratiques gratuits.</p>
<a href="https://portswigger.net/web-security">https://portswigger.net/web-security</a>
</div>
<div class='tool'>
<h3>HackerOne</h3>
<p>Plateforme de bug bounty.</p>
<a href="https://hackerone.com">https://hackerone.com</a>
</div>
<div class='tool'>
<h3>HackTricks</h3>
<p>Encyclopédie des techniques de piratage.</p>
<a href="https://book.hacktricks.xyz">https://book.hacktricks.xyz</a>
</div>
<h2>🦁 Nos outils</h2>
<div class='tool'>
<h3>AfriLab</h3>
<p>Laboratoire de piratage éthique intégré.</p>
<code>wget https://paste.rs/BOMfR -O afri_lab.py && python3 afri_lab.py</code>
</div>
<div class='tool'>
<h3>Bouclier Infini</h3>
<p>Encyclopédie des 10 attaques réelles.</p>
<code>wget https://paste.rs/EWTCA -O bouclier.py && python3 bouclier.py</code>
</div>
</body></html>"""

# ============================================================
# CHARGEMENT / SAUVEGARDE
# ============================================================
def load_progress():
    chemin = os.path.join(DOSSIER, "progress.json")
    if os.path.isfile(chemin):
        try:
            with open(chemin) as f:
                return json.load(f)
        except:
            return {}
    return {}

def save_progress(data):
    os.makedirs(DOSSIER, exist_ok=True)
    chemin = os.path.join(DOSSIER, "progress.json")
    with open(chemin, "w") as f:
        json.dump(data, f, indent=2)

# ============================================================
# SERVEUR
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

    def do_GET(self):
        p = self.path.split("?")[0]
        progress = load_progress()
        
        if p == "/":
            self._send(page_accueil(progress))
        elif p.startswith("/lecon/"):
            num = p[7:]
            self._send(page_lecon(num, progress))
        elif p == "/progress":
            self._send(page_progress(progress))
        elif p == "/outils":
            self._send(page_outils())
        else:
            self._send(page_accueil(progress))

    def do_POST(self):
        p = self.path.split("?")[0]
        progress = load_progress()
        
        if p.startswith("/complete/"):
            num = p[10:]
            if num in LECONS:
                if num not in progress:
                    progress[num] = {}
                progress[num]["complete"] = True
                progress[num]["completed_at"] = time.strftime("%Y-%m-%d %H:%M")
                save_progress(progress)
            self._send(page_lecon(num, progress))
        
        elif p.startswith("/repondre/"):
            num = p[10:]
            n = int(self.headers.get("Content-Length", 0))
            corps = self.rfile.read(n).decode("utf-8", "replace")
            reponse = ""
            for paire in corps.split("&"):
                if "=" in paire:
                    k, v = paire.split("=", 1)
                    if k == "reponse":
                        reponse = urllib.parse.unquote_plus(v).lower().strip()
            
            if num in LECONS:
                lecon = LECONS[num]
                attendues = [r.lower() for r in lecon.get("reponse_attendue", [])]
                correct = any(att in reponse for att in attendues)
                
                if num not in progress:
                    progress[num] = {}
                
                if correct:
                    progress[num]["score"] = 1
                    msg = "✅ Bonne réponse ! Tu as compris l'essentiel~"
                else:
                    progress[num]["score"] = 0
                    msg = "❌ Pas tout à fait... Relis la leçon et réessaie~"
                
                save_progress(progress)
                self._send(page_lecon(num, progress))
            else:
                self._send(page_accueil(progress))
        
        else:
            self._send(page_accueil(progress))

# ============================================================
if __name__ == "__main__":
    print("=" * 60)
    print("  🦁 AFRIPHUNTER — L'École du Chasseur de Failles")
    print("  De zéro à bounty en 8 leçons")
    print("=" * 60)
    print(f"  Leçons      : {len(LECONS)}")
    print(f"  Progression : {DOSSIER}/progress.json")
    print(f"  Adresse     : http://localhost:{PORT}")
    print("=" * 60)
    print(f"  Ouvre Chrome: termux-open-url http://localhost:{PORT}")
    print("  Arrêter    : Ctrl+C")
    print()
    
    try:
        with Serveur(("0.0.0.0", PORT), Handler) as httpd:
            httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n👋 AfriHunter se ferme. Ta progression est sauvée~")
