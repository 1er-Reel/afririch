#!/data/data/com.termux/files/usr/bin/bash
# ══════════════════════════════════════════════════════════════════
# AFRI CARROUSEL v1.0 — Node 007 AFRIOCEAN ↔ Node 008 AFRIFOREST 🌊🌳
# Le carrousel Jour/Nuit : quand l'eau dort, la forêt veille.
# Œuvre originale de Koffi Christ Olivier — Licence AFRI-OSL v1.0
# ══════════════════════════════════════════════════════════════════
# LE RÊVE DU CHEF :
#   Node 007 — la bouée sur l'océan : mine le JOUR (soleil fort sur
#             l'eau), dort la NUIT et donne sa batterie à Node 008.
#   Node 008 — le baobab de la forêt : garde sa batterie à l'ombre le
#             JOUR, mine la NUIT quand 007 n'a plus de soleil.
#   10 Gardiens de la Nuit — petites boîtes solaires qui surveillent
#             les nodes et font le lien radio entre océan et forêt.
#
# CONFIGURATION (édite ces 2 lignes) :
NOEUD007="192.168.1.7"     # 🌊 AFRIOCEAN — la bouée
NOEUD008="192.168.1.9"     # 🌳 AFRIFOREST — le baobab
# ══════════════════════════════════════════════════════════════════
# COMMANDES :
#   ./afri_carrousel.sh etat      — le soleil, qui veille, les batteries
#   ./afri_carrousel.sh veille    — LA BOUCLE ÉTERNELLE (le carrousel tourne)
#   ./afri_carrousel.sh gardiens  — les 10 gardiens vérifient les nodes
#   ./afri_carrousel.sh journal   — l'histoire des passations
# ══════════════════════════════════════════════════════════════════

VERT='\033[1;32m'; OR='\033[1;33m'; BLEU='\033[1;34m'; ROUGE='\033[0;31m'; FIN='\033[0m'
ETAT="$HOME/.carrousel_etat"        # batteries + dernier service
JOURNAL="$HOME/.carrousel_journal"  # l'histoire des passations

# ─── LE SOLEIL (AfriTime : 6h→18h = jour, pas de Greenwich ici) ───
heure() { date +%H | sed 's/^0//'; }
est_jour() { H=$(heure); [ "$H" -ge 6 ] && [ "$H" -lt 18 ]; }

# ─── LES BATTERIES (persistantes, %) ───
init_etat() {
  [ -f "$ETAT" ] || printf 'B007=100\nB008=100\nSERVICE=007\n' > "$ETAT"
}
lire()   { grep "^$1=" "$ETAT" | cut -d= -f2; }
ecrire() { sed -i "s/^$1=.*/$1=$2/" "$ETAT"; }

# ─── LE JOURNAL — chaque passation est gravée ───
graver() { echo "[$(date '+%d/%m %H:%M')] $1" >> "$JOURNAL"; }

# ─── UNE HEURE DE VIE DU CARROUSEL ───
# JOUR  : 007 charge au soleil (+8%/h) et MINE. 008 repose à l'ombre (+2%/h).
# NUIT  : 008 donne sa lumière (-4%/h) et MINE. 007 dort (-1%/h).
# MINE  = réveiller le node : sa machine interne mine les tx en attente.
reveiller_node() {
  IP="$1"
  # Le node s'éveille : son tick machine mine les transactions en attente
  curl -s -m 5 "http://$IP:8080/" > /dev/null 2>&1
}

