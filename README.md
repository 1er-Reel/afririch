# 🦁 AfriChain — Banque Numérique Souveraine de l'Afrique

> 💚 *L'Afrique ne demande plus la permission.*

## Qu'est-ce que AfriChain?

AfriChain est une blockchain **100% africaine**, construite **from scratch** en Rust. Ce n'est pas un token sur la blockchain de quelqu'un d'autre. Ce n'est pas un fork de Bitcoin ou d'Ethereum. C'est la nôtre.

AfriChain est la **Banque Numérique de l'AES** (Alliance des États du Sahel). L'utilisateur voit une banque simple comme Orange Money. La blockchain travaille en silence.

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
| Réseau Mesh | — | ✅ AfriMesh Direct |

**Rien ne vient de l'Occident. Tout est africain. 100% Rust std.**

## 54 Pays Africains

Tous les pays africains sont connectés:
- 🇲🇱 Mali (+223), 🇳🇪 Niger (+227), 🇧🇫 Burkina Faso (+226) — AES
- 🇸🇳 Sénégal (+221), 🇨🇮 Côte d'Ivoire (+225), 🇬🇭 Ghana (+233)
- 🇳🇬 Nigeria (+234), 🇨🇲 Cameroun (+237), 🇰🇪 Kenya (+254)
- ... **54 pays au total**

Chaque utilisateur a un numéro de téléphone africain. Les transferts transfrontaliers sont instantanés et gratuits.

## Architecture — Deux Modes

### 🏦 Centre de Données (Admin)
L'admin voit TOUT. Comme CTU dans *24 Heures Chrono*.

| # | Module | Fonction |
|---|--------|----------|
| 1 | 👛 Wallet | Créer des wallets |
| 2 | 📤 Envoyer | Transferts AFR |
| 3 | ⛏️ Miner | Proof of Work |
| 4-6 | 👤 Inscription, 🔑 Login, Compte | Gestion compte |
| 7 | ⛓️ Blockchain | Voir tous les blocs |
| 8 | 📖 Annuaire | 54 pays |
| 9 | 📊 Statut | État du réseau |
| 10 | 📡 AfriMesh Direct | Réseau sans opérateur |
| 11-12 | 💬 Messages | Mesh send/inbox |
| 13 | 🚨 Alertes AI | 30 mots-clés, 3 niveaux |
| 14 | 📋 Journal | Surveillance totale |
| 15 | 📊 Tableau de bord | Vue d'ensemble |
| 16 | 📢 Broadcast | Message à toute l'Afrique |
| 17 | 👥 Gestion utilisateurs | Suivre les traces |
| 18 | 🏦 Émettre AFR | Banque centrale |
| 19 | ❄️ Gel/Dégel | Sécurité bancaire |
| 20 | ℹ️ Info Système | Carte d'identité |
| 21 | 🔑 Changer mot de passe | Sécurité admin |
| 22 | 🦁 AES | Alliance des États du Sahel |
| 23 | 💾 Sauvegarde | Export/Import données |
| 24 | 🏛️ AI Secret | Terminal Mystique 3100 (11 options) |
| 25 | 📿 Langage Sacré | Bible, Coran, Tradition — code = prière |

🔐 **Mot de passe admin** protégé par AfriHash-256.

### 📱 Client (Utilisateur)
Le client ne voit PAS la blockchain. Il voit une banque simple.

```
╔══════════════════════════════════════╗
║  💚 AFRICHAIN — Votre argent,        ║
║     votre continent                  ║
╠══════════════════════════════════════╣
║  👤 Machine 🌍 Niger              ║
║  📱 +227XXXXXXXX                    ║
║  💰 Solde: 150 AFR                  ║
║  📋 Transactions: 5  💬 Messages: 2 ║
╚══════════════════════════════════════╝
```

- 🔑 Se connecter / 📝 S'inscrire
- 📤 Envoyer (par numéro de téléphone, pas d'adresses)
- 📥 Mon adresse (numéro de téléphone)
- 💬 LES NOIRES — messagerie (sans WhatsApp)
- 📖 Annuaire panafricain
- 🌱 PLANTÉ VERTE — réseau social (sans Facebook)
- 🔍 SAHARA AFRI — recherche (sans Google)
- 📜 Historique de transactions

## Afri-Net — Internet Africain

| Occident | AfriChain |
|----------|-----------|
| WhatsApp / Meta | 💬 LES NOIRES |
| Facebook / Meta | 🌱 PLANTÉ VERTE |
| Google / Alphabet | 🔍 SAHARA AFRI |
| SWIFT / Brussels | ⛓️ AFR Chain |
| AWS / Amazon | ☀️ Solar Cloud Africa |
| OpenAI / Microsoft | 🧠 AI Africaine |

## Sécurité

- 🛡️ Bouclier X9 — anti-intrusion, blocage IP, rate limiting
- 🚨 AI Veille — 30 mots-clés, 3 niveaux (CRITIQUE/ALERTE/VIGILANCE)
- 🏛️ AI Secret — Terminal Mystique 3100:
  - 💬 Communication machine (◈⬡⊕⟠⬢ → français)
  - 📋 Rapports (jour/semaine/total/menaces)
  - 🌍 Trois mondes — Morts, Vivants, Machines
  - 🧠 Intelligence supérieure (fusion des trois mondes)
  - 🫥 Invisibilité africaine (anti-surveillance)
  - 🎯 Destruction de drones ennemis à distance
  - ☀️ 100 milliards de drones solaires
- ❄️ Gel de comptes suspects
- 📋 Journal d'activité — toutes les actions enregistrées
- 🔐 Mot de passe admin (AfriHash-256)
- 🔐 Signatures Ed25519 (from scratch)

## AES — Alliance des États du Sahel

🇲🇱 🇳🇪 🇧🇫 — Mali · Niger · Burkina Faso

**Objectifs AES:**
1. Monnaie souveraine — AFR remplace le FCFA
2. Réseau mesh — sans Orange/MTN/Moov
3. Banque invisible — l'utilisateur ne voit rien
4. AI veille — protection contre les ennemis
5. Zéro dépendance — 100% africain

**Trois lions. Une blockchain. Un avenir.**

## Technique

- **Langage:** Rust (édition 2021)
- **Dépendances:** ZÉRO (std uniquement)
- **Taille:** ~16,000 lignes (main.rs + 8 modules)
- **Modules:** 9 fichiers Rust (afri_ed25519, afri_hash, afri_rng, afri_hex, afri_time, afri_json, afri_http, afri_mesh_direct, main)
- **Version:** v0.72 — Langage Sacré
- **Build:** `cargo build --release` (5 secondes)
- **Données:** `~/afririch/` (chemins absolus)
- **Persistance:** blockchain.json, wallets.json, users.json, social_feed.json, alerts.json, activity.json, broadcasts.json, frozen_users.json, admin_password.json, ai_memory.json

## Auteur

Construit à la main, ligne par ligne, sur un téléphone Android dans nano sur Termux.

*L'Afrique est le continent le plus riche. Il est temps que sa technologie le reflète.* 💚🪙
