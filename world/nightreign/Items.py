from typing import Dict, NamedTuple

from BaseClasses import ItemClassification

ITEM_ID_BASE = 839_100_000


class ItemSpec(NamedTuple):
    code: int
    classification: ItemClassification


item_table: Dict[str, ItemSpec] = {
    "Expedition Unlock - Tricephalos": ItemSpec(ITEM_ID_BASE + 1, ItemClassification.progression),
    "Murk Bundle": ItemSpec(ITEM_ID_BASE + 100, ItemClassification.filler),
    "Victory": ItemSpec(ITEM_ID_BASE + 900, ItemClassification.progression),
}

ITEM_NAME_TO_ID = {name: spec.code for name, spec in item_table.items()}
