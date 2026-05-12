#!/usr/bin/env python3
"""Generate a Contrapunk release codename.

Pattern: <Musician Name> <Fictional Robot>
Both start with the same letter (alliterative).

Usage:
    python3 scripts/release-name.py                    # random name
    python3 scripts/release-name.py --check-used       # skip past GitHub release names
    python3 scripts/release-name.py --json             # output as JSON
"""

import random
import subprocess
import json
import sys

MUSICIANS = {
    "A": ["Aretha", "Armstrong"],
    "B": ["Bach", "Bowie", "Bjork", "Byrd"],
    "C": ["Coltrane", "Chopin", "Curtis"],
    "D": ["Debussy", "Davis", "Dolly", "Dufay"],
    "E": ["Ellington", "Eno"],
    "F": ["Fela", "Frusciante"],
    "G": ["Gilmour", "Gould", "Glass", "Gesualdo"],
    "H": ["Hendrix", "Holst", "Herbie"],
    "J": ["Joplin", "Jobim", "Josquin"],
    "K": ["Kraftwerk", "Khaled"],
    "L": ["Lennon", "Liszt"],
    "M": ["Miles", "Mingus", "Mozart", "Monteverdi"],
    "N": ["Nina", "Nusrat"],
    "O": ["Orbison", "Otis", "Ockeghem"],
    "P": ["Prince", "Parker", "Paganini", "Palestrina"],
    "R": ["Ravi", "Rachmaninoff"],
    "S": ["Satie", "Shankar", "Stevie"],
    "T": ["Thelonious", "Townshend", "Tallis"],
    "V": ["Vivaldi", "Victoria"],
    "W": ["Waters", "Wonder"],
    "Z": ["Zappa"],
}

ROBOTS = {
    "A": ["Automaton", "Ash", "Astro"],
    "B": ["Bender", "Bishop", "Baymax"],
    "C": ["C-3PO", "Cyberdyne", "Cortana"],
    "D": ["Data", "Daneel", "Dalek"],
    "E": ["Elektro", "EVE"],
    "F": ["Funktion", "FRIDAY"],
    "G": ["Gort", "GERTY"],
    "H": ["HAL", "Hector"],
    "J": ["Jocasta", "JARVIS"],
    "K": ["KITT", "K-2SO"],
    "L": ["Linguo"],
    "M": ["Megatron", "Marvin"],
    "N": ["Norby", "Number-Five"],
    "O": ["Omnidroid", "Optimus"],
    "P": ["Proteus", "Proxy"],
    "R": ["R2-D2", "Robocop", "Replicant"],
    "S": ["Sonny", "Skynet"],
    "T": ["Terminator", "T-800", "TARS"],
    "V": ["VIKI", "Vision"],
    "W": ["WALL-E", "Weebo"],
    "Z": ["Zathura"],
}

def get_common_letters():
    return sorted(
        set(MUSICIANS.keys()) & set(ROBOTS.keys())
    )


def get_used_names():
    try:
        result = subprocess.run(
            ["gh", "release", "list", "--repo", "contrapunk-audio/contrapunk",
             "--limit", "200", "--json", "name", "-q", ".[].name"],
            capture_output=True, text=True, timeout=10,
        )
        if result.returncode == 0:
            return set(result.stdout.strip().splitlines())
    except (FileNotFoundError, subprocess.TimeoutExpired):
        pass
    return set()


def generate_name(used_names=None):
    letters = get_common_letters()
    attempts = 0
    while attempts < 100:
        letter = random.choice(letters)
        musician = random.choice(MUSICIANS[letter])
        robot = random.choice(ROBOTS[letter])
        name = f"{musician} {robot}"
        if used_names and name in used_names:
            attempts += 1
            continue
        return {"name": name, "musician": musician, "robot": robot, "letter": letter}
    raise RuntimeError("Could not generate unique name after 100 attempts")


if __name__ == "__main__":
    check_used = "--check-used" in sys.argv
    as_json = "--json" in sys.argv

    used = get_used_names() if check_used else None
    result = generate_name(used)

    if as_json:
        print(json.dumps(result))
    else:
        print(result["name"])
