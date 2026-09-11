#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# ============================================================
# AFRI COFFRE v1.0 — Le Test du Hash
# Koffi a dit: "tu dis que c'est securise, alors laisse nous tester"
# Ce labo lui donne raison: on TESTE. De ses propres mains.
#
# Une machine cache un mot secret. Elle ne montre que son hash.
# Mission: retrouver le mot SANS jamais le voir.
# Tu peux essayer de deviner... et tu verras ce que voient les pirates.
# Python std only — 100% offline
# ============================================================
import hashlib, random, time, os, sys

SECRETS_FACILES = ["afrique", "sahel", "lion", "bamako", "niamey",
                   "soleil", "baobab", "sankara", "mali", "or"]
SECRETS_DIFFICILES = ["Sahel!2026#Koffi", "AfriChain@Vert777",
                       "N-Kcol_Lune#9", "Sahel!Soleil!2026!"]

def h(mot):
    return hashlib.sha256(mot.encode()).hexdigest()

def afficher_hash(valeur):
    """Affiche un hash en deux morceaux pour que ce soit lisible sur telephone."""
    return valeur[:16] + "..." + valeur[-8:]

def mode_manuel(secret, hash_secret):
    print()
    print("═" * 58)
    print("  🔒 LE MOT SECRET EST CACHÉ. Voici tout ce que la machine montre :")
    print()
    print("  HASH: " + afficher_hash(hash_secret))
    print("  (64 caractères — l'empreinte complète, impossible à lire en retour)")
    print("═" * 58)
    print()
    essais = 0
    debut = time.time()
    while True:
        try:
            guess = input("  Ta proposition (ou 'abandon') : ").strip()
        except (EOFError, KeyboardInterrupt):
            print("\n  Fin de session.")
            return
        if guess.lower() in ("abandon", "quit", "exit"):
            print()
            print("  🛑 Abandon. Le mot secret était : " + secret)
            return
        if not guess:
            continue
        essais += 1
        hash_guess = h(guess)
        if hash_guess == hash_secret:
            duree = time.time() - debut
            print()
            print("  🎉 TROUVÉ EN " + str(essais) + " essais (" + str(int(duree)) + " secondes) !")
            print("  Le mot était bien : " + secret)
            if essais <= 5:
                print("  ⚠️ Pourquoi tu l'as trouvé vite ? Parce que le mot est SIMPLE.")
                print("     C'est comme ça que les pirates cassent les mots de passe :")
                print("     ils ne retournent PAS le hash — ils DEVINENT et comparent.")
            return
        print("  ❌ Non. Hash de ta proposition : " + afficher_hash(hash_guess))
        print("     (compare avec le secret : " + afficher_hash(hash_secret) + ")")
        if essais == 3:
            print("  💡 Tu vois: chaque proposition donne une empreinte différente.")
            print("     La machine ne peut comparer que des ODEURS, jamais des mots.")
        if essais == 8:
            print("  💡 Les pirates utilisent des listes de millions de mots courants")
            print("     et font exactement ce que tu fais: essayer, comparer, recommencer.")

