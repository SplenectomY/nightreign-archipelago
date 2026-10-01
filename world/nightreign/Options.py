from dataclasses import dataclass

from Options import Choice, Range, Toggle, PerGameCommonOptions


class Goal(Choice):
    """How this slot is completed."""
    display_name = "Goal"
    option_heolstor = 0
    option_count = 1
    option_specific = 2
    default = 0


class NightlordCount(Range):
    """Used when goal is count. Number of distinct Nightlords to defeat."""
    display_name = "Nightlord Count"
    range_start = 1
    range_end = 10
    default = 4


class SpecificNightlord(Choice):
    """Used when goal is specific. Default Heolstor."""
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


class ShopChecks(Choice):
    """Which Hold purchases become locations."""
    display_name = "Shop Checks"
    option_none = 0
    option_unique_only = 1
    default = 1


class StartingNightfarers(Range):
    """How many Nightfarers are unlocked at the start. Chosen at random from the enabled roster."""
    display_name = "Starting Nightfarers"
    range_start = 1
    range_end = 10
    default = 1


class DeathLink(Toggle):
    """Not implemented in Phase 0."""
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
    shop_checks: ShopChecks
    starting_nightfarers: StartingNightfarers
    death_link: DeathLink
