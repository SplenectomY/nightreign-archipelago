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
    "Murk Bundle": ItemSpec(ITEM_ID_BASE + 100, ItemClassification.filler),
    "Victory": ItemSpec(ITEM_ID_BASE + 900, ItemClassification.progression),
}

ITEM_NAME_TO_ID = {name: spec.code for name, spec in item_table.items()}

# Base seven. Heolstor is omitted from the pool unless heolstor_in_pool.
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
