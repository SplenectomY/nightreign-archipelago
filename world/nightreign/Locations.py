from typing import Dict

LOCATION_ID_BASE = 839_000_000

# Shop ids match flags.toml and CSFD4VirtualMemoryFlag::GetFlag on 1.3.3.0.
# Confirmed 2026-09-30: 67600/67640/67650/67700 already set on a bought save,
# 67670 flipped 0->1 on buying Calm Down. Slab math does not see group 67.
LOCATION_NAME_TO_ID: Dict[str, int] = {
    "Nightlord - Gladius": LOCATION_ID_BASE + 1,
    "Nightlord - Adel": LOCATION_ID_BASE + 2,
    "Nightlord - Gnoster": LOCATION_ID_BASE + 3,
    "Board unlock after first Nightlord": LOCATION_ID_BASE + 100,
    "Shop - Polite Bow": LOCATION_ID_BASE + 501,
    "Shop - Strength": LOCATION_ID_BASE + 503,
    "Shop - Warm Welcome": LOCATION_ID_BASE + 504,
    "Shop - Delicate Burning Scene": LOCATION_ID_BASE + 505,
    "Shop - Heartening Cry": LOCATION_ID_BASE + 506,
    "Shop - Calm Down": LOCATION_ID_BASE + 507,
    "Victory": LOCATION_ID_BASE + 900,
}

SHOP_LOCATIONS = [
    "Shop - Polite Bow",
    "Shop - Strength",
    "Shop - Warm Welcome",
    "Shop - Delicate Burning Scene",
    "Shop - Heartening Cry",
    "Shop - Calm Down",
]

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
