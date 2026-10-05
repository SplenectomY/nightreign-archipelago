from dataclasses import dataclass

from Options import Choice, OptionSet, Range, Toggle, PerGameCommonOptions


class Goal(Choice):
    """How this slot is completed.
    heolstor: defeat Heolstor. Default.
    count: defeat nightlord_count distinct Nightlords.
    specific: defeat the Nightlord named by specific_nightlord.
    """
    display_name = "Goal"
    option_heolstor = 0
    option_count = 1
    option_specific = 2
    default = 0


class NightlordCount(Range):
    """Used when goal is count. Distinct Nightlord defeats required. 1 to 10. Default 4."""
    display_name = "Nightlord Count"
    range_start = 1
    range_end = 10
    default = 4


class SpecificNightlord(Choice):
    """Used when goal is specific.
    gladius, adel, gnoster, maris, libra, fulghor, caligo, heolstor, harmonia, straghess,
    everdark_gladius, everdark_adel, everdark_gnoster, everdark_maris, everdark_libra,
    everdark_fulghor, everdark_caligo, everdark_harmonia.
    harmonia, straghess, and everdark_harmonia need include_dlc.
    An everdark goal still creates that defeat location if include_everdark is off.
    """
    display_name = "Specific Nightlord"
    option_gladius = 0
    option_adel = 1
    option_gnoster = 2
    option_maris = 3
    option_libra = 4
    option_fulghor = 5
    option_caligo = 6
    option_heolstor = 7
    option_harmonia = 8
    option_straghess = 9
    option_everdark_gladius = 10
    option_everdark_adel = 11
    option_everdark_gnoster = 12
    option_everdark_maris = 13
    option_everdark_libra = 14
    option_everdark_fulghor = 15
    option_everdark_caligo = 16
    option_everdark_harmonia = 17
    default = 7


class HeolstorInPool(Toggle):
    """If false, Heolstor is not a shuffled unlock. The slot opens him after heolstor_unlock_count other Nightlord defeats."""
    display_name = "Heolstor In Pool"
    default = False


class HeolstorUnlockCount(Range):
    """Other Nightlord defeats required before this slot can open Heolstor. Ignored when Heolstor is in the pool."""
    display_name = "Heolstor Unlock Count"
    range_start = 1
    range_end = 9
    default = 4


class IncludeEverdark(Toggle):
    """Add Everdark Sovereign locations. Client will attempt an offline unlock."""
    display_name = "Include Everdark"
    default = True


class IncludeDlc(Toggle):
    """Include Forsaken Hollows Nightlords and Nightfarers."""
    display_name = "Include DLC"
    default = True


class TutorialMargit(Toggle):
    """Defeat Tutorial Margit is a location. Flag 6012."""
    display_name = "Tutorial Margit"
    default = True


class ShopChecks(Choice):
    """Which Hold purchases become locations.
    none: no shop locations.
    unique_only: Small Jar Bazaar rows. Finding the item stocks it. Buying it is the check.
    """
    display_name = "Shop Checks"
    option_none = 0
    option_unique_only = 1
    default = 1


class StartingNightlords(OptionSet):
    """Nightlords eligible to be the starting expedition. One is chosen at random.
    Possible values: gladius, adel, gnoster, maris, libra, fulghor, caligo, heolstor, harmonia, straghess,
    everdark_gladius, everdark_adel, everdark_gnoster, everdark_maris, everdark_libra, everdark_fulghor,
    everdark_caligo, everdark_harmonia.
    gladius is Tricephalos. harmonia and straghess are the DLC bosses. Default excludes everdarks, Heolstor, and DLC.
    """
    display_name = "Starting Nightlords"
    valid_keys = {
        "gladius", "adel", "gnoster", "maris", "libra", "fulghor", "caligo", "heolstor",
        "harmonia", "straghess",
        "everdark_gladius", "everdark_adel", "everdark_gnoster", "everdark_maris",
        "everdark_libra", "everdark_fulghor", "everdark_caligo", "everdark_harmonia",
    }
    default = {"gladius", "adel", "gnoster", "maris", "libra", "fulghor", "caligo"}


class StartingNightfarers(Range):
    """How many Nightfarers are unlocked at the start. Chosen at random from the enabled roster."""
    display_name = "Starting Nightfarers"
    range_start = 1
    range_end = 10
    default = 1


