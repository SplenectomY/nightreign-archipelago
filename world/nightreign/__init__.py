# Victory is locked to the goal. Heolstor is local unless opted into the pool.
# One base expedition unlock is precollected. Tricephalos is flag 189, not free.

from typing import Dict, List

from BaseClasses import Item, ItemClassification, Location, Region, Tutorial
from worlds.AutoWorld import WebWorld, World
from worlds.generic.Rules import set_rule

from .Items import (
    SHOP_ITEMS,
    BASE_NIGHTFARERS,
    BASE_UNLOCKS,
    DLC_NIGHTFARERS,
    DLC_UNLOCKS,
    ITEM_NAME_TO_ID,
    item_table,
)
from .Locations import (
    BASE_NIGHTLORDS,
    DLC_NIGHTLORDS,
    EVERDARK_DLC,
    EVERDARK_NIGHTLORDS,
    LOCATION_NAME_TO_ID,
    SHOP_LOCATIONS,
    SPECIFIC_LOCATION,
)
from .Options import NightreignOptions


class NightreignWeb(WebWorld):
    theme = "stone"
    tutorials = [
        Tutorial(
            "Setup Guide",
            "Generating and launching Elden Ring Nightreign for Archipelago.",
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

    def _nightlords(self):
        rows = list(BASE_NIGHTLORDS)
        if self.options.include_dlc:
            rows += DLC_NIGHTLORDS
        return rows

    def _everdark(self):
        if not self.options.include_everdark:
            return []
        rows = list(EVERDARK_NIGHTLORDS)
        if self.options.include_dlc:
            rows += EVERDARK_DLC
        return rows

    def _unlocks(self) -> List[str]:
        names = list(BASE_UNLOCKS)
        if self.options.include_dlc:
            names += DLC_UNLOCKS
        if self.options.heolstor_in_pool:
            names.append("Expedition Unlock - Heolstor")
        return names

    def _nightfarers(self) -> List[str]:
        names = list(BASE_NIGHTFARERS)
        if self.options.include_dlc:
            names += DLC_NIGHTFARERS
        return names

    def create_regions(self) -> None:
        menu = Region("Menu", self.player, self.multiworld)
        hold = Region("Roundtable Hold", self.player, self.multiworld)
        limveld = Region("Limveld", self.player, self.multiworld)

        hold_locs = {
            "Board unlock after first Nightlord": LOCATION_NAME_TO_ID[
                "Board unlock after first Nightlord"
            ],
            "Defeat Tutorial Margit": LOCATION_NAME_TO_ID["Defeat Tutorial Margit"],
            "Nightlord - Heolstor": LOCATION_NAME_TO_ID["Nightlord - Heolstor"],
            "Nightfarer Reserve": LOCATION_NAME_TO_ID["Nightfarer Reserve"],
        }
        if self.options.goal.current_key == "count":
            hold_locs["Goal - Nightlord Count"] = LOCATION_NAME_TO_ID["Goal - Nightlord Count"]
        for n in range(1, int(self.options.day1_boss_count) + 1):
            hold_locs[f"Day 1 Boss {n}"] = LOCATION_NAME_TO_ID[f"Day 1 Boss {n}"]
        for n in range(1, int(self.options.day2_boss_count) + 1):
            hold_locs[f"Day 2 Boss {n}"] = LOCATION_NAME_TO_ID[f"Day 2 Boss {n}"]
        if self.options.shop_checks.current_key != "none":
            for name in SHOP_LOCATIONS:
                hold_locs[name] = LOCATION_NAME_TO_ID[name]
        for i in range(1, 28):
            hold_locs[f"Murk {i}"] = LOCATION_NAME_TO_ID[f"Murk {i}"]
        for n in range(1, int(self.options.evergaol_count) + 1):
            hold_locs[f"Seal Evergaol {n}"] = LOCATION_NAME_TO_ID[f"Seal Evergaol {n}"]
        for n in range(1, int(self.options.tower_count) + 1):
            hold_locs[f"Open Magician Tower {n}"] = LOCATION_NAME_TO_ID[f"Open Magician Tower {n}"]
        for n in range(1, int(self.options.invader_count) + 1):
            hold_locs[f"Defeat Invaders {n}"] = LOCATION_NAME_TO_ID[f"Defeat Invaders {n}"]
        for loc_name, _item in self._nightlords() + self._everdark():
            hold_locs[loc_name] = LOCATION_NAME_TO_ID[loc_name]
        for name in ("Unlock Duchess", "Unlock Revenant"):
            hold_locs[name] = LOCATION_NAME_TO_ID[name]
        if self.options.include_dlc:
            for name in ("Unlock Scholar", "Unlock Undertaker"):
                hold_locs[name] = LOCATION_NAME_TO_ID[name]
        hold.add_locations(hold_locs, NightreignLocation)

        menu.connect(hold)
        hold.connect(limveld, "Commence Expedition")
        self.multiworld.regions += [menu, hold, limveld]

    def create_items(self) -> None:
        unlocks = self._unlocks()
        everdark = [item for _loc, item in self._everdark()]
        start = self.random.choice([n for n in unlocks if n != "Expedition Unlock - Heolstor"] or unlocks)
        self.push_precollected(self.create_item(start))
        roster = self._nightfarers()
        self.random.shuffle(roster)
        start_count = min(int(self.options.starting_nightfarers), len(roster))
        for name in roster[:start_count]:
            self.push_precollected(self.create_item(name))
        pool: List[Item] = [self.create_item(name) for name in unlocks if name != start]
        pool += [self.create_item(name) for name in everdark]
        pool += [self.create_item(name) for name in roster[start_count:]]
        shop = list(SHOP_ITEMS) if self.options.shop_checks.current_key != "none" else []
        self.random.shuffle(shop)
        low = int(self.options.starting_shop_min)
        high = int(self.options.starting_shop_max)
        if high < low:
            low, high = high, low
        start_shop = min(self.random.randint(low, high) if high else 0, len(shop))
        for name in shop[:start_shop]:
            self.push_precollected(self.create_item(name))
        pool += [self.create_item(name) for name in shop[start_shop:]]
        unfilled = sum(1 for loc in self.multiworld.get_locations(self.player) if not loc.item)
        # Victory is locked in set_rules, so leave one location empty.
        murk = ["Murk Purse"] * 8 + ["Murk Bundle"] * 8 + ["Murk Coffer"] * 6 + ["Murk Chest"] * 3 + ["Murk Hoard"] * 2
        for name in murk:
            pool.append(self.create_item(name))
        while len(pool) < unfilled - 1:
            pool.append(self.create_item("Murk Purse"))
        self.multiworld.itempool += pool

    def create_item(self, name: str) -> Item:
        data = item_table[name]
        return NightreignItem(name, data.classification, data.code, self.player)

    def set_rules(self) -> None:
        player = self.player
        defeats = [loc for loc, _item in self._nightlords() if loc != "Nightlord - Heolstor"]
        need = int(self.options.heolstor_unlock_count)
        in_pool = bool(self.options.heolstor_in_pool)

        def heolstor_gate(state) -> bool:
            if in_pool:
                return state.has("Expedition Unlock - Heolstor", player)
            return sum(state.can_reach(name, "Location", player) for name in defeats) >= need

        for loc_name, item_name in self._nightlords() + self._everdark():
            set_rule(
                self.get_location(loc_name),
                lambda state, item_name=item_name: state.has(item_name, player),
            )
        quests = {
            "Unlock Duchess": "Nightfarer - Duchess",
            "Unlock Revenant": "Nightfarer - Revenant",
        }
        if self.options.include_dlc:
            quests["Unlock Scholar"] = "Nightfarer - Scholar"
            quests["Unlock Undertaker"] = "Nightfarer - Undertaker"
        for loc_name, item_name in quests.items():
            if loc_name == "Unlock Revenant" and self.options.shop_checks.current_key != "none":
                set_rule(
                    self.get_location(loc_name),
                    lambda state, item_name=item_name: state.has(item_name, player)
                    and state.has("Shop - Besmirched Frame", player),
                )
            else:
                set_rule(
                    self.get_location(loc_name),
                    lambda state, item_name=item_name: state.has(item_name, player),
                )
        set_rule(self.get_location("Nightlord - Heolstor"), heolstor_gate)

        goal = self.options.goal.current_key
        if goal == "count":
            names = [loc for loc, _item in self._nightlords()] + ["Nightlord - Heolstor"]
            count_need = int(self.options.nightlord_count)

            def enough(state) -> bool:
                return sum(state.can_reach(name, "Location", player) for name in names) >= count_need

            goal_loc = self.get_location("Goal - Nightlord Count")
            set_rule(goal_loc, enough)
        elif goal == "specific":
            goal_loc = self.get_location(SPECIFIC_LOCATION[self.options.specific_nightlord.current_key])
        else:
            goal_loc = self.get_location("Nightlord - Heolstor")

        goal_loc.place_locked_item(self.create_item("Victory"))
        self.multiworld.completion_condition[player] = lambda state: state.has("Victory", player)

    def fill_slot_data(self) -> Dict[str, object]:
        return {
            "goal": self.options.goal.current_key,
            "nightlord_count": int(self.options.nightlord_count),
            "specific_nightlord": self.options.specific_nightlord.current_key,
            "heolstor_in_pool": bool(self.options.heolstor_in_pool),
            "heolstor_unlock_count": int(self.options.heolstor_unlock_count),
            "include_everdark": bool(self.options.include_everdark),
            "include_dlc": bool(self.options.include_dlc),
            "starting_nightfarers": int(self.options.starting_nightfarers),
            "shop_checks": self.options.shop_checks.current_key,
        }


class NightreignItem(Item):
    game = "Elden Ring Nightreign"


class NightreignLocation(Location):
    game = "Elden Ring Nightreign"
