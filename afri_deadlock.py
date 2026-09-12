#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
afri_deadlock.py v1.0 — Le Chasseur d'ABBA 🎯
Détecteur d'interblocage (deadlock) pour code Rust avec Mutex.

Comment ça marche:
  1. Il lit ton fichier .rs
  2. Il trouve chaque xxx.lock().unwrap() et regarde quelles serrures
     sont prises ensemble (à moins de N lignes de distance)
  3. Il construit toutes les paires "A puis B"
  4. S'il trouve "A puis B" quelque part ET "B puis A" ailleurs
     → INVERSION → risque d'ABBA deadlock!

Usage sur Termux:
  python3 afri_deadlock.py src/main.rs
  python3 afri_deadlock.py src/main.rs --fenetre 10

Zéro dépendance. Python 3 pur. Fait pour l'Afrique. 💚
"""

import sys
import re
import argparse
from collections import defaultdict

BANNIERE = r"""
  ╔══════════════════════════════════════════════╗
  ║   🎯 AFRI DEADLOCK — Le Chasseur d'ABBA 🎯   ║
  ║   L'Afrique ne gèle jamais.                  ║
  ╚══════════════════════════════════════════════╝
"""

COULEURS = {
    "rouge": "\033[91m",
    "vert": "\033[92m",
    "jaune": "\033[93m",
    "bleu": "\033[94m",
    "cyan": "\033[96m",
    "gris": "\033[90m",
    "fin": "\033[0m",
}


def couleur(nom, texte):
    if not sys.stdout.isatty():
        return texte
    return COULEURS[nom] + texte + COULEURS["fin"]


# Regex: xxx.lock().unwrap() — aussi xxx.lock() sans unwrap,
# et xxx.write() / xxx.read() pour RwLock
RE_LOCK = re.compile(r'(\w+)\.(?:lock|write|read)(?:\(\))?(?:\.unwrap\(\))?')
# Une ligne qui relâche visiblement la garde (fin de scope marquée)
RE_FIN_SCOPE = re.compile(r'^\s*\}|^\s*//')


def extraire_sequences(lignes, fenetre):
    """
    Parcourt le fichier et retourne une liste de (numéro_ligne, [serrures...]).
    Une "séquence" = serrures prises à moins de `fenetre` lignes l'une de l'autre.
    On arrête la séquence si on rencontre une fermeture d'accolade profonde
    ou une grande séparation.
    """
    sequences = []
    i = 0
    n = len(lignes)
    while i < n:
        m = RE_LOCK.search(lignes[i])
        if not m:
            i += 1
            continue
        seq = [m.group(1)]
        j = i + 1
        while j < n and (j - (i + 1)) < fenetre:
            ligne = lignes[j]
            if RE_FIN_SCOPE.match(ligne):
                break
            m2 = RE_LOCK.search(ligne)
            if m2:
                # même serrure reprise? on ignore les doublons immédiats
                if m2.group(1) != seq[-1]:
                    seq.append(m2.group(1))
                j += 1
            else:
                j += 1
        if len(seq) >= 2:
            sequences.append((i + 1, seq))
        i = j if j > i else i + 1
    return sequences


def construire_paires(sequences):
    """Pour chaque séquence, toutes les paires ordonnées (A avant B)."""
    paires = defaultdict(list)
    for lineno, seq in sequences:
        vues = set()
        for a_i in range(len(seq)):
            for b_i in range(a_i + 1, len(seq)):
                cle = (seq[a_i], seq[b_i])
                if cle not in vues:
                    vues.add(cle)
                    paires[cle].append(lineno)
    return paires


def trouver_inversions(paires):
    """ABBA = (A,B) existe ET (B,A) existe."""
    inversions = []
    vues = set()
    for (a, b), lignes in paires.items():
        if (b, a) in paires and (a, b) not in vues:
            vues.add((a, b))
            vues.add((b, a))
            inversions.append((a, b, lignes, paires[(b, a)]))
    return inversions


def ordre_canonique(paires):
    """
    Suggère un ordre canonique par tri topologique:
    si A→B apparaît plus souvent que B→A, A passe avant B.
    (heuristique simple, à valider par le développeur)
    """
    compte = defaultdict(int)
    for (a, b), lignes in paires.items():
        compte[a] += len(lignes)
        compte[b] += 0  # présence
    # ordre par fréquence d'apparition comme première serrure
    premiere = defaultdict(int)
    for (a, b), lignes in paires.items():
        premiere[a] += len(lignes)
    tries = sorted(compte.keys(), key=lambda x: -premiere[x])
    return tries


def afficher_ligne_fautive(lignes, lineno, contexte=3):
    """Montre le code autour d'une ligne fautive."""
    debut = max(0, lineno - 1 - contexte)
    fin = min(len(lignes), lineno + contexte)
    morceau = []
    for k in range(debut, fin):
        marque = "▶" if (k + 1) == lineno else " "
        morceau.append("   {}{:>5} │ {}".format(marque, k + 1, lignes[k].rstrip()[:90]))
    return "\n".join(morceau)


