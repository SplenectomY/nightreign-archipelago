# Locations and unlocks

What the current world generates. The template is `players/Nightreign.yaml`. A check is sent when the flag flips, even if the generator had not reached that sphere yet.

## Shipped

- One random expedition and one random Nightfarer at start. Tricephalos is not free. Heolstor stays local unless `heolstor_in_pool` is set, then opens after `heolstor_unlock_count` other Nightlord defeats (template: 4).
- Nightlord and Everdark defeat checks. A defeat counts only if that expedition is unlocked. Everdarks do not count toward the Heolstor gate.
- Small Jar Bazaar and Garb Shop. Finding the item stocks the row. Buying it is the check. Template starts 3 to 6 bazaar rows unlocked. `shop_checks: none` removes shop locations and shop items from the pool.
- Character unlock checks: Duchess, Revenant, and with DLC on, Scholar and Undertaker. These do not require owning that Nightfarer. Revenant's check requires `Shop - Besmirched Frame` while shop checks are in the pool.
- Counters: Day 1 bosses, Day 2 bosses, evergaol seals, magician towers, invaders, buried treasure maps, flask max-use increases. Tutorial Margit is optional and on by default.
- Murk Purse, Bundle, Coffer, Chest, and Hoard fill leftover locations. Amounts are yaml values.
- Starting Runes + 1000 / + 5000 / + 10000. A pile received in an expedition is granted immediately and banked. Every later run pays the full bank 5 seconds after day 1 starts.
- Goal: defeat Heolstor, defeat `nightlord_count` Nightlords, or defeat one specific Nightlord, including an Everdark.
- Death Link, off by default. `instant`, `percent`, or `dice`. Incoming links are ignored outside an expedition.
- `always_wylder_in_tutorial`, on by default. Flag 9801 forces the body back to Wylder. The Hold still uses the granted Nightfarer.

## Planned

1. Fix the Wylder model when the active Nightfarer is forced off Wylder.
2. Optional remembrance quest checks.
3. DLC map checks.
4. Collector Signboard currency items and checks.

## Goals

| `goal` | Completes when |
|---|---|
| `heolstor` | Nightlord - Heolstor is checked. Default. |
| `count` | `nightlord_count` distinct Nightlord locations are checked. Default 4. Everdarks do not count. |
| `specific` | `specific_nightlord` is checked. Includes Everdark choices. |

## Expedition unlocks (items)

One is precollected from `starting_nightlords`. Default pool: Gladius, Adel, Gnoster, Maris, Libra, Fulghor, Caligo. Heolstor, Harmonia, Straghess, Everdarks, and Deep of Night are excluded from that draw.

| Item | Opens |
|---|---|
| Expedition Unlock - Tricephalos | Nightlord - Gladius |
| Expedition Unlock - Adel | Nightlord - Adel |
| Expedition Unlock - Gnoster | Nightlord - Gnoster |
| Expedition Unlock - Maris | Nightlord - Maris |
| Expedition Unlock - Libra | Nightlord - Libra |
| Expedition Unlock - Fulghor | Nightlord - Fulghor |
| Expedition Unlock - Caligo | Nightlord - Caligo |
| Expedition Unlock - Heolstor | Nightlord - Heolstor, only if `heolstor_in_pool` |
| Expedition Unlock - Harmonia | Nightlord - Harmonia, DLC |
| Expedition Unlock - Straghess | Nightlord - Straghess, DLC |
| Expedition Unlock - Deep of Night | Deep of Night. Invader checks after 5 require it. |
| Everdark Unlock - Gladius through Caligo | Matching Everdark, if `include_everdark` |
| Everdark Unlock - Harmonia | Everdark Harmonia, DLC and everdark |

## Nightlord locations

Base: Gladius, Adel, Gnoster, Maris, Libra, Fulghor, Caligo. DLC adds Harmonia and Straghess. Heolstor is always a location. Everdark adds Gladius, Adel, Gnoster, Maris, Libra, Fulghor, Caligo, and Harmonia when those toggles are on.

A Nightlord location is in logic once its expedition unlock is found. Heolstor is in logic after the local defeat count, or after its unlock item if it is in the pool.

## Nightfarers (items)

Wylder, Guardian, Ironeye, Duchess, Raider, Recluse, Executor, Revenant. DLC adds Scholar and Undertaker. Template grants 1 at random. The rest are items.

## Character unlock locations

| Location | In the pool when | Logic |
|---|---|---|
| Unlock Duchess | always | no Nightfarer requirement |
| Unlock Revenant | always | Besmirched Frame if shop checks are on |
| Unlock Scholar | `include_dlc` | no Nightfarer requirement |
| Unlock Undertaker | `include_dlc` | no Nightfarer requirement |

## Shop and garb locations

Only when `shop_checks` is `unique_only`. Each purchase requires the matching unlock item. Purchases are spread across spheres gated by Nightlord defeats. Garb also requires that Nightfarer and two base Nightlord defeats. Harmonia, Straghess, and Everdark Harmonia do not count toward that two.

Bazaar groups: scene relics, goblets and grails, gestures, prattling pates. Garb is four outfits for each base Nightfarer, plus Scholar and Undertaker when DLC is on. Names are `Shop - …` and `Garb - …` in the spoiler.

Template `starting_shop_min` / `starting_shop_max` is 3 to 6. Those rows start stocked and are not pool items.

## Counter locations

Template counts. The generator spreads them across spheres, one band per Nightlord gate.

| Location | Template count | Range | Counts when |
|---|---|---|---|
| Day 1 Boss N | 30 | 1–60 | Night 1 boss dies, flag 7502 |
| Day 2 Boss N | 10 | 1–60 | Night 2 boss dies, flag 7507 |
| Seal Evergaol N | 20 | 0–50 | Evergaol sealed, flag 8145 |
| Open Magician Tower N | 30 in the template, 10 if omitted | 1–50 | Tower opened, flag 8140 |
| Defeat Invaders N | 20 | 0–50 | Invader dies, flag 8155. 6 and up need Deep of Night |
| Buried Treasure Map N | 40 | 0–100 | Map point opens, flags 8120–8131 |
| Flask Max Uses Increased N | 30 | 0–60 | Flag 9041 rises or falls. Edges within 7 seconds of Day 1 start are ignored |
| Defeat Tutorial Margit | 1 | toggle, default on | Flag 6012 |

Leaving an expedition clears the toggling flags. Those clears do not add another check. Flask also ignores any edge within 7 seconds of the Day 1 start flag.

## Other locations

| Location | Notes |
|---|---|
| Board unlock after first Nightlord | Sent when the board flag rises. Does not unlock the rest of the board by itself. |

## Filler and useful items

| Item | Template amount | Notes |
|---|---|---|
| Murk Purse | 150 | filler, placed only to fill extra locations |
| Murk Bundle | 300 | filler |
| Murk Coffer | 500 | filler |
| Murk Chest | 750 | filler |
| Murk Hoard | 1300 | filler |
| Starting Runes + 1000 | 20 copies | useful, any sphere |
| Starting Runes + 5000 | 10 copies | useful, after one Nightlord defeat |
| Starting Runes + 10000 | 5 copies | useful, back half of spheres |

Murk is granted in Roundtable Hold. Starting runes pay on receipt during an expedition, and the full bank pays 5 seconds after flag 7500 on every run.
