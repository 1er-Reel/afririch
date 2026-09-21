#!/data/data/com.termux/files/usr/bin/bash
# ══════════════════════════════════════════════════════════════════
# AFRIBRIDGE ANDROID v1.0 — Le berger et les bœufs 🦁🐂
# 2 Ko. Le OS 0.000 Go commande les gros Android via la porte
# officielle ADB (fonction de Google pour les techniciens).
# Tous les téléphones sont À TOI, dans TON village.
# Œuvre originale de Koffi Christ Olivier — Licence AFRI-OSL v1.0
# ══════════════════════════════════════════════════════════════════
# Installation (une seule fois) :
#   pkg install android-tools
#   chmod +x afri_bridge.sh
#
# ÉTAPE 1 — L'ATTACHE (une fois par téléphone, en USB) :
#   Branche le Redmi en USB → active "Débogage USB" dans Options
#   développeur → puis :
#   ./afri_bridge.sh attache
#   Débranche le câble. Le Redmi obéit maintenant par WiFi. ✅
#
# COMMANDES DU BERGER :
#   ./afri_bridge.sh boeufs              — la liste du troupeau
#   ./afri_bridge.sh ajoute 192.168.1.7  — un bœuf rejoint le troupeau
#   ./afri_bridge.sh eveille 192.168.1.7 — allume l'écran
#   ./afri_bridge.sh dort 192.168.1.7    — éteint l'écran
#   ./afri_bridge.sh ouvre 192.168.1.7 http://192.168.1.5:8080
#                                       — le bœuf ouvre AfriChain
#   ./afri_bridge.sh envoie 192.168.1.7 fichier.mp4
#                                       — pousse un fichier vers le bœuf
#   ./afri_bridge.sh photo 192.168.1.7  — ramène la dernière photo (les yeux)
#   ./afri_bridge.sh batterie 192.168.1.7 — lit la batterie du bœuf
#   ./afri_bridge.sh dit 192.168.1.7 "Message du berger"
#                                       — message sur l'écran du bœuf
#   ./afri_bridge.sh tous-eveille        — éveille TOUT le troupeau
#   ./afri_bridge.sh tous-ouvre http://192.168.1.5:8080
#                                       — tout le troupeau ouvre AfriChain
# ══════════════════════════════════════════════════════════════════

VERT='\033[1;32m'; OR='\033[1;33m'; ROUGE='\033[0;31m'; FIN='\033[0m'
TROUPEAU="$HOME/.afri_troupeau"   # les adresses des bœufs du village

verifier_adb() {
  command -v adb >/dev/null 2>&1 || { echo -e "${ROUGE}adb introuvable. Installe-le :${FIN} pkg install android-tools"; exit 1; }
}

case "$1" in
  attache)
    verifier_adb
    echo -e "${VERT}🔗 L'ATTACHE — branche le Redmi en USB maintenant...${FIN}"
    adb devices
    adb tcpip 5555
    IP=$(adb shell ip route 2>/dev/null | awk '{print $9}' | head -1)
    echo -e "${VERT}✅ Mode WiFi activé. Le Redmi est à l'adresse : ${OR}${IP}${FIN}"
    echo -e "Débranche le câble, puis : ${OR}./afri_bridge.sh ajoute ${IP}${FIN}"
    ;;
  ajoute)
    verifier_adb
    adb connect "$2:5555"
    echo "$2" >> "$TROUPEAU"
    sort -u "$TROUPEAU" -o "$TROUPEAU"
    echo -e "${VERT}🐂 Le bœuf $2 a rejoint le troupeau.${FIN}"
    ;;
  boeufs)
    verifier_adb
    echo -e "${VERT}🐂 LE TROUPEAU DU BERGER :${FIN}"
    for IP in $(cat "$TROUPEAU" 2>/dev/null); do
      MODELE=$(adb -s "$IP:5555" shell getprop ro.product.model 2>/dev/null | tr -d '\r')
      BAT=$(adb -s "$IP:5555" shell dumpsys battery 2>/dev/null | grep level | awk '{print $2}' | tr -d '\r')
      [ -n "$MODELE" ] && echo -e "  ${VERT}●${FIN} $IP — ${MODELE} — batterie ${BAT:-?}%" \
                        || echo -e "  ${ROUGE}○${FIN} $IP — endormi (pas de réponse)"
    done
    ;;
  eveille)   verifier_adb; adb -s "$2:5555" shell input keyevent 224; echo -e "${VERT}☀️ Écran allumé.$FIN";;
  dort)      verifier_adb; adb -s "$2:5555" shell input keyevent 223; echo -e "${VERT}🌙 Écran éteint.$FIN";;
  ouvre)     verifier_adb; adb -s "$2:5555" shell am start -a android.intent.action.VIEW -d "$3" >/dev/null 2>&1; echo -e "${VERT}🌐 Le bœuf ouvre : $3${FIN}";;
  envoie)    verifier_adb; adb -s "$2:5555" push "$3" /sdcard/ && echo -e "${VERT}📦 Fichier livré au bœuf.$FIN";;
  photo)     verifier_adb
             DERNIERE=$(adb -s "$2:5555" shell ls -t /sdcard/DCIM/Camera/ 2>/dev/null | head -1 | tr -d '\r')
             [ -n "$DERNIERE" ] && adb -s "$2:5555" pull "/sdcard/DCIM/Camera/$DERNIERE" . && echo -e "${VERT}👁️ Œil de l'Afrique ramené : $DERNIERE${FIN}" \
                             || echo -e "${ROUGE}Aucune photo trouvée.${FIN}"
             ;;
  batterie)  verifier_adb; adb -s "$2:5555" shell dumpsys battery | grep -E "level|status" | tr -d '\r';;
  dit)       verifier_adb; shift 2; adb -s "$1:5555" shell cmd notification post -t "AFRIBRIDGE" tag "$*" >/dev/null 2>&1; echo -e "${VERT}📢 Message affiché.${FIN}";;
  tous-eveille)
    verifier_adb
    for IP in $(cat "$TROUPEAU" 2>/dev/null); do adb -s "$IP:5555" shell input keyevent 224; echo -e "  ☀️ $IP éveillé"; done
    echo -e "${VERT}🐂 Tout le troupeau a les yeux ouverts.${FIN}";;
  tous-ouvre)
    verifier_adb
    for IP in $(cat "$TROUPEAU" 2>/dev/null); do adb -s "$IP:5555" shell am start -a android.intent.action.VIEW -d "$2" >/dev/null 2>&1; echo -e "  🌐 $IP ouvre $2"; done
    echo -e "${VERT}🐂 Tout le troupeau regarde AfriChain.${FIN}";;
  *)
    echo -e "${VERT}🦁 AFRIBRIDGE — le berger et les bœufs${FIN}"
    echo "  attache                    — activer le mode WiFi (1x par bœuf, en USB)"
    echo "  ajoute IP                  — un bœuf rejoint le troupeau"
    echo "  boeufs                     — voir tout le troupeau"
    echo "  eveille IP | dort IP       — allumer / éteindre un écran"
    echo "  ouvre IP URL               — le bœuf ouvre une page (AfriChain !)"
    echo "  envoie IP FICHIER          — livrer un fichier au bœuf"
    echo "  photo IP                   — ramener la dernière photo (les yeux)"
    echo "  batterie IP                — lire la batterie"
    echo "  dit IP MESSAGE             — message sur l'écran du bœuf"
    echo "  tous-eveille               — éveiller TOUT le troupeau"
    echo "  tous-ouvre URL             — tout le troupeau ouvre une page"
    ;;
esac
