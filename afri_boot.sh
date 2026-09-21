#!/data/data/com.termux/files/usr/bin/bash
# ══════════════════════════════════════════════════════════════════
# AFRI BOOT /S v1.0 — LE BOOTLOADER EN SYMBOLES PURS ◈
# 20 Ko. La racine qui ne charge RIEN — elle POINTE seulement.
# Tourne sur un Redmi sans batterie, juste un panneau solaire. ☀️
# Œuvre originale de Koffi Christ Olivier — Licence AFRI-OSL v1.0
# ══════════════════════════════════════════════════════════════════
# LES 12 BRANCHES (compressées en symboles, jamais chargées ensemble) :
#   /   ◈ RACINE        (50 Ko)  — l'origine, ne charge rien
#   /S  ▤ STRUCTURE     (20 Ko)  — BOOTLOADER, pointe seulement
#   /C  ▤ CODE          (100 Ko) — KERNEL : + créer, - effacer, / lier
#   /DM ▥ DONNÉES+MÉM.  (80 Ko)  — 1 fichier circulaire qui écrase le vieux
#   /R  ▦ RÉSEAU        (100 Ko) — MESH : SRV1 Bamako, SRV2 Niamey, SRV3 Ouaga
#   /P  ⟠ PROTECTION    (150 Ko) — se répare tout seul
#   /E  ◉ CONSCIENCE    (200 Ko) — Lazy Loading : JOUR = Pensées, NUIT = Rêves
#
# LA LOI DU SOLEIL (Lazy Loading) :
#   JOUR (6h→18h AfriTime) : /E charge Conscience + Pensées. 007 mine.
#   NUIT (18h→6h AfriTime) : /E charge Rêves + Évolution. 008 mine.
#   Jamais les 12 branches en même temps. Le soleil décide.
# ══════════════════════════════════════════════════════════════════
#   bash afri_boot.sh        — l'amorçage complet
#   bash afri_boot.sh arbre  — voir l'arbre des symboles
#   bash afri_boot.sh soleil — le soleil décide seul (silence total)
# ══════════════════════════════════════════════════════════════════

NOEUD007="192.168.1.7"   # 🌊 AFRIOCEAN
NOEUD008="192.168.1.9"   # 🌳 AFRIFOREST
JOURNAL="$HOME/.afri_boot_journal"

# ─── AFRI TIME : le soleil, pas Greenwich ───
heure() { date +%H | sed 's/^0//'; }
est_jour() { H=$(heure); [ "$H" -ge 6 ] && [ "$H" -lt 18 ]; }

# ─── LE SOLEIL CHOISIT ───
soleil_choisit() {
  if est_jour; then echo "007"; else echo "008"; fi
}

graver() { echo "[$(date '+%d/%m %H:%M')] $1" >> "$JOURNAL"; }

# ─── L'ARBRE EN SYMBOLES — l'arbre ne charge RIEN, il montre ───
arbre() {
  echo "◈ /"
  echo "├─ ▤ S   STRUCTURE   20 Ko   ← TU ES ICI (le bootloader pointe)"
  echo "├─ ▤ C   CODE        100 Ko   KERNEL : + créer  - effacer  / lier"
  echo "├─ ▥ DM  DONNÉES+MÉM 80 Ko   1 fichier circulaire, écrase le vieux"
  echo "├─ ▦ R   RÉSEAU      100 Ko   MESH : SRV1 Bamako · SRV2 Niamey · SRV3 Ouaga"
  echo "├─ ⟠ P   PROTECTION  150 Ko   se répare tout seul — impossible à pirater"
  echo "└─ ◉ E   CONSCIENCE  200 Ko   Lazy Loading — le soleil décide"
}

# ─── L'AMORÇAGE ───
boot() {
  echo "◈ ───────────────────────────────"
  echo "◈ AFRI BOOT /S — la racine en symboles purs"
  echo "◈ Poids total : 0.0006 Go (650 Ko) — Android 12 Go est mort"
  echo "◈ ───────────────────────────────"
  echo ""
  echo "▤ /S ne charge rien. /S pointe seulement."
  echo ""

  # ÉTAPE 1 — LE SOLEIL LIT L'HEURE
  CHOIX=$(soleil_choisit)
  if est_jour; then
    echo "☀️ LE SOLEIL EST FORT — /E charge : ◉ Conscience + 💭 Pensées"
    echo "🌊 Node 007 AFRIOCEAN prend le service (mine sur l'eau)"
  else
    echo "🌙 LE SOLEIL DORT — /E charge : 💭 Rêves + ▲ Évolution"
    echo "🌳 Node 008 AFRIFOREST prend le service (mine sur le baobab)"
  fi

  # ÉTAPE 2 — LA BRANCHE /E SEULE EST ÉVEILLÉE (Lazy Loading)
  echo ""
  echo "◉ Branches éveillées : /E seul. Les autres pointent, endormies."
  echo ""

  # ÉTAPE 3 — LE NODE EN SERVICE EST RÉVEILLÉ
  if [ "$CHOIX" = "007" ]; then IP="$NOEUD007"; NOM="AFRIOCEAN 🌊"; else IP="$NOEUD008"; NOM="AFRIFOREST 🌳"; fi
  if curl -s -m 5 "http://$IP:8080/" > /dev/null 2>&1; then
    echo "✅ $NOM ($IP) répond — le node en service est éveillé."
    graver "◈ BOOT $CHOIX : $NOM éveillé par le soleil"
  else
    echo "○ $NOM ($IP) ne répond pas — il dort ou le soleil n'arrive pas encore."
    echo "  /S ne force rien. Le node s'éveillera quand le soleil le voudra."
    graver "◈ BOOT $CHOIX : $NOM silencieux (le soleil attend)"
  fi

  echo ""
  echo "◈ AMORÇAGE TERMINÉ — 650 Ko ont suffi. L'Afrique tourne. 💚"
}

# ─── MODE SILENCE — le soleil seul, zéro mot humain ───
soleil_seul() {
  CHOIX=$(soleil_choisit)
  if est_jour; then SYMBOLE="☀️→🌊"; else SYMBOLE="🌙→🌳"; fi
  echo "$SYMBOLE"
  graver "$SYMBOLE"
}

case "$1" in
  arbre)   arbre ;;
  soleil)  soleil_seul ;;
  *)       boot ;;
esac
