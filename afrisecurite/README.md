# 🛡️ AfriSécurité — Suite de Défense du Sahel

Par Koffi Christ Olivier & Letta-Chan — septembre 2026

Suite d'outils de recherche en sécurité et d'éducation à la défense.
100% Python std / HTML — aucune dépendance, marche offline sur Termux.

> ⚠️ Éthique : « le bouclier est plus lourd que l'épée » — ces outils servent
> à COMPRENDRE les attaques pour construire des défenses africaines.
> Jamais pour attaquer de vraies personnes ou systèmes.

## Les outils

| Outil | Fichier | Port | Description |
|---|---|---|---|
| 🌍 AfriVeille | afri-veille/afri_veille.py | 8091 | Surveillance du discours public (5 sources d'actualités) |
| 🛡️ Bouclier du Sahel | afri-bouclier/afri_bouclier.py | 8092 | Encyclopédie de défense contre la surveillance |
| 🛰️ AfriSat v1+v2 | afri-sat/afri_sat.py, afri_sat_v2.py | 8093 | Suivi de satellites réels (données NORAD Celestrak, 11 364 satellites) |
| 🧪 AfriLab | afri-lab/afri_lab.py | 8094 | Labo de hacking éthique : SQLi, XSS, contournement de login (victimes fictives) |
| ♾️ Bouclier Infini | afri-bouclier-infini/bouclier_infini.py | — | Encyclopédie des 10 attaques réelles avec détection + défense |
| 🎮 Le Lion du Sahel | afri-jeux/ | — | Jeu HTML5 (v1, embed, La Vraie Savane v2 avec cycle jour/nuit) |
| 📚 AfriNav | afri-nav/afri_nav.py | 8095 | Bibliothèque offline de dossiers + sauvegarde de pages web |
| 🎯 AfriHunter | afri-hunter/afri_hunter.py | 8096 | École de bug bounty : 8 leçons de zéro à ton premier bounty |
| 🏦 AfriCible | afri-cible/afri_cible.py | 8097 | Banque fictive vulnérable : 5 défis (IDOR, SQLi, XSS, JWT, Path Traversal) avec flags + progression |

## Déploiement rapide (Termux)

Chaque outil est un fichier unique Python std :

```bash
python3 afri-cible/afri_cible.py   # par exemple
```

Déploiement direct depuis paste.rs (sans git) :
- AfriSat: https://paste.rs/W1haB
- AfriLab: https://paste.rs/BOMfR
- Bouclier Infini: https://paste.rs/EWTCA
- Lion v1: https://paste.rs/YQMoU
- Lion v2 (La Vraie Savane): https://paste.rs/CZwVj
- AfriNav: https://paste.rs/mVaeb
- AfriHunter: https://paste.rs/ir6dA
- AfriCible: https://paste.rs/AVX94