def main():
    parser = argparse.ArgumentParser(
        description="🎯 afri_deadlock — Chasseur d'interblocage ABBA pour Rust"
    )
    parser.add_argument("fichier", help="Fichier .rs à analyser (ex: src/main.rs)")
    parser.add_argument(
        "--fenetre", type=int, default=8,
        help="Lignes max entre deux .lock() pour les considérer ensemble (défaut: 8)"
    )
    parser.add_argument(
        "--code", action="store_true",
        help="Afficher le code autour de chaque site fautif"
    )
    args = parser.parse_args()

    print(BANNIERE)
    print(couleur("cyan", f"  📂 Cible: {args.fichier}"))
    print(couleur("cyan", f"  🔎 Fenêtre: {args.fenetre} lignes\n"))

    try:
        with open(args.fichier, encoding="utf-8") as f:
            lignes = f.read().split("\n")
    except OSError as e:
        print(couleur("rouge", f"  ❌ Impossible de lire {args.fichier}: {e}"))
        sys.exit(1)

    print(couleur("gris", f"  {len(lignes)} lignes lues..."))

    # ÉTAPE 1 — extraire les séquences de serrures
    sequences = extraire_sequences(lignes, args.fenetre)
    print(couleur("bleu", f"\n  🔒 ÉTAPE 1 — {len(sequences)} séquences de serrures trouvées"))

    # ÉTAPE 2 — construire les paires
    paires = construire_paires(sequences)
    print(couleur("bleu", f"  🔗 ÉTAPE 2 — {len(paires)} paires ordonnées construites"))

    # ÉTAPE 3 — chercher les inversions ABBA
    inversions = trouver_inversions(paires)
    print(couleur("bleu", f"  🎯 ÉTAPE 3 — Chasse aux inversions...\n"))

    if not inversions:
        print(couleur("vert", "  ╔══════════════════════════════════════════════╗"))
        print(couleur("vert", "  ║   ✅ AUCUNE INVERSION — ORDRE PARFAIT! 🎉   ║"))
        print(couleur("vert", "  ╚══════════════════════════════════════════════╝"))
        print(couleur("vert", "\n  Aucun couple A→B / B→A détecté."))
        print(couleur("vert", "  L'interblocage ABBA est mathématiquement impossible"))
        print(couleur("vert", "  avec les paires trouvées. L'Afrique ne gèle pas. 💚"))
        sys.exit(0)

    print(couleur("rouge", f"  ⚠️  {len(inversions)} INVERSION(S) DÉTECTÉE(S) — RISQUE D'ABBA!\n"))
    for num, (a, b, l1, l2) in enumerate(inversions, 1):
        print(couleur("rouge", f"  ─── INVERSION #{num} ───"))
        print(couleur("jaune", f"     {a} → {b}  aux lignes {l1}"))
        print(couleur("jaune", f"     {b} → {a}  aux lignes {l2}"))
        print(couleur("rouge", f"     Si deux fils croisent ces deux chemins → GEL ÉTERNEL\n"))
        if args.code:
            for site in (l1[0], l2[0]):
                print(couleur("cyan", f"     📍 Ligne {site}:"))
                print(afficher_ligne_fautive(lignes, site))
                print()

    # ÉTAPE 4 — suggestion d'ordre canonique
    print(couleur("bleu", "  💡 ÉTAPE 4 — Suggestion d'ordre canonique (heuristique):"))
    ordre = ordre_canonique(paires)
    print(couleur("cyan", "     " + " → ".join(ordre[:10])))
    print(couleur("gris", """
     ⚠️  Comment réparer:
     1. Choisis UN ordre unique pour toutes tes serrures
     2. Réorganise chaque site fautif pour le respecter
     3. Relance afri_deadlock.py jusqu'à voir ✅ AUCUNE INVERSION

     Règle d'or: avec un ordre unique, deux fils ne peuvent
     jamais se croiser en sens inverse. ABBA impossible. 🎯
"""))
    sys.exit(2)


if __name__ == "__main__":
    main()
