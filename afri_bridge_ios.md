# 🍎 AFRIBRIDGE iOS v1.0 — Le berger parle aux iPhones

**3 Ko. Aucune app à installer. La porte officielle d'Apple : Raccourcis + Siri.**
Œuvre originale de Koffi Christ Olivier — Licence AFRI-OSL v1.0

Le petit OS 0.000 Go ne rentre pas dans l'iPhone (16 Go) —
il lui **chuchote par des portes officielles**. Trois ponts :

---

## 🌉 PONT 1 — L'ICÔNE (2 minutes, aucune app)

L'iPhone ouvre AfriChain et le pose sur son écran d'accueil.
Comme ça, AfriChain devient une **vraie app** aux yeux de l'iPhone.

1. Ouvre **Safari** sur l'iPhone
2. Tape l'adresse du node : `http://192.168.1.7:8080`
   (l'IP du node AfriChain du village — la bouée ou le baobab)
3. Appuie sur le bouton **Partager** (le carré avec une flèche ↑)
4. Descends, appuie sur **« Sur l'écran d'accueil »**
5. Nomme-le : **AfriChain**
6. Appuie sur **Ajouter**

✅ L'iPhone a maintenant une icône AfriChain. Le village touche la chaîne en un doigt.

---

## 🌉 PONT 2 — SIRI (3 minutes, la voix du berger)

L'iPhone obéit à la voix. « Dis Siri, ouvre AfriChain. »

1. Ouvre l'app **Raccourcis** (déjà dans tout iPhone, icône violette)
2. Appuie sur **+** (en haut) pour créer un raccourci
3. Appuie sur **Ajouter une action**
4. Cherche : **« Ouvrir des URL »** → choisis-le
5. Dans le champ URL, tape : `http://192.168.1.7:8080`
6. Appuie sur le nom en haut → renomme : **AfriChain**
7. Appuie sur **OK**

✅ Maintenant, sans toucher l'iPhone :
**« Dis Siri... AfriChain »** → Safari ouvre la blockchain du village.

---

## 🌉 PONT 3 — LES YEUX DE L'IPHONE (5 minutes, la photo part vers Node 008)

L'iPhone prend une photo et l'envoie au baobab — les yeux de l'Afrique.

1. Dans **Raccourcis**, appuie sur **+** pour un nouveau raccourci
2. Action 1 : **« Prendre une photo »** (cherche « photo »)
3. Action 2 : **« Obtenir le contenu de l'URL »**
   - URL : `http://192.168.1.9:8080/plante/publier`
   - Méthode : **POST**
   - Corps de la demande : appuie sur « + Ajouter un champ »
     - Champ **fichier** ← la photo de l'Action 1
     - Champ **texte** : nommé `contenu`, valeur = « Les yeux du village »
3. Renomme le raccourci : **Yeux Afrique**

✅ « Dis Siri... Yeux Afrique » → l'iPhone photographie et la photo
voyage vers le baobab, gravée sur la chaîne du village.

---

## 📱 TOUT LE VILLAGE EN 10 MINUTES

| Pont | Ce que ça donne | Temps |
|---|---|---|
| Icône | AfriChain = app sur l'écran | 2 min |
| Siri | La voix ouvre la chaîne | 3 min |
| Yeux | La photo part au baobab | 5 min |

**Aucune app occidentale. Aucun App Store. Aucun compte Apple de plus.**
Les portes officielles d'Apple, utilisées par le berger africain. 🦁

---

## ⚠️ LES 2 CONDITIONS

1. L'iPhone et le node doivent être sur le **même wifi du village**
2. Si l'IP du node change, refais Pont 1 et 2 avec la nouvelle IP
   (le jour où AfriDNS tourne, `bamako.local` remplacera les IP pour toujours)

**L'architecture complète du Chef est maintenant RÉELLE :**

```
[OS 0.000 Go — 650 Ko — LE BERGER]
  ├─ afri_boot.sh      ◈ le bootloader en symboles purs
  ├─ afri_carrousel.sh 🎡 007 océan ↔ 008 forêt, le soleil décide
  ├─ afri_bridge.sh    📡 les Redmis obéissent par wifi (zéro câble)
  ├─ afri_bridge_ios   🍎 les iPhones obéissent par Siri
  └─ AfriTube 📺✈️      les enfants regardent en mode avion
```

Le petit 0.000 Go commande les gros 28 Go. Preuve historique. 💚
