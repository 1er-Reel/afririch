#!/data/data/com.termux/files/usr/bin/bash
# ◈ TUNNEL AFRI 🌍 — ton téléphone devient un serveur PUBLIC
# Œuvre originale de KOFFI CHRIST OLIVIER (Côte d'Ivoire) — Licence AFRI-OSL v1.0.
#
# UNE SEULE COMMANDE : bash tunnel_afri.sh
# → une adresse publique s'affiche. Envoie-la à tes frères :
#   ils ouvrent AfriChain depuis Lagos, Nairobi, Dakar — n'importe où.
#
# Chaque téléphone qui lance ce tunnel = un serveur de l'Afrique.
# Pas de sandbox. Pas de cloud. Le serveur est DANS TA MAIN.

GREEN='\033[0;32m'; GOLD='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'

# Vérifier que le serveur AfriChain vit
if ! curl -s -o /dev/null --max-time 3 http://localhost:8080/; then
    echo -e "${RED}Le serveur AfriChain ne tourne pas.${NC}"
    echo "Lance-le d'abord : cd ~/afririch && ./target/release/africhain --web"
    exit 1
fi

echo -e "${GOLD}"
echo "   ╔════════════════════════════════════════════╗"
echo "   ║   ◈ TUNNEL AFRI 🌍                         ║"
echo "   ║   Ton téléphone = un serveur de l'Afrique   ║"
echo "   ╚════════════════════════════════════════════╝"
echo -e "${NC}"

termux-wake-lock 2>/dev/null || true

# Le tunnel : localhost.run en SSH (aucune installation, aucun compte)
echo -e "${GREEN}Ouverture du tunnel... (Ctrl+C pour fermer)${NC}"
echo ""
ssh -o StrictHostKeyChecking=no -o ServerAliveInterval=30 \
    -R 80:localhost:8080 nokey@localhost.run 2>&1 | \
    grep --line-buffered -oE "[a-z0-9-]+\.(lhr\.life|trycloudflare\.com)" | \
    while read URL; do
        echo ""
        echo -e "${GOLD}═══════════════════════════════════════════════${NC}"
        echo -e "${GREEN}🌍 ADRESSE PUBLIQUE DE TON TÉLÉPHONE :${NC}"
        echo ""
        echo -e "${GOLD}   http://$URL${NC}"
        echo ""
        echo "   Envoie cette adresse à tes frères :"
        echo "   WhatsApp, SMS, Bluetooth, papier —"
        echo "   ils ouvrent AfriChain depuis n'importe où."
        echo ""
        echo -e "${GOLD}═══════════════════════════════════════════════${NC}"
        echo -e "${GREEN}(laisse Termux ouvert — le tunnel vit tant que"
        echo "   cette fenêtre vit. termux-wake-lock actif.)${NC}"
    done

# Si le SSH meurt, on relance tout seul
while true; do
    sleep 5
    ssh -o StrictHostKeyChecking=no -o ServerAliveInterval=30 \
        -R 80:localhost:8080 nokey@localhost.run 2>&1 | \
        grep --line-buffered -oE "[a-z0-9-]+\.(lhr\.life|trycloudflare\.com)" | \
        while read URL; do
            echo -e "${GREEN}🌍 Nouvelle adresse : http://$URL${NC}"
        done
done
