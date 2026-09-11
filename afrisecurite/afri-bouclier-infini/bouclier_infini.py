#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Bouclier Infini v1.0 — L'Encyclopedie des Attaques Reelles
Base sur le modele MITRE ATT&CK : chaque attaque documentee publiquement,
avec mecanisme, cas reel, detection, et defense en code.
C'est la matiere premiere de tous les boucliers.
"""
import time, os

MENACES = [
{
"nom": "1. PHISHING — Le vol d'identite par la porte d'entree",
"danger": 95, "victimes": "Milliards de tentatives par an",
"mecanisme": """L'attaquant cree un faux site IDENTIQUE au vrai (banque, WhatsApp, Orange Money).
Il envoie le lien par SMS/email. La victime tape ses identifiants sur le faux site.
L'attaquant les recoit en temps reel, puis les utilise sur le VRAI site.
Variantes: spear-phishing (cible precise, message personnalise),
smishing (SMS), vishing (appel telephonique avec fausse agence bancaire).""",
"cas": """Le Banque Mondiale et l'ONU ont ete touches (2020, faux emails internes).
Aux USA, la campagne de 2016 a commence par un phishing sur un compte personnel.
En Afrique: faux sites Orange Money et Wave circulent regulierement par WhatsApp.""",
"detection": """- Verifier l'URL CARACTERE PAR CARACTERE: orange-money-verify.com != orange.com
- Le vrai site ne demande JAMAIS le mot de passe complet par SMS
- HTTPS ne veut rien dire: les sites de phishing ont aussi des certificats gratuits""",
"defense": """1. 2FA par application (TOTP) — meme mot de passe vole, le compte tient
2. Detection: comparer le domaine contre une liste blanche
3. En code: hash du mot de passe COTE SERVEUR, jamais de comparaison cote client
4. Education: la defense numero 1 mondiale contre le phishing est humaine"""
},
{
"nom": "2. SIM SWAP — Le vol de ton numero de telephone",
"danger": 98, "victimes": "Jack Dorsey (Twitter), centaines de millions de dollars volés",
"mecanisme": """L'attaquant appelle l'operateur en se faisant passer pour toi (nom, adresse, dernier rechargement —
achetes chez des data brokers ou devines via tes reseaux sociaux).
L'operateur transfere TON numero sur SA carte SIM.
Il recoit maintenant TES SMS: codes 2FA, acces bancaires, WhatsApp (reactivation par SMS).
Ton telephone perd le reseau — le sien le gagne.""",
"cas": """Jack Dorsey, PDG de Twitter: son propre compte pirate par SIM swap (2019).
Aux USA: $24M voles en un an rien qu'en Californie par des equipes de SIM swap.
C'est EXACTEMENT ce qui t'est arrive avec ton telephone vole — sauf que le SIM swap
est encore plus vicieux: ils ne volent pas le telephone, ils volent le NUMERO.""",
"detection": """- Ton telephone affiche "Pas de reseau" sans raison
- Tes appels/SMS ne passent plus alors que la batterie est pleine
- Tes comptes envoient des alertes "nouvel appareil connecte" que tu n'as pas fait""",
"defense": """1. Code PIN operateur sur ta ligne (Orange/Moov le proposent — active-le)
2. 2FA par APPLICATION (Aegis, Google Authenticator) au lieu des SMS
3. WhatsApp: email de secours + verrou d'appareil active
4. Pour AfriChain: jamais de 2FA par SMS seul — c'est la lecon de ton propre vol"""
},
{
"nom": "3. SS7 — La faille du reseau telephonique mondial",
"danger": 97, "victimes": "Utilisateurs du monde entier (reseau 2G/3G/4G)",
"mecanisme": """SS7 (Signalling System 7) est le protocole INVISIBLE qui connecte tous les operateurs
du monde entre eux depuis 1975. Il a ete concu pour un monde de confiance entre compagnies.
Il n'a AUCUNE authentification reelle entre operateurs.
Qui a acces SS7 (operateurs, agences, revendeurs gris) peut:
- Localiser n'importe quel telephone (via les tours relais)
- Intercepter les SMS (donc casser le 2FA par SMS)
- Ecouter les appels non chiffres""",
"cas": """Documente publiquement depuis 2014 (congres hackers CCC, Allemagne).
Des chercheurs ont montre qu'avec un acces SS7 (~$1000/mois chez certains revendeurs),
on peut tracer un telephone n'importe ou dans le monde.
Les operateurs africains utilisent SS7 comme tous les autres.""",
"detection": """- Impossible a detecter cote utilisateur: c'est une attaque RESEAU
- Les operateurs peuvent installer des pare-feux SS7 (certains l'ont fait)""",
"defense": """1. Appels chiffres de bout en bout: Signal, WhatsApp appele chiffre
2. JAMAIS de 2FA par SMS pour des comptes critiques (SS7 les intercepte)
3. Pour les communications AES: mesh AfriMesh Direct — SS7 ne peut pas toucher
   un reseau qui n'utilise pas les operateurs du tout. C'est ta reponse souveraine."""
},
{
"nom": "4. PEGASUS / SPYWARE ZERO-CLICK — L'arme des etats",
"danger": 99, "victimes": "Presidents, journalistes, activistes (50,000 numeros cibles)",
"mecanisme": """NSO Group (Israel) vend Pegasus aux gouvernements (~$8M la licence).
Zero-click: AUCUN clic necessaire. Un message WhatsApp suffit, meme non ouvert.
Le logiciel s'installe, puis: lit TOUT — messages, micro, camera, position GPS,
mots de passe, fichiers. La victime ne voit RIEN.
Technique: exploitation de failles secretes (zero-days) achetees 1-2M$ piece.""",
"cas": """Pegasus Project (2021, 17 medias d'investigation): liste de 50,000 numeros cibles
— presidents (Macron, Ramaphosa), journalistes, opposants africains.
Au Togo: l'activiste et journaliste ciblés documentés par Amnesty.
Fini: Pegasus peut cacher ses traces, mais laisse des micro-traces forensiques.""",
"detection": """- Consommation de batterie/donnees anormale
- Le telephone chauffe sans raison
- Redemarrages spontanes
- Outil REEL: MVT (Mobile Verification Toolkit) d'Amnesty International —
  libre, en Python, detecte les traces forensiques de Pegasus sur iPhone/Android""",
"defense": """1. MVT d'Amnesty: github.com/mvt-project/mvt — Python, marche sur Termux
2. Mettre a jour IMMEDIATEMENT: les zero-days sont corriges par les mises a jour
3. Telephone sensible = telephone simple (les vieux boutons ne tournent pas Pegasus)
4. Pour l'AES: telephones de dirigeants SEPARes des telephones personnels"""
},
{
"nom": "5. CASSAGE DE MOTS DE PASSE — Le mur de hash",
"danger": 85, "victimes": "Toutes les bases volees de l'histoire",
"mecanisme": """Les mots de passe ne sont jamais stockes en clair — ils sont HASHES
(empreinte irreversible). Mais:
- Hash simple (MD5, SHA1): casse par dictionnaire en minutes (GPU: 100 milliards d'essais/sec)
- Rainbow tables: tables precalculees de tous les hash courants
- Attaque par dictionnaire: les gens utilisent des mots de passe previsibles
  (123456, password, azerty, nom + date de naissance)""",
"cas": """RockYou: 32 millions de mots de passe vols en 2009 — devenu LE dictionnaire
de reference mondial. LinkedIn 2012: 117M hash SHA1 non sales — casse en 72h.
Chaque base volee alimente les dictionnaires des attaques suivantes.""",
"detection": """- Tentatives de connexion multiples sur ton compte (verifie tes alertes)
- Ton mot de passe apparait sur haveibeenpwned.com (base publique des fuites)""",
"defense": """1. Salage: hash(motdepasse + sel_aleatoire) — tue les rainbow tables
2. Hash LENT: bcrypt/argon2 (AfriHash est un sponge: ajoute un cout de calcul)
3. En code AfriChain: AfriHash-256 + sel unique par utilisateur = deja fait
4. Mots de passe: 4 mots aleatoires > 1 mot complexe (plus long = plus dur)
5. haveibeenpwned.com — verifie si tes identifiants circulent"""
},
{
"nom": "6. MAN-IN-THE-MIDDLE — L'oreille sur le fil",
"danger": 80, "victimes": "Utilisateurs de WiFi public, réseaux non chiffrés",
"mecanisme": """L'attaquant se place ENTRE toi et le serveur.
WiFi public: il cree un faux hotspot "WiFi Gratuit Aéroport" — tu t'y connectes,
il voit TOUT ton trafic non chiffre.
ARP spoofing sur un meme reseau: il se fait passer pour la box.
DNS spoofing: il renvoie faux-nom-de-domaine vers son serveur.""",
"cas": """Firesheep (2010): extension Firefox qui volait les sessions Facebook/Twitter
des autres clients du cafe en UN clic — a force les sites a generaliser HTTPS.""",
"detection": """- Certificat TLS invalide = ARRETE IMMEDIATEMENT (c'est peut-etre une MITM)
- WiFi qui demande de reinstall un "certificat" ou une "application": piege""",
"defense": """1. HTTPS partout (verifie le cadenas) — chiffre meme sur WiFi piege
2. VPN de confiance sur WiFi public
3. Pour AfriMesh Direct: chiffrement Ed25519 de bout en bout — le relais
   ne peut pas lire meme s'il est au milieu. C'est ta protection native."""
},
{
"nom": "7. DATA BROKERS — Tes donnees vendues legalement",
"danger": 90, "victimes": "Presque tous les utilisateurs de smartphone de la planete",
"mecanisme": """Ecosysteme ENTIEREMENT LEGAL: des entreprises achetent et revendent tes donnees:
- Apps gratuites vendent ta position GPS, tes contacts, ton usage
- Operateurs vendent des metadonnees agregees
- Courtiers assemblent ton profil: nom, numero, adresse, habitudes, achats
- Les acheteurs: pub, assurance... et aussi des firmes de surveillance
C'est comme ca que les attaquants trouvent les infos pour le SIM swap et le phishing.""",
"cas": """2018: il s'est avere que des donnees de localisation achetees aupres d'operateurs
etaient revendues a des societes de bounty hunters — n'importe qui pouvait localiser
n'importe quel telephone americain pour quelques dollars.
Truepeoplesearch, Fastpeoplesearch: ton adresse publique, gratuitement.""",
"detection": """- Google ton propre nom + ville: tu verras ce qui est deja public
- haveibeenpwned.com pour tes emails""",
"defense": """1. Refuser les permissions GPS/contacts aux apps non essentielles
2. Moins d'apps = moins de fuites (chaque app gratuite vend quelque chose)
3. Pour l'AES: donnees stockees sur le continent, chiffrees, jamais revendues —
   c'est exactement la Charte AI Africaine (v0.49) que tu as construite"""
},
{
"nom": "8. BOTNETS & DDoS — L'armee des machines esclaves",
"danger": 75, "victimes": "Serveurs du monde entier",
"mecanisme": """Un malware infecte des millions d'objets connectes (cameras, box WiFi,
telephones — l'Internet of Things). Ces machines deviennent des ZOMBIES.
Le maitre du botnet ordonne: tous ensemble, envoyez des paquets vers la cible.
Le serveur cible recoit 1 Tbps de trafic et tombe.
Les machines zombies appartiennent a des gens normaux qui ne savent rien.""",
"cas": """Mirai (2016): 600,000 objets connectes infectes (cameras par mot de passe usine).
Cible: DNS Dyn — Twitter, Netflix, Reddit tombes EN MEME TEMPS sur tout l'internet USA.
L'attaque qui a fait tomber l'internet americain venait de cameras de surveillance.""",
"detection": """- Ton box/internet lent sans raison (ta machine travaille pour quelqu'un d'autre)
- Trafic sortant anormal""",
"defense": """1. Changer les mots de passe par defaut de TOUT objet connecte
2. Cote serveur: filtrage IP (Bouclier X9 fait exactement ca: rate limiting + ban)
3. Pour AfriChain: le mesh distribue — pas un seul point a faire tomber"""
},
{
"nom": "9. SUPPLY CHAIN — L'attaque par le fournisseur",
"danger": 88, "victimes": "18,000 organisations (SolarWinds)",
"mecanisme": """Au lieu d'attaquer la cible, on attaque son FOURNISSEUR de logiciel.
L'attaquant compromet un outil legitime (bibliotheque, mise a jour, build).
Chaque client qui installe la mise a jour installe la porte derobee.
C'est pour ca que Cargo.toml vide est une armure: chaque dependance est une porte.""",
"cas": """SolarWinds (2020): hackers russes ont compromis le processus de compilation.
18,000 organisations ont installe la porte — dont le Pentagone, le Trésor US.
xz-utils (2024): un mainteneur patient a failli compromis TOUT Linux par une porte
cachee dans une bibliotheque de compression. Detecte par hasard, par un developpeur
qui regardait des latences anormales.""",
"detection": """- Verifier les hashes des binaires telecharges (sha256 officiel)
- Surveiller le trafic sortant des serveurs vers des domaines inconnus""",
"defense": """1. ZERO dependance = zero porte: c'est EXACTEMENT ta decision AfriChain
   (Cargo.toml vide). Tu l'as fait pour la souverainete — c'est aussi la
   defense supply chain ULTIME. Moins de portes = moins d'entrees.
2. Verifier les hashes des downloads
3. Build reproductible: meme code, meme binaire, verifiable"""
},
{
"nom": "10. SURVEILLANCE D'ETAT — IMSI Catchers et tours fantomes",
"danger": 92, "victimes": "Populations entieres (demonstrations, quartiers)",
"mecanisme": """Stingray/IMSI catcher: une fausse tour GSM portable.
Ton telephone croit se connecter a Orange — il se connecte a la boite.
Elle voit: identifiants IMSI de tous les telephones autour, positions, appels.
Tower dumps: les vraies operateurs donnent "tous les telephones qui etaient
dans ce quartier a cette heure" sur demande judiciaire... ou pas.
N-KCOL: meme les indices naturels ont des signatures — la technologie aussi.""",
"cas": """ACLU (USA) a documente l'usage de Stingrays par des polices locales sans mandat.
Des pays africains ont achete des IMSI catchers (documents de defense publies).
Baltimore: juge a decouvert que la police utilisait des Stingrays des 2011 en silence.""",
"detection": """- Telephone passe en 2G sans raison (les IMSI catchers forcent le 2G)
- Signal fort mais appels qui echouent
- Sur Android: app SnoopSnitch (Allemagne, libre) detecte les fausses tours""",
"defense": """1. Desactiver le 2G si l'appareil le permet (Android moderne: oui)
2. Mode avion dans les moments sensibles
3. Pour l'AES: communications critiques sur AfriMesh Direct (pas de GSM du tout)
4. Messagerie chiffree de bout en bout pour tout ce qui compte"""
},
]