class StartingShopMin(Range):
    """Minimum shop items unlocked at the start. Rolled once per generation."""
    display_name = "Starting Shop Min"
    range_start = 0
    range_end = 40
    default = 3


class StartingShopMax(Range):
    """Maximum shop items unlocked at the start. Rolled once per generation."""
    display_name = "Starting Shop Max"
    range_start = 0
    range_end = 40
    default = 6


class Day1BossCount(Range):
    """How many Day 1 night-boss defeats become locations."""
    display_name = "Day 1 Boss Count"
    range_start = 1
    range_end = 50
    default = 30


class Day2BossCount(Range):
    """How many Day 2 night-boss defeats become locations."""
    display_name = "Day 2 Boss Count"
    range_start = 1
    range_end = 30
    default = 10


class EvergaolCount(Range):
    """How many evergaol seals become locations. The flag toggles, so both edges count."""
    display_name = "Evergaol Count"
    range_start = 1
    range_end = 40
    default = 20


class TowerCount(Range):
    """How many magician tower openings become locations. The flag toggles, so both edges count."""
    display_name = "Tower Count"
    range_start = 1
    range_end = 30
    default = 10


class InvaderCount(Range):
    """How many invader defeats become locations. Flag 8155 toggles, so both edges count. Counts after 5 require Deep of Night."""
    display_name = "Invader Count"
    range_start = 0
    range_end = 50
    default = 20


class BuriedTreasureCount(Range):
    """How many buried treasure openings become locations. Flags 8120-8131 rise to 1 and clear on exit, so only the rise counts."""
    display_name = "Buried Treasure Count"
    range_start = 0
    range_end = 80
    default = 40


class FlaskCount(Range):
    """How many crimson flask max-use increases become locations. Flag 9041 rises when an expedition starts, so the check is the later 1 to 0. Only read during an expedition."""
    display_name = "Flask Count"
    range_start = 0
    range_end = 60
    default = 30


class MurkPurse(Range):
    """Murk granted by each Murk Purse. 1 to 100000. Default 150."""
    display_name = "Murk Purse"
    range_start = 1
    range_end = 100000
    default = 150


class MurkBundle(Range):
    """Murk granted by each Murk Bundle. 1 to 100000. Default 300."""
    display_name = "Murk Bundle"
    range_start = 1
    range_end = 100000
    default = 300


class MurkCoffer(Range):
    """Murk granted by each Murk Coffer. 1 to 100000. Default 500."""
    display_name = "Murk Coffer"
    range_start = 1
    range_end = 100000
    default = 500


class MurkChest(Range):
    """Murk granted by each Murk Chest. 1 to 100000. Default 750."""
    display_name = "Murk Chest"
    range_start = 1
    range_end = 100000
    default = 750


class MurkHoard(Range):
    """Murk granted by each Murk Hoard. 1 to 100000. Default 1300."""
    display_name = "Murk Hoard"
    range_start = 1
    range_end = 100000
    default = 1300


class DeathLink(Toggle):
    """Not implemented. Leave off. A death does not kill other players."""
    display_name = "Death Link"
    default = False


@dataclass
class NightreignOptions(PerGameCommonOptions):
    goal: Goal
    nightlord_count: NightlordCount
    specific_nightlord: SpecificNightlord
    heolstor_in_pool: HeolstorInPool
    heolstor_unlock_count: HeolstorUnlockCount
    include_everdark: IncludeEverdark
    include_dlc: IncludeDlc
    tutorial_margit: TutorialMargit
    shop_checks: ShopChecks
    starting_nightlords: StartingNightlords
    starting_nightfarers: StartingNightfarers
    starting_shop_min: StartingShopMin
    starting_shop_min: StartingShopMin
    starting_shop_max: StartingShopMax
    day1_boss_count: Day1BossCount
    day2_boss_count: Day2BossCount
    evergaol_count: EvergaolCount
    tower_count: TowerCount
    invader_count: InvaderCount
    buried_treasure_count: BuriedTreasureCount
    flask_count: FlaskCount
    murk_purse: MurkPurse
    murk_bundle: MurkBundle
    murk_coffer: MurkCoffer
    murk_chest: MurkChest
    murk_hoard: MurkHoard
    death_link: DeathLink