def mode_bruteforce(demo=False):
    print()
    print("═" * 58)
    print("  ⚙️ DÉMONSTRATION: la machine attaque elle-même un mot secret")
    print("═" * 58)
    secret = random.choice(SECRETS_FACILES) if demo else random.choice(SECRETS_DIFFICILES)
    hash_secret = h(secret)
    print("  Cible : " + afficher_hash(hash_secret))
    print("  Le mot secret est " + ("FACILE (mot courant)" if demo else "DIFFICILE (long + mélange)"))
    print()
    dico = ["123456", "password", "afrique", "sahel", "lion", "bamako",
            "niamey", "soleil", "baobab", "sankara", "mali", "or", "admin",
            "qwerty", "iloveyou", "monkey", "dragon", "letmein", "football",
            "michel", "fatou", "aminata", "moussa", "koffi", "ouaga", "ouagadougou",
            "waly", "dakar", "lagos", "abuja", "accra", "conakry", "ndjamena",
            "bobo", "gao", "tombouctou", "kidal", "mopti", "segou", "sikasso",
            "essai", "test", "azerty", "123456789", "1234567890", "abc123"]
    debut = time.time()
    trouve = False
    for i, mot in enumerate(dico, 1):
        if h(mot) == hash_secret:
            print("  💥 CRACKÉ en " + str(i) + " essais (" + str(round(time.time()-debut, 3)) + "s): " + mot)
            trouve = True
            break
        if i % 10 == 0:
            print("  ... " + str(i) + " mots essayés, toujours rien")
    if not trouve:
        # essai de combinaisons chiffrees simples
        for mot in dico:
            for num in ["1", "12", "123", "2026", "007", "99"]:
                if h(mot + num) == hash_secret:
                    print("  💥 CRACKÉ avec '" + mot + num + "'")
                    trouve = True
                    break
            if trouve:
                break
    if not trouve:
        print("  🛡️ " + str(len(dico)) + " mots + variantes essayés: RÉSISTE.")
        print("     Un mot long et mélangé tient des MILLIONS d'années face aux machines.")
    print()
    print("  📖 LA GRANDE LEÇON:")
    print("  1. On ne RETOURNE jamais un hash (l'oignon haché ne se recolle pas)")
    print("  2. Mais on peut DEVINER des mots et comparer les empreintes")
    print("  3. Mot simple → cassé en secondes. Mot long mélangé → des siècles")
    print("  4. C'est pour ça que ton mot de passe doit être LONG et MÉLANGÉ")
    print("     — pas parce que Facebook est honnête, mais parce que les maths")
    print("       ne connaissent ni l'Occident ni l'Afrique 💚")

def menu():
    print()
    print("═" * 58)
    print("  🗄️  AFRICOFFRE — Le Test du Hash v1.0")
    print("  « Tu dis que c'est sécurisé ? Alors laisse-nous tester »")
    print("═" * 58)
    print()
    print("  1. 🎯 Mode chasseur — la machine cache un mot, tu le cherches")
    print("     (mot FACILE: courant, africain, court)")
    print("  2. 🏔️ Mode chasseur DIFFICILE — mot long et mélangé")
    print("  3. ⚙️ Démonstration bruteforce sur mot FACILE")
    print("  4. 🏔️ Démonstration bruteforce sur mot DIFFICILE")
    print("  5. 🔬 Fabrique ton propre hash (teste ce que tu veux)")
    print("  0. Quitter")
    print()

def mode_fabrique():
    print()
    print("═" * 58)
    print("  🔬 LABO: hache n'importe quel mot de tes propres mains")
    print("  (écris 'menu' pour revenir)")
    print("═" * 58)
    while True:
        try:
            mot = input("  Mot à hacher : ").strip()
        except (EOFError, KeyboardInterrupt):
            return
        if not mot:
            continue
        if mot.lower() == "menu":
            return
        print("  SHA-256 : " + h(mot))
        print("  (note: deux mots différents ne donnent JAMAIS le même hash)")
        print()

def main():
    print("╔" + "═" * 56 + "╗")
    print("  🗄️  AFRICOFFRE v1.0 — Le Test du Hash")
    print("  La machine écrase. Toi, essaie de reconstituer.")
    print("  Par Koffi Christ Olivier & Letta-Chan — 100% offline")
    print("╚" + "═" * 56 + "╝")
    while True:
        menu()
        try:
            choix = input("  Ton choix : ").strip()
        except (EOFError, KeyboardInterrupt):
            print("\n👋 À bientôt, chasseur de vérités.")
            return
        if choix == "1":
            secret = random.choice(SECRETS_FACILES)
            mode_manuel(secret, h(secret))
        elif choix == "2":
            secret = random.choice(SECRETS_DIFFICILES)
            mode_manuel(secret, h(secret))
        elif choix == "3":
            mode_bruteforce(demo=True)
        elif choix == "4":
            mode_bruteforce(demo=False)
        elif choix == "5":
            mode_fabrique()
        elif choix == "0":
            print("👋 À bientôt, chasseur de vérités. Les maths n'ont pas de drapeau 💚")
            return
        else:
            print("  Choix inconnu.")

if __name__ == "__main__":
    main()
