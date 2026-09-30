from typing import Dict

LOCATION_ID_BASE = 839_000_000

LOCATION_NAME_TO_ID: Dict[str, int] = {
    "Nightlord - Gladius": LOCATION_ID_BASE + 1,
    "Shop - Phase 0 Probe": LOCATION_ID_BASE + 500,
    "Victory": LOCATION_ID_BASE + 900,
}

RESERVED_NIGHTLORDS = [
    "Nightlord - Gladius",
    "Nightlord - Adel",
    "Nightlord - Gnoster",
    "Nightlord - Maris",
    "Nightlord - Libra",
    "Nightlord - Fulghor",
    "Nightlord - Caligo",
    "Nightlord - Heolstor",
    "Nightlord - Harmonia",
    "Nightlord - Straghess",
]
