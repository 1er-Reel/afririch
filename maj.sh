#!/data/data/com.termux/files/usr/bin/bash
# ◈ MISE À JOUR AFRICHAIN — UNE SEULE COMMANDE 🦁
# Œuvre originale de KOFFI CHRIST OLIVIER (Côte d'Ivoire) — Licence AFRI-OSL v1.0.
#
#   bash maj.sh
#
# Répare tout : les fichiers modifiés par le serveur sont remis,
# le git pull passe, le build se fait, le serveur repart.
# À la fin, la VERSION s'affiche — tu vois si tu es à jour.

GREEN='\033[0;32m'; GOLD='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'
cd ~/afririch || { echo -e "${RED}Dossier ~/afririch introuvable${NC}"; exit 1; }

echo -e "${GOLD}═══ MISE À JOUR AFRICHAIN ═══${NC}"

# 1. Arrêter le serveur proprement (le regex évite de tuer ce script)
pkill -f 'africhai[n] --web' 2>/dev/null
sleep 1

# 2. Réparer les fichiers que le serveur modifie (blockchain.json, etc.)
#    On les remet à l'état git pour que le pull passe.
echo -e "${GREEN}1/4 Réparation des fichiers...${NC}"
git checkout -- machines.json seeds.json 2>/dev/null
git checkout -- blockchain.json wallets.json users.json 2>/dev/null
# Les fichiers de données locaux ne sont PAS suivis par git (gitignorés) —
# tes blocs, tes comptes, ton argent restent INTACTS. Seul le code bouge.

# 3. Tirer la dernière version (force si besoin)
echo -e "${GREEN}2/4 Récupération de la dernière version...${NC}"
git stash 2>/dev/null
git pull --rebase 2>&1 | tail -1
git stash pop 2>/dev/null

# 4. Compiler (une seule tâche pour la RAM du téléphone)
echo -e "${GREEN}3/4 Compilation (patiente, ne ferme pas Termux)...${NC}"
CARGO_BUILD_JOBS=1 cargo build --release --bin africhain 2>&1 | tail -2

# 5. Relancer
echo -e "${GREEN}4/4 Relance du serveur...${NC}"
termux-wake-lock 2>/dev/null
nohup ./target/release/africhain --web > ~/afririch_serveur.log 2>&1 &
sleep 3

# La version — TU VOIS SI TU ES À JOUR
echo ""
echo -e "${GOLD}═══ VERSION INSTALLÉE ═══${NC}"
grep -o 'v2\.[0-9.]*' src/main.rs | head -1
echo ""
echo -e "${GREEN}✅ Serveur sur http://localhost:8080${NC}"
echo -e "${GREEN}👑 Trône sur http://localhost:9090${NC}"
echo ""
echo -e "${GOLD}Si la version affichée est v2.23 ou plus → tu es à jour. 🦁${NC}"
