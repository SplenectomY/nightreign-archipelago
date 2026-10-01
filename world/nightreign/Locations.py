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
    "Everdark - Gladius": LOCATION_ID_BASE + 101,
    "Everdark - Adel": LOCATION_ID_BASE + 102,
    "Everdark - Gnoster": LOCATION_ID_BASE + 103,
    "Everdark - Maris": LOCATION_ID_BASE + 104,
    "Everdark - Libra": LOCATION_ID_BASE + 105,
    "Everdark - Fulghor": LOCATION_ID_BASE + 106,
    "Everdark - Caligo": LOCATION_ID_BASE + 107,
    "Everdark - Harmonia": LOCATION_ID_BASE + 108,
    "Goal - Nightlord Count": LOCATION_ID_BASE + 200,
    "Shop - Polite Bow": LOCATION_ID_BASE + 501,
    "Shop - Strength": LOCATION_ID_BASE + 503,
    "Shop - Warm Welcome": LOCATION_ID_BASE + 504,
    "Shop - Delicate Burning Scene": LOCATION_ID_BASE + 505,
    "Shop - Heartening Cry": LOCATION_ID_BASE + 506,
    "Shop - Calm Down": LOCATION_ID_BASE + 507,
    "Nightfarer Reserve": LOCATION_ID_BASE + 600,
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

# Defeat location, Everdark unlock item. These do not count toward the Heolstor gate.
EVERDARK_NIGHTLORDS: List[Tuple[str, str]] = [
    ("Everdark - Gladius", "Everdark Unlock - Gladius"),
    ("Everdark - Adel", "Everdark Unlock - Adel"),
    ("Everdark - Gnoster", "Everdark Unlock - Gnoster"),
    ("Everdark - Maris", "Everdark Unlock - Maris"),
    ("Everdark - Libra", "Everdark Unlock - Libra"),
    ("Everdark - Fulghor", "Everdark Unlock - Fulghor"),
    ("Everdark - Caligo", "Everdark Unlock - Caligo"),
]

EVERDARK_DLC: List[Tuple[str, str]] = [
    ("Everdark - Harmonia", "Everdark Unlock - Harmonia"),
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
