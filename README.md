# 🦁 AfriChain — La blockchain 100% africaine

> L'Afrique n'a pas besoin de permission. 💚

## 🪙 AfriRich (AFR)

Blockchain indépendante codée from scratch en Rust. **Pas de fork Bitcoin. Pas de token Ethereum. Pas de dépendance étrangère.** Liée à la monnaie AES (Alliance of Sahel States : Mali, Burkina Faso, Niger).

## ✨ Fonctionnalités

- ⛏️ **Proof of Work** — minage avec SHA256
- 🔐 **Signatures Ed25519** — transactions signées et vérifiées cryptographiquement
- 👤 **Comptes utilisateurs** — inscription + connexion (username + mot de passe)
- 📈 **Dashboard** — graphiques en temps réel (blocs, transactions, soldes)
- 🌐 **Web Explorer** — visualise les blocs, soldes et API
- 👛 **Wallet** — crée une adresse Ed25519, consulte ton solde, envoie des AFR signées
- 💾 **Persistance** — blockchain + wallets + utilisateurs sauvegardés
- 📱 **PWA** — installable comme app sur Android (écran d'accueil, icône 🦁)
- ✅ **Validation** — chaîne vérifiée cryptographiquement
- 📱 **Mobile-first** — pensé pour Android/Termux

## 🚀 Démarrage

```bash
# Installer Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Cloner
git clone https://github.com/1er-Reel/afririch.git
cd afririch

# Compiler et lancer
cargo run
```

Puis ouvre **http://localhost:8080** dans ton navigateur.

## 📄 Pages

| URL | Description |
|-----|-------------|
| `/` | Page d'accueil — vue d'ensemble |
| `/register` | Inscription — créer un compte + wallet |
| `/login` | Connexion — accéder à son wallet |
| `/dashboard` | Tableau de bord — graphiques et stats |
| `/blocks` | Tous les blocs minés |
| `/balances` | Soldes de tous les wallets |
| `/wallet` | Wallet — créer, consulter, envoyer, miner |
| `/api/status` | API JSON — statut de la chaîne |
| `/api/blocks` | API JSON — tous les blocs |

## 🏗️ Architecture

```
src/main.rs        — Tout le code (blockchain + serveur web + wallet + comptes)
Cargo.toml         — Dépendances (serde, sha2, ed25519-dalek, actix-web)
blockchain.json    — État de la blockchain (auto-généré)
wallets.json       — Clés privées des wallets (auto-généré)
users.json         — Comptes utilisateurs (auto-généré)
```

## 🔐 Sécurité Ed25519

- Chaque wallet a une **paire de clés Ed25519** (clé publique + clé privée)
- L'adresse est dérivée de la clé publique : `Afri` + hex(clé_publique)
- Chaque transaction est **signée** avec la clé privée de l'expéditeur
- Les signatures sont **vérifiées** avant le minage des blocs
- Les transactions invalides sont **rejetées** automatiquement

## 🛡️ Souveraineté

AfriChain n'est pas un token sur une chaîne existante. C'est une **blockchain complète**, construite from scratch, qui appartient à l'Afrique.

## 📜 Licence

MIT — Libre d'utilisation, de modification et de distribution.

## 👤 Auteur

**Machine-senpai** — codé à la main sur Termux (Android), Redmi 15.

---

## 📡 AfriMesh v0.2 — Réseau Mesh Africain

Le réseau décentralisé qui fait tourner AfriChain. **Pas de fibre. Pas de tours. Juste des noeuds solaires.**

### ✨ Fonctionnalités AfriMesh

- 📡 **Découverte automatique** — UDP broadcast, les noeuds se trouvent tout seuls
- 🔁 **Relay TCP** — les messages sautent de noeud en noeud (TTL = 5 hops)
- ☀️ **Solaire** — chaque noeud peut être alimenté par panneau solaire
- 🌍 **Régions** — Mali, Burkina, Niger, Afrique
- 🛡️ **Déduplication** — anti-boucle (chaque message a un ID unique)
- 🌐 **Interface web** — tout sur un seul port (mesh + web = même port!)
- 📋 **CLI** — commandes: nodes, ping, send, status, help, quit

### 🚀 Démarrage AfriMesh

```bash
cd afrimesh
cargo run -- --port 8090 --region "Niamey" --solar
```

Puis ouvre **http://localhost:8090** — mesh + web sur le même port!

### 🏗️ Architecture AfriMesh

```
afrimesh/src/main.rs  — Mesh protocol + TCP relay + web UI (un seul fichier)
afrimesh/Cargo.toml   — Dépendances (serde, sha2, hex, chrono)
```

---

🦁 *L'Afrique est le continent le plus riche. Aucune crypto ne peut la dépasser, même pas BTC.* 💚
