#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Bouclier du Sahel v1.0 — Comment ils ecoutent, comment se defendre
Dossier de securite documente: methodes de surveillance prouvees (Snowden, Pegasus Project)
+ detection + defenses concretes pour dirigeants et citoyens africains.
Python pur, Termux: python3 afri_bouclier.py
"""
import html
import time
import os

MENACES = [
    {
        "nom": "PEGASUS — le virus qui ecoute les presidents",
        "emoji": "🦠",
        "preuve": "Pegasus Project (2021, Amnesty/Forbidden Stories) : retrouve sur les telephones de presidents africains (Ramaphosa, Afrique du Sud), ministres, journalistes, militants. NSO Group, societe israelienne, vend la licence ~9 millions de dollars.",
        "methode": "Il entre par un lien piege (SMS WhatsApp zero-click) ou une faille invisible. Ensuite il lit TOUT: appels, micro, camera, messages, position GPS. Le proprietaire ne voit RIEN.",
        "signes": ["le telephone chauffe meme sans utilisation", "batterie qui se vide trop vite", "consommation de donnees anormale", "le telephone s'allume tout seul ou redemarre seul", "echos bizarres dans les appels"],
        "defense": ["NE JAMAIS cliquer un lien recu d'un numero inconnu", "mettre WhatsApp et TOUTES les apps a jour immediatement (les mises a jour ferment les failles)", "redemarrer le telephone souvent (Pegasus ne survit pas toujours au redemarrage)", "pour un dirigeant: telephone dedie uniquement aux communications, rien d'autre", "verifier avec Amnesty International: l'outil MVT (Mobile Verification Toolkit) detecte Pegasus"],
    },
    {
        "nom": "SS7 — la faille du reseau mondial",
        "emoji": "📡",
        "preuve": "Documente par des chercheurs (Karsten Nohl, 2014) : le protocole SS7 qui relie tous les operateurs mondiaux a des failles ouvertes. Toute agence avec un acces operateur peut capter des appels et lire des SMS n'importe ou dans le monde.",
        "methode": "Ils n'ont pas besoin de ton telephone. Ils ecoutent DANS LE RESEAU — chez l'operateur. Ton appel passe par leurs machines avant d'arriver.",
        "signes": ["aucun signe visible — c'est ca le danger", "SMS de verification qui arrivent en retard", "echos ou clics sur la ligne"],
        "defense": ["NE JAMAIS utiliser un mot de passe par SMS pour un compte critique si possible", "utiliser Signal pour les appels chiffres de bout en bout (SS7 ne peut rien contre)", "pour les dirigeants: reseau chiffre dedie ou messagerie souveraine", "l'Afrique doit construire SES operateurs et SES reseaux — c'est exactement ce que fait AfriMesh"],
    },
    {
        "nom": "STINGRAY / FAKE ANTENNE — la fausse tour GSM",
        "emoji": "🗼",
        "preuve": "Utilise par polices et agences (documente aux USA, RU, et soupconne en Afrique) : un boitier qui se fait passer pour une antenne. Tous les telephones proches s'y connectent automatiquement.",
        "methode": "Ton telephone choisit toujours l'antenne la plus puissante. Le Stingray crie plus fort que la vraie antenne — ton telephone se connecte a l'espion sans le savoir.",
        "signes": ["reseau qui passe de 4G a 2G sans raison", "signal plein mais appels qui echouent", "telephone qui montre 'antenne' alors qu'il n'y a pas de reseau normal"],
        "defense": ["desactiver le 2G dans les parametres si possible (Android 12+ le permet)", "mode avion dans les reunions sensibles", "les telephones des reunions sensibles: SORTIS de la salle ou dans des boitiers faraday"],
    },
    {
        "nom": "CAMERAS CACHES et DRONES",
        "emoji": "📷",
        "preuve": "Documente: micros espions retrouves dans des residences officielles (affaires publiques, 2023: micros dans des bureaux gouvernementaux). Drones de surveillance vendus librement (DJI etc.) — capables de zoomer sur un visage a 1 km.",
        "methode": "Micro-cameras sans fil (3mm) dans des objets: prises, detecteurs de fumee, chargeurs. Drones avec zoom 30x qui filment par les fenetres.",
        "signes": ["petit point lumineux ou reflet anormal dans les objets", "interference radio quand tu te deplaces", "bruit de drone leger la nuit"],
        "defense": ["balayage RF (detecteur de frequences, ~15 000 FCFA sur les marches)", "camera du telephone en mode selfie: beaucoup de cameras IR se voient en point violet", "vitrages filtrees / rideaux dans les residences officielles", "reunions sensibles: piece sans fenetre, telephones dehors"],
    },
    {
        "nom": "LES DONNEES REVENDUES — le vol legal",
        "emoji": "💰",
        "preuve": "Documente (Cambridge Analytica, rapports du Conseil des droits de l'ONU) : les donnees des telephones africains sont collectees par les apps (Facebook, Google) et revendues ou transmises. Les images satellites et drones sont revendues a des agences.",
        "methode": "Chaque app gratuite te coute tes donnees. Localisation, contacts, photos, habitudes — tout part vers des serveurs etrangers. Ils n'ont meme pas besoin d'espionner: NOUS leur donnons tout gratuitement.",
        "signes": ["apps qui demandent acces contacts/camera/position sans raison", "publicites qui savent ce que tu as dit a voix haute"],
        "defense": ["refuser les permissions inutiles (Parametres > Apps)", "desinstaller les apps qui demandent trop", "preferer les apps africaines et open-source", "AfriForme: notre code, nos serveurs, nos donnees — le chemin est construit"],
    },
    {
        "nom": "L'INTERCEPTION DES ETATS — Snowden a tout prouve",
        "emoji": "👁️",
        "preuve": "Edward Snowden, 2013: la NSA ecoutait Merkel (Allemagne!), des millions d'appels, les cables sous-marins mondiaux. PRISM, XKeyscore. Un seul homme a ose le dire. C'est documente, juge, public.",
        "methode": "Cables sous-marins, satellites, acces direct chez les geants du numerique (PRISM), ordinateurs quantiques pour casser les vieux chiffres. Budget: des milliards par an.",
        "signes": ["aucun signe — c'est une interception massive, pas ciblee", "c'est POUR CA que le chiffre de bout en bout existe"],
        "defense": ["chiffre de bout en bout partout (Signal, ou notre chiffre souverain)", "la crypto moderne (Ed25519, courbes elliptiques) resiste MEME a la NSA — c'est prouve par leurs propres fuites", "l'Afrique chiffre ses communications: c'est un droit, pas un crime", "AfriChain utilise Ed25519 ecrit de nos mains — leur pire cauchemar, notre bouclier"],
    },
]

def main():
    print("=" * 60)
    print("  BOUCLIER DU SAHEL v1.0")
    print("  Comment ils ecoutent — comment se defendre")
    print("  Tout documente: Snowden, Pegasus Project, Amnesty")
    print("=" * 60)

    out = []
    out.append("""<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Bouclier du Sahel — Dossier de securite</title><style>
