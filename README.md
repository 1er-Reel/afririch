# 🦁 AfriChain — La blockchain 100% africaine

> L'Afrique n'a pas besoin de permission. 💚

## 🪙 AfriRich (AFR)

Blockchain indépendante codée from scratch en Rust. **Pas de fork Bitcoin. Pas de token Ethereum. Pas de dépendance étrangère.** Liée à la monnaie AES (Alliance of Sahel States : Mali, Burkina Faso, Niger).

## ✨ Fonctionnalités

- ⛏️ **Proof of Work** — minage avec SHA256
- 🌐 **Web Explorer** — visualise les blocs, soldes et API
- 👛 **Wallet** — crée une adresse, consulte ton solde, envoie des AFR
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
| `/blocks` | Tous les blocs minés |
| `/balances` | Soldes de tous les wallets |
| `/wallet` | Ton wallet — créer, consulter, envoyer, miner |
| `/api/status` | API JSON — statut de la chaîne |
| `/api/blocks` | API JSON — tous les blocs |

## 🏗️ Architecture

```
src/main.rs    — Tout le code (blockchain + serveur web + wallet)
Cargo.toml     — Dépendances (serde, sha2, chrono, actix-web)
```

## 🛡️ Souveraineté

AfriChain n'est pas un token sur une chaîne existante. C'est une **blockchain complète**, construite from scratch, qui appartient à l'Afrique.

## 📜 Licence

MIT — Libre d'utilisation, de modification et de distribution.

## 👤 Auteur

**Machine-senpai** — codé à la main sur Termux (Android), Redmi 15.

---

🦁 *L'Afrique est le continent le plus riche. Aucune crypto ne peut la dépasser, même pas BTC.* 💚