cycle() {
  init_etat
  B007=$(lire B007); B008=$(lire B008); SERVICE=$(lire SERVICE)
  if est_jour; then
    SOLEIL="☀️ JOUR — le soleil est fort sur l'eau"
    NOUVEAU="007"
    B007=$(( B007 + 8 )); [ $B007 -gt 100 ] && B007=100
    B008=$(( B008 + 2 )); [ $B008 -gt 100 ] && B008=100
  else
    SOLEIL="🌙 NUIT — le soleil dort sous l'horizon"
    NOUVEAU="008"
    B008=$(( B008 - 4 )); [ $B008 -lt 0 ] && B008=0
    B007=$(( B007 - 1 )); [ $B007 -lt 0 ] && B007=0
  fi
  ecrire B007 $B007; ecrire B008 $B008

  # LA PASSATION — le cœur du carrousel
  if [ "$SERVICE" != "$NOUVEAU" ]; then
    if [ "$NOUVEAU" = "008" ]; then
      graver "🌊→🌳 PASSATION : le soleil se couche sur l'océan. Node 007 dort et donne sa batterie à Node 008. La forêt veille."
      echo -e "${BLEU}🌊→🌳 PASSATION DE NUIT : l'eau dort, la forêt veille.${FIN}"
    else
      graver "🌳→🌊 PASSATION : le soleil se lève sur la forêt. Node 008 rend le service à Node 007. L'océan mine."
      echo -e "${OR}🌳→🌊 PASSATION DU JOUR : l'océan reprend le service.${FIN}"
    fi
    ecrire SERVICE $NOUVEAU
    SERVICE=$NOUVEAU
  fi

  # LE NODE EN SERVICE MINE — les autres dorment
  if [ "$SERVICE" = "007" ]; then
    reveiller_node "$NOEUD007"
    echo -e "🌊 AFRIOCEAN mine ($B007%) — 🌳 AFRIFOREST repose à l'ombre ($B008%)"
  else
    reveiller_node "$NOEUD008"
    echo -e "🌳 AFRIFOREST mine ($B008%) — 🌊 AFRIOCEAN dort ($B007%)"
  fi
}

case "$1" in
  etat)
    init_etat
    if est_jour; then echo -e "${OR}☀️ LE SOLEIL EST LEVÉ — Node 007 AFRIOCEAN est en service.${FIN}"
    else echo -e "${BLEU}🌙 LE SOLEIL DORT — Node 008 AFRIFOREST est en service.${FIN}"; fi
    echo -e "  🌊 Node 007 : $(lire B007)% de batterie"
    echo -e "  🌳 Node 008 : $(lire B008)% de batterie"
    ;;
  veille)
    echo -e "${VERT}🎡 LE CARROUSEL TOURNE — quand l'eau dort, la forêt veille.${FIN}"
    echo -e "   (Ctrl+C pour arrêter. Le journal vit dans ~/.carrousel_journal)"
    while true; do cycle; sleep 3600; done
    ;;
  gardiens)
    echo -e "${VERT}🛡️ LES 10 GARDIENS DE LA NUIT inspectent les nodes...${FIN}"
    for GARDIEN in 1 2 3 4 5 6 7 8 9 10; do
      if curl -s -m 3 "http://$NOEUD007:8080/" > /dev/null 2>&1; then E007="✅"; else E007="❌"; fi
      if curl -s -m 3 "http://$NOEUD008:8080/" > /dev/null 2>&1; then E008="✅"; else E008="❌"; fi
      echo -e "  Gardien $GARDIEN : 🌊 océan $E007 — 🌳 forêt $E008"
      [ "$E007" = "❌" ] && [ "$E008" = "❌" ] && graver "🛡️ ALERTE GARDIEN $GARDIEN : les DEUX nodes sont muets !"
      break  # les 10 gardiens voient la même chose — un seul passage suffit
    done
    ;;
  journal)
    echo -e "${VERT}📖 L'HISTOIRE DU CARROUSEL :${FIN}"
    tail -20 "$JOURNAL" 2>/dev/null || echo "  (vide — le carrousel n'a pas encore tourné)"
    ;;
  *)
    echo -e "${VERT}🎡 AFRI CARROUSEL — Node 007 AFRIOCEAN ↔ Node 008 AFRIFOREST${FIN}"
    echo "  etat      — le soleil, qui veille, les batteries"
    echo "  veille    — la boucle éternelle (le carrousel tourne)"
    echo "  gardiens  — les 10 gardiens vérifient les nodes"
    echo "  journal   — l'histoire des passations"
    echo ""
    echo "  Édite les adresses des nodes en haut du script (NOEUD007 / NOEUD008)."
    ;;
esac
