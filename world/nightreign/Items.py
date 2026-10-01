from typing import Dict, NamedTuple

from BaseClasses import ItemClassification

ITEM_ID_BASE = 839_100_000


class ItemSpec(NamedTuple):
    code: int
    classification: ItemClassification


item_table: Dict[str, ItemSpec] = {
    "Expedition Unlock - Tricephalos": ItemSpec(ITEM_ID_BASE + 1, ItemClassification.progression),
    "Expedition Unlock - Adel": ItemSpec(ITEM_ID_BASE + 2, ItemClassification.progression),
    "Expedition Unlock - Gnoster": ItemSpec(ITEM_ID_BASE + 3, ItemClassification.progression),
    "Expedition Unlock - Maris": ItemSpec(ITEM_ID_BASE + 4, ItemClassification.progression),
    "Expedition Unlock - Libra": ItemSpec(ITEM_ID_BASE + 5, ItemClassification.progression),
    "Expedition Unlock - Fulghor": ItemSpec(ITEM_ID_BASE + 6, ItemClassification.progression),
    "Expedition Unlock - Caligo": ItemSpec(ITEM_ID_BASE + 7, ItemClassification.progression),
    "Expedition Unlock - Heolstor": ItemSpec(ITEM_ID_BASE + 8, ItemClassification.progression),
    "Expedition Unlock - Harmonia": ItemSpec(ITEM_ID_BASE + 9, ItemClassification.progression),
    "Expedition Unlock - Straghess": ItemSpec(ITEM_ID_BASE + 10, ItemClassification.progression),
    "Everdark Unlock - Gladius": ItemSpec(ITEM_ID_BASE + 111, ItemClassification.progression),
    "Everdark Unlock - Adel": ItemSpec(ITEM_ID_BASE + 112, ItemClassification.progression),
    "Everdark Unlock - Gnoster": ItemSpec(ITEM_ID_BASE + 113, ItemClassification.progression),
    "Everdark Unlock - Maris": ItemSpec(ITEM_ID_BASE + 114, ItemClassification.progression),
    "Everdark Unlock - Libra": ItemSpec(ITEM_ID_BASE + 115, ItemClassification.progression),
    "Everdark Unlock - Fulghor": ItemSpec(ITEM_ID_BASE + 116, ItemClassification.progression),
    "Everdark Unlock - Caligo": ItemSpec(ITEM_ID_BASE + 117, ItemClassification.progression),
    "Everdark Unlock - Harmonia": ItemSpec(ITEM_ID_BASE + 118, ItemClassification.progression),
    "Nightfarer - Wylder": ItemSpec(ITEM_ID_BASE + 301, ItemClassification.progression),
    "Nightfarer - Duchess": ItemSpec(ITEM_ID_BASE + 302, ItemClassification.progression),
    "Nightfarer - Raider": ItemSpec(ITEM_ID_BASE + 303, ItemClassification.progression),
    "Nightfarer - Executor": ItemSpec(ITEM_ID_BASE + 304, ItemClassification.progression),
    "Nightfarer - Recluse": ItemSpec(ITEM_ID_BASE + 305, ItemClassification.progression),
    "Nightfarer - Ironeye": ItemSpec(ITEM_ID_BASE + 306, ItemClassification.progression),
    "Nightfarer - Guardian": ItemSpec(ITEM_ID_BASE + 307, ItemClassification.progression),
    "Nightfarer - Revenant": ItemSpec(ITEM_ID_BASE + 308, ItemClassification.progression),
    "Nightfarer - Scholar": ItemSpec(ITEM_ID_BASE + 309, ItemClassification.progression),
    "Nightfarer - Undertaker": ItemSpec(ITEM_ID_BASE + 310, ItemClassification.progression),
    "Murk Bundle": ItemSpec(ITEM_ID_BASE + 100, ItemClassification.filler),
    "Victory": ItemSpec(ITEM_ID_BASE + 900, ItemClassification.progression),
}

ITEM_NAME_TO_ID = {name: spec.code for name, spec in item_table.items()}

# Base seven. Tricephalos is flag 189 in the regulation override.
# Heolstor is omitted from the pool unless heolstor_in_pool.
BASE_UNLOCKS = [
    "Expedition Unlock - Tricephalos",
    "Expedition Unlock - Adel",
    "Expedition Unlock - Gnoster",
    "Expedition Unlock - Maris",
    "Expedition Unlock - Libra",
    "Expedition Unlock - Fulghor",
    "Expedition Unlock - Caligo",
]

DLC_UNLOCKS = [
    "Expedition Unlock - Harmonia",
    "Expedition Unlock - Straghess",
]

EVERDARK_UNLOCKS = [
    "Everdark Unlock - Gladius",
    "Everdark Unlock - Adel",
    "Everdark Unlock - Gnoster",
    "Everdark Unlock - Maris",
    "Everdark Unlock - Libra",
    "Everdark Unlock - Fulghor",
    "Everdark Unlock - Caligo",
    "Everdark Unlock - Harmonia",
]

BASE_NIGHTFARERS = [
    "Nightfarer - Wylder",
    "Nightfarer - Duchess",
    "Nightfarer - Raider",
    "Nightfarer - Executor",
    "Nightfarer - Recluse",
    "Nightfarer - Ironeye",
    "Nightfarer - Guardian",
    "Nightfarer - Revenant",
]

DLC_NIGHTFARERS = [
    "Nightfarer - Scholar",
    "Nightfarer - Undertaker",
]
