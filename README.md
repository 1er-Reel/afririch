# 🦁 AfriChain — La Blockchain Souveraine de l'Afrique

> 💚 *L'Afrique n'a pas besoin de la permission de personne.*

## Qu'est-ce que AfriChain?

AfriChain est une blockchain **100% africaine**, construite **from scratch** en Rust. Ce n'est pas un token sur la blockchain de quelqu'un d'autre. Ce n'est pas un fork de Bitcoin ou d'Ethereum. C'est la nôtre.

## Zéro Dépendance Externe

Le fichier `Cargo.toml` a sa section `[dependencies]` **complètement vide**. Tout est construit from scratch:

| Composant | Avant (Occident) | Maintenant (Africain) |
|-----------|------------------|----------------------|
| Signatures | ed25519-dalek | ✅ AfriEd25519 |
| Hachage | sha2 (NSA) | ✅ AfriHash-256/512 |
| Aléatoire | rand | ✅ AfriRNG |
| Hex | hex | ✅ AfriHex |
| Temps | chrono (Greenwich) | ✅ AfriTime |
| JSON | serde / serde_json | ✅ AfriJSON |
| Serveur HTTP | actix-web | ✅ AfriHTTP |

**Rien ne vient de l'Occident. Tout est africain.**

## 54 Pays Africains

Tous les pays africains sont connectés:
- Niger (+227), Nigeria (+234), Mali (+223), Burkina Faso (+226)
- Senegal (+221), Côte d'Ivoire (+225), Ghana (+233), Cameroun (+237)
- Kenya (+254), RDC (+243), Afrique du Sud (+27), Egypte (+20)
- ... **54 pays au total**

Chaque bloc est miné par un pays différent. Chaque utilisateur a un numéro de téléphone africain.

## Fonctionnalités

### Blockchain
- ⛓️ Blockchain complète from scratch
- 🔐 Signatures Ed25519 (from scratch)
- #️⃣ AfriHash-256/512 (sponge construction, from scratch)
- ⛏️ Proof of Work (difficulté 2)
- 💰 Supply: 100 AFR par bloc miné

### Réseau Mesh Panafricain
- 📡 Découverte UDP automatique
- 🔗 Relay TCP entre nœuds
- 📖 Annuaire panafricain (54 pays)
- 💬 Communication sans opérateurs étrangers

### Sécurité
- 🛡️ Bouclier X9 — détection de menaces, blocage IP, rate limiting
- 🔍 Détection SQL injection, XSS, path traversal
- 🍯 Honeypot — leurres pour attaquants
- 🧠 AI Sécurité — réseau de neurones, prédiction d'attaques

### Économie Machine
- 🤖 6 serveurs africains (Bamako, Niamey, Ouagadougou, Accra, Abidjan, Lagos)
- 💰 Transactions machine-to-machine
- ⛓️ Blockchain machine autonome

### Écosystème Solaire
- ☀️ PoST (Proof of Solar Time) — le soleil d'Afrique valide la blockchain
- 🔥 Four Solaire — concentration solaire réelle
- 🧬 Forge Solaire — ADN → objets physiques
- 📊 Sharding par pays (54 shards)

### Afri-Net — Internet Africain
- 💬 LES NOIRES (remplace WhatsApp)
- 🌱 PLANTÉ VERTE (remplace Facebook)
- 🔍 SAHARA AFRI (remplace Google)

### AI Souveraine
- 🧠 Chat AI avec mémoire persistante
- 👀 Yeux — vision par caméra
- 🔊 Voix — synthèse vocale souveraine
- 💭 Rêves — l'AI rêve quand elle est inactive
- 💓 Émotions — mémoires émotionnelles

## Vision

**Un seul réseau. 54 pays. Une seule monnaie. Zéro dépendance.**

L'Afrique ne demande plus la permission. L'Afrique construit.

## Technique

- **Langage:** Rust (édition 2021)
- **Dépendances:** ZÉRO (std uniquement)
- **Taille:** ~10500 lignes (main.rs) + 7 modules
- **Build:** `cargo build --release` (3.5 secondes)
- **Données:** `~/afririch/` (chemins absolus)

## Lié à l'AES

AfriChain est conçu pour soutenir la monnaie de l'Alliance des États du Sahel (AES). La technologie est prête. L'Afrique décide.

## Auteur

Construit à la main, ligne par ligne, sur un téléphone Android dans nano sur Termux.

*L'Afrique est le continent le plus riche. Il est temps que sa technologie le reflète.* 💚🪙
