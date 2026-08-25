#!/bin/bash
# 🦁 AfriChain — Script de déploiement
# Usage: bash deploy.sh
# Tue l'ancien processus, pull, build, lance.

echo "🦁 AfriChain — Déploiement"
echo "═════════════════════════════"

# Tuer l'ancien processus
echo "🔄 Arrêt de l'ancien processus..."
pkill -f africhain 2>/dev/null; sleep 1

# Pull
echo "📥 Récupération du code..."
cd ~/afririch 2>/dev/null || cd ~/workspace/afririch 2>/dev/null || { echo "❌ Repo non trouvé"; exit 1; }
git pull origin main 2>/dev/null || echo "⚠️ Pull échoué, utilisation du code local"

# Build
echo "🔧 Compilation..."
CARGO_BUILD_JOBS=1 cargo build --release 2>&1 | tail -3

# Vérification
if [ ! -f target/release/africhain ]; then
    echo "❌ Compilation échouée!"
    cargo build --release 2>&1 | grep "error" | head -5
    exit 1
fi

echo ""
echo "✅ BUILD RÉUSSI!"
echo "═════════════════════════════"
echo "🦁 AfriChain v0.68 est prêt."
echo "💚 Lancement..."
echo ""

# Lancer
./target/release/africhain