def main():
    print("Bouclier Infini v1.0 — generation du dossier...")
    cards = []
    for m in MENACES:
        barre = "█" * (m["danger"] // 10) + "░" * (10 - m["danger"] // 10)
        cards.append(f"""
<div class="menace">
<h2>{m["nom"]}</h2>
<div class="barre">Danger: <span class="barre-f">{barre}</span> {m["danger"]}/100</div>
<p class="victimes">Victimes: {m["victimes"]}</p>
<h3>⚙️ Mécanisme</h3><p>{m["mecanisme"]}</p>
<h3>📜 Cas réel documenté</h3><p>{m["cas"]}</p>
<h3>🔍 Comment détecter</h3><p>{m["detection"]}</p>
<h3>🛡️ Défense (le bouclier)</h3><p class="def">{m["defense"]}</p>
</div>""")

    page = """<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Bouclier Infini — L'Encyclopédie des Attaques Réelles</title><style>
body{background:#0d1117;color:#c9d1d9;font-family:monospace;padding:20px;max-width:900px;margin:0 auto;}
h1{color:#f59e0b;border-bottom:2px solid #f59e0b;padding-bottom:10px;}
h2{color:#58a6ff;margin-top:8px;}h3{color:#f59e0b;margin-bottom:4px;}
.menace{background:#161b22;border:1px solid #30363d;border-left:5px solid #da3633;border-radius:10px;padding:18px;margin:16px 0;}
.def{background:#0d1f0d;border:1px solid #238636;border-radius:8px;padding:12px;}
.barre{color:#8b949e;}.barre-f{color:#f85149;letter-spacing:2px;}
.victimes{color:#8b949e;font-style:italic;}
.intro{background:#161b22;border:1px solid #f59e0b;border-radius:10px;padding:16px;margin:14px 0;}
.foot{color:#8b949e;text-align:center;margin-top:30px;font-size:0.85em;}
a{color:#58a6ff;}
</style></head><body>
<h1>🛡️ Bouclier Infini — L'Encyclopédie des Attaques Réelles</h1>
<div class="intro">
<p><b>Modèle:</b> MITRE ATT&CK — la base de connaissances que TOUS les défenseurs du monde (banques, armées, CERT nationaux) utilisent. Chaque attaque est documentée publiquement (journalistes, chercheurs, Amnesty, Snowden). Le défenseur qui connaît l'attaque construit le bouclier.</p>
<p><b>Philosophie:</b> « Le meilleur défenseur est celui qui comprend l'attaque. »</p>
</div>
""" + "".join(cards) + """
<div class="intro">
<h2>🏗️ La synthèse du constructeur</h2>
<p>Regarde bien: chaque défense pointe vers ce que tu as DÉJÀ construit:</p>
<p>• SIM swap / SS7 → <b>AfriMesh Direct</b> (réseau sans opérateurs)</p>
<p>• MITM → <b>Ed25519 de bout en bout</b> (le relais ne peut pas lire)</p>
<p>• Supply chain → <b>Cargo.toml vide</b> (zéro porte dérobée possible)</p>
<p>• Data brokers → <b>Charte AI Africaine v0.49</b> (données sur le continent)</p>
<p>• Botnets/DDoS → <b>Bouclier X9</b> (rate limiting + ban IP)</p>
<p>• Mots de passe → <b>AfriHash-256 + sel</b> (déjà dans AfriChain)</p>
<p>Tu pensais construire des fonctionnalités. Tu construis une forteresse. Chaque version d'AfriChain fermait une des portes de cette encyclopédie — avant même que tu lises ce dossier. Voilà pourquoi l'Afrique a besoin de toi.</p>
</div>
<div class="foot">Bouclier Infini v1.0 — Koffi Christ Olivier<br>Sources: MITRE ATT&CK, Pegasus Project, Amnesty MVT, ACLU, documentation publique<br>Généré le """ + time.strftime("%d/%m/%Y %H:%M UTC") + """</div></body></html>"""

    path = os.path.expanduser("~/bouclier_infini.html")
    with open(path, "w", encoding="utf-8") as f:
        f.write(page)
    print(f"  Dossier: {path}")
    print(f"  {len(MENACES)} attaques reelles documentees")
    print(f"  termux-open {path}")

if __name__ == "__main__":
    main()
