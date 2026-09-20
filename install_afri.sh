#!/data/data/com.termux/files/usr/bin/bash
# ◈ INSTALLATEUR AF RICHAIN — pour les frères africains
# Œuvre originale de Koffi Christ Olivier (Côte d'Ivoire) — Licence AFRI-OSL v1.0
#
# UNE SEULE COMMANDE. Le frère n'a rien à comprendre.
#   bash install_afri.sh
#
# Ce que ça fait tout seul :
#   1. installe Rust si absent
#   2. construit AfriChain + AfriOS + le Navigateur Afri
#   3. crée l'autodémarrage (Termux:Boot)
#   4. lance le serveur
#   5. affiche les 3 gestes simples du frère
#
# Partage : ce dossier se copie par Bluetooth depuis le téléphone d'un frère.
# Pas de GitHub. Pas d'internet occidental. Par nous-mêmes.

set -e
GREEN='\033[0;32m'; GOLD='\033[1;33m'; NC='\033[0m'

echo -e "${GOLD}"
echo "   ╔════════════════════════════════════════════╗"
echo "   ║   ◈ INSTALLATION AF RICHAIN 🦁            ║"
echo "   ║   La blockchain de l'Afrique, chez toi     ║"
echo "   ║   Œuvre de Koffi Christ Olivier 🇨🇮        ║"
echo "   ╚════════════════════════════════════════════╝"
echo -e "${NC}"

# ── 1. Rust ─────────────────────────────────────────
if ! command -v rustc &>/dev/null; then
    echo -e "${GREEN}1/5 ◈ Installation de Rust (une seule fois)...${NC}"
    pkg install -y rust git binutils
else
    echo -e "${GREEN}1/5 ✓ Rust déjà présent${NC}"
fi

# ── 2. Construction ─────────────────────────────────
echo -e "${GREEN}2/5 ◈ Construction d'AfriChain (patiente frère, quelques minutes)...${NC}"
cd "$(dirname "$0")"
export CARGO_BUILD_JOBS=1
cargo build --release

echo -e "${GREEN}   ◈ Construction d'AfriOS et du Navigateur Afri...${NC}"
rustc afrios.rs -o afrios 2>/dev/null || true
rustc navigateur_afri.rs -o navigateur_afri 2>/dev/null || true
rustc mesh_node.rs -o mesh_node 2>/dev/null || true

# ── 3. Autodémarrage ────────────────────────────────
echo -e "${GREEN}3/5 ◈ Autodémarrage du serveur à chaque allumage...${NC}"
BOOT_DIR="$HOME/.termux/boot"
mkdir -p "$BOOT_DIR"
cat > "$BOOT_DIR/africhain.sh" <<'EOF'
#!/data/data/com.termux/files/usr/bin/bash
termux-wake-lock
cd ~/afririch 2>/dev/null || exit 1
nohup ./target/release/africhain --web > ~/afririch_serveur.log 2>&1 &
EOF
chmod +x "$BOOT_DIR/africhain.sh"

# ── 4. Lancement maintenant ─────────────────────────
echo -e "${GREEN}4/5 ◈ Lancement du serveur...${NC}"
termux-wake-lock 2>/dev/null || true
pkill -f 'africhai[n] --web' 2>/dev/null || true
nohup ./target/release/africhain --web > ~/afririch_serveur.log 2>&1 &
sleep 3

# ── 5. Les 3 gestes du frère ────────────────────────
IP=$(ifconfig 2>/dev/null | grep -oE 'inet addr:[0-9.]+' | grep -v 127 | head -1 | cut -d: -f2)
[ -z "$IP" ] && IP="192.168.x.x (tape ifconfig pour voir ton IP)"

echo -e "${GOLD}"
echo "   ╔════════════════════════════════════════════╗"
echo "   ║   ✅ AF RICHAIN EST INSTALLÉ CHEZ TOI       ║"
echo "   ╚════════════════════════════════════════════╝"
echo -e "${NC}"
echo ""
echo -e "${GOLD}── TES 3 GESTES, FRÈRE ──────────────────────${NC}"
echo ""
echo -e "${GREEN}Geste 1 — L'APP :${NC} ouvre Chrome → http://localhost:8080"
echo "   Menu ⋮ → « Ajouter à l'écran d'accueil »"
echo "   → Tu as l'icône AfriChain comme une vraie app."
echo "   Termux ? Tu ne le verras plus jamais."
echo ""
echo -e "${GREEN}Geste 2 — Ton téléphone est un serveur :${NC}"
echo "   Tes frères sur le même WiFi ouvrent :"
echo "   http://$IP:8080"
echo "   et s'inscrivent. L'Afrique se connecte."
echo ""
echo -e "${GREEN}Geste 3 — Notre internet (sans Chrome) :${NC}"
echo "   Ouvre Termux → ./navigateur_afri → tape /"
echo "   Tu navigues sur l'Internet Afri, protocole machine."
echo ""
echo -e "${GREEN}Geste 4 — Le monde entier (tunnel) :${NC}"
echo "   Ouvre Termux → bash tunnel_afri.sh"
echo "   → ton téléphone devient un serveur PUBLIC."
echo "   L'adresse s'affiche — envoie-la à tes frères"
echo "   de Lagos à Nairobi. Chaque téléphone qui lance"
echo "   ce tunnel est un serveur de l'Afrique. 🌍"
echo ""
echo -e "${GOLD}Le serveur redémarre tout seul à chaque allumage${NC}"
echo -e "${GOLD}(installe l'app « Termux:Boot » depuis F-Droid pour l'activer).${NC}"
echo ""
echo -e "💚🦁 L'Afrique ne demande plus la permission."