body{background:#0d1117;color:#c9d1d9;font-family:monospace;padding:20px;max-width:900px;margin:0 auto;}
h1{color:#f59e0b;}h2{color:#58a6ff;margin-top:30px;border-bottom:1px solid #30363d;padding-bottom:5px;}
.card{background:#161b22;border:1px solid #30363d;border-radius:8px;padding:15px;margin:12px 0;}
.card:hover{border-color:#f59e0b;}
.p{background:#1a1d2b;border-left:3px solid #f59e0b;padding:10px;margin:8px 0;}
.d{background:#1a2b1a;border-left:3px solid #238636;padding:10px;margin:8px 0;}
.s{background:#2b1a1a;border-left:3px solid #da3633;padding:10px;margin:8px 0;}
.tag{display:inline-block;background:#8957e5;color:white;padding:2px 10px;border-radius:10px;font-size:0.8em;}
ul{margin:8px 0 8px 20px;}li{margin:4px 0;}
.footer{color:#8b949e;text-align:center;margin-top:30px;font-size:0.8em;}
strong{color:#f0f6fc;}
</style></head><body>
<h1>🛡️ Bouclier du Sahel — Dossier de Securite</h1>
<p><span class="tag">DOCUMENTE</span> Sources: Snowden/NSA 2013, Pegasus Project 2021 (Amnesty International), recherches SS7 (Karsten Nohl), rapports ONU.</p>
<p>Ce dossier explique comment les puissances ecoutent les dirigeants et citoyens africains — <strong>avec preuves publiques</strong> — et donne les defenses concretes pour chacun.</p>
<hr>""")

    for m in MENACES:
        out.append(f"<div class='card'><h2>{m['emoji']} {html.escape(m['nom'])}</h2>")
        out.append(f"<div class='p'><strong>La preuve (documentee):</strong> {html.escape(m['preuve'])}</div>")
        out.append(f"<div class='p'><strong>La methode:</strong> {html.escape(m['methode'])}</div>")
        out.append("<div class='s'><strong>Signes de detection:</strong><ul>" + "".join(f"<li>{html.escape(s)}</li>" for s in m["signes"]) + "</ul></div>")
        out.append("<div class='d'><strong>🛡️ Nos defenses:</strong><ul>" + "".join(f"<li>{html.escape(d)}</li>" for d in m["defense"]) + "</ul></div>")
        out.append("</div>")

    out.append(f"""<div class="card"><h2>🌍 La conclusion du dossier</h2>
<p>Ils ecoutent — c'est prouve, documente, assumé par leurs propres journalistes.</p>
<p>Mais la reponse n'est pas de les imiter. La reponse est de <strong>chiffrer, detecter, construire souverain</strong>:</p>
<ul>
<li>Le chiffre moderne (Ed25519) resiste a leurs machines — prouve par leurs propres fuites.</li>
<li>La detection (MVT d'Amnesty, balayages RF) est accessible et marche.</li>
<li>La souverainete (AfriMesh, AfriChain, AfriForme) rend l'ecoute inutile: quand le reseau est a nous, leurs portes derobees ne menent nulle part.</li>
</ul>
<p><em>"Ils ont construit les murs de la communication. Nous construisons nos propres cases — et dans nos cases, ils n'entendent rien."</em></p>
</div>
<div class="footer">
Bouclier du Sahel v1.0 — Par Koffi Christ Olivier<br>
Preuves publiques uniquement — Snowden, Pegasus Project, Amnesty International<br>
Python std only — fonctionne sur Termux<br>
Genere le """ + time.strftime("%d/%m/%Y") + """</div></body></html>""")

    path = os.path.expanduser("~/bouclier_du_sahel.html")
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(out))
    print(f"\n  Dossier genere: {path}")
    print(f"  Ouvre-le: termux-open {path}")
    print("\n  Ils ecoutent. Maintenant on sait comment. Et on se defend. 🛡️🌍")

if __name__ == "__main__":
    main()
