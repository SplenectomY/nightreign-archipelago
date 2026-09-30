# Phase 0 world. Generates. Logic is a stub so we can talk to a real MultiServer.

from typing import Dict, List

from BaseClasses import Item, ItemClassification, Location, MultiWorld, Region, Tutorial
from worlds.AutoWorld import WebWorld, World

from .Items import ITEM_NAME_TO_ID, item_table
from .Locations import LOCATION_NAME_TO_ID
from .Options import NightreignOptions


class NightreignWeb(WebWorld):
    theme = "stone"
    tutorials = [
        Tutorial(
            "Setup Guide",
            "Generating and launching Elden Ring Nightreign for Archipelago (Phase 0).",
            "en",
            "setup_en.md",
            "setup/en",
            ["SplenectomY"],
        )
    ]


class NightreignWorld(World):
    """Elden Ring Nightreign"""

    game = "Elden Ring Nightreign"
    author = "SplenectomY"
    web = NightreignWeb()
    options_dataclass = NightreignOptions
    options: NightreignOptions

    item_name_to_id = ITEM_NAME_TO_ID
    location_name_to_id = LOCATION_NAME_TO_ID

    ut_can_gen_without_yaml = True

    def create_regions(self) -> None:
        menu = Region("Menu", self.player, self.multiworld)
        hold = Region("Roundtable Hold", self.player, self.multiworld)
        limveld = Region("Limveld", self.player, self.multiworld)

        hold.add_locations(
            {
                "Nightlord - Gladius": LOCATION_NAME_TO_ID["Nightlord - Gladius"],
                "Nightlord - Adel": LOCATION_NAME_TO_ID["Nightlord - Adel"],
                "Nightlord - Gnoster": LOCATION_NAME_TO_ID["Nightlord - Gnoster"],
                "Board unlock after first Nightlord": LOCATION_NAME_TO_ID[
                    "Board unlock after first Nightlord"
                ],
                "Shop - Phase 0 Probe": LOCATION_NAME_TO_ID["Shop - Phase 0 Probe"],
            },
            NightreignLocation,
        )
        limveld.add_locations(
            {
                "Victory": LOCATION_NAME_TO_ID["Victory"],
            },
            NightreignLocation,
        )

        menu.connect(hold)
        hold.connect(limveld, "Commence Expedition")
        self.multiworld.regions += [menu, hold, limveld]

    def create_items(self) -> None:
        pool: List[Item] = [
            self.create_item("Expedition Unlock - Tricephalos"),
            self.create_item("Expedition Unlock - Heolstor"),
            self.create_item("Expedition Unlock - Harmonia"),
            self.create_item("Expedition Unlock - Straghess"),
            self.create_item("Murk Bundle"),
            self.create_item("Victory"),
        ]
        self.multiworld.itempool += pool

    def create_item(self, name: str) -> Item:
        data = item_table[name]
        return NightreignItem(name, data.classification, data.code, self.player)

    def set_rules(self) -> None:
        self.multiworld.completion_condition[self.player] = lambda state: state.has(
            "Victory", self.player
        )

    def fill_slot_data(self) -> Dict[str, object]:
        return {
            "goal": self.options.goal.current_key,
            "nightlord_count": int(self.options.nightlord_count),
            "specific_nightlord": self.options.specific_nightlord.current_key,
            "include_everdark": bool(self.options.include_everdark),
            "include_dlc": bool(self.options.include_dlc),
            "starting_nightfarers": int(self.options.starting_nightfarers),
            "shop_checks": self.options.shop_checks.current_key,
        }


class NightreignItem(Item):
    game = "Elden Ring Nightreign"


class NightreignLocation(Location):
    game = "Elden Ring Nightreign"
