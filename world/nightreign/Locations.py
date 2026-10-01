from typing import Dict, List, Tuple

LOCATION_ID_BASE = 839_000_000

# Shop ids match flags.toml and CSFD4VirtualMemoryFlag::GetFlag on 1.3.3.0.
LOCATION_NAME_TO_ID: Dict[str, int] = {
    "Nightlord - Gladius": LOCATION_ID_BASE + 1,
    "Nightlord - Adel": LOCATION_ID_BASE + 2,
    "Nightlord - Gnoster": LOCATION_ID_BASE + 3,
    "Nightlord - Maris": LOCATION_ID_BASE + 4,
    "Nightlord - Libra": LOCATION_ID_BASE + 5,
    "Nightlord - Fulghor": LOCATION_ID_BASE + 6,
    "Nightlord - Caligo": LOCATION_ID_BASE + 7,
    "Nightlord - Heolstor": LOCATION_ID_BASE + 8,
    "Nightlord - Harmonia": LOCATION_ID_BASE + 9,
    "Nightlord - Straghess": LOCATION_ID_BASE + 10,
    "Board unlock after first Nightlord": LOCATION_ID_BASE + 100,
    "Goal - Nightlord Count": LOCATION_ID_BASE + 200,
    "Shop - Polite Bow": LOCATION_ID_BASE + 501,
    "Shop - Strength": LOCATION_ID_BASE + 503,
    "Shop - Warm Welcome": LOCATION_ID_BASE + 504,
    "Shop - Delicate Burning Scene": LOCATION_ID_BASE + 505,
    "Shop - Heartening Cry": LOCATION_ID_BASE + 506,
    "Shop - Calm Down": LOCATION_ID_BASE + 507,
}

SHOP_LOCATIONS = [
    "Shop - Polite Bow",
    "Shop - Strength",
    "Shop - Warm Welcome",
    "Shop - Delicate Burning Scene",
    "Shop - Heartening Cry",
    "Shop - Calm Down",
]

# Defeat location, expedition unlock item required to reach it.
BASE_NIGHTLORDS: List[Tuple[str, str]] = [
    ("Nightlord - Gladius", "Expedition Unlock - Tricephalos"),
    ("Nightlord - Adel", "Expedition Unlock - Adel"),
    ("Nightlord - Gnoster", "Expedition Unlock - Gnoster"),
    ("Nightlord - Maris", "Expedition Unlock - Maris"),
    ("Nightlord - Libra", "Expedition Unlock - Libra"),
    ("Nightlord - Fulghor", "Expedition Unlock - Fulghor"),
    ("Nightlord - Caligo", "Expedition Unlock - Caligo"),
]

DLC_NIGHTLORDS: List[Tuple[str, str]] = [
    ("Nightlord - Harmonia", "Expedition Unlock - Harmonia"),
    ("Nightlord - Straghess", "Expedition Unlock - Straghess"),
]

SPECIFIC_LOCATION = {
    "gladius": "Nightlord - Gladius",
    "adel": "Nightlord - Adel",
    "gnoster": "Nightlord - Gnoster",
    "maris": "Nightlord - Maris",
    "libra": "Nightlord - Libra",
    "fulghor": "Nightlord - Fulghor",
    "caligo": "Nightlord - Caligo",
    "heolstor": "Nightlord - Heolstor",
    "harmonia": "Nightlord - Harmonia",
    "straghess": "Nightlord - Straghess",
}
