# Locations and unlocks

What 0.5.0 ships, then what is still planned.

## Shipped

- Nightlord defeats, including DLC, and Everdark defeats. A defeat counts only after that expedition is unlocked.
- Expedition unlocks, one random at start. Tricephalos is gated like the others. Heolstor is local unless `heolstor_in_pool` is set.
- Nightfarer unlocks, one random at start.
- Small Jar Bazaar rows. Finding the item stocks it. Buying it is the check. Three to six start unlocked.
- Murk purses, bundles, coffers, chests, and hoards.
- Starting Runes + 1000 / + 5000 / + 10000. Useful. Larger piles are kept out of early spheres. The full total is granted 5 seconds after day 1 starts, on every run.
- Day 1 boss, Day 2 boss, evergaol, and magician tower counters. First credit is 1.
- Goal: Nightlord count, or a named Nightlord. Default Heolstor.

## Planned

1. Tutorial Margit as an optional check.
2. Garb shop checks.
3. Optional remembrance quest checks.
4. More overworld checks, where a stable flag exists.
5. Walking model update when the active Nightfarer is forced off Wylder.

The tables below are the working catalog. A row with no flag is not a check yet.

## How to read this

| Column | Meaning |
|---|---|
| **Kind** | `location` = a check. `item` = something the multiworld can hand you. |
| **Default** | In the shipped yaml when that content toggle is on. |
| **Gate** | What logic will require before the check is in logic (not before the client will send it — kills still count immediately). |

Client still sends a check the moment the boss dies or the SKU is bought, even if you were out of logic.

---

## A. Named Nightlords (locations)

Always unique. These are the only named boss checks.

| Location | Expedition | Default | Gate | Notes |
|---|---|---|---|---|
| Nightlord - Gladius | Tricephalos | yes | starting board / unlock item | Phase 0 target |
| Nightlord - Adel | Gaping Jaw | yes | board unlock | |
| Nightlord - Gnoster | Sentient Pest | yes | board unlock | |
| Nightlord - Maris | Augur | yes | board unlock | |
| Nightlord - Libra | Equilibrious Beast | yes | board unlock | |
| Nightlord - Fulghor | Darkdrift Knight | yes | board unlock | |
| Nightlord - Caligo | Fissure in the Fog | yes | board unlock | |
| Nightlord - Heolstor | Night Aspect | yes | `goal: heolstor` plus four Nightlord items or equivalent | Final in vanilla |
| Nightlord - Harmonia | Balancers | `include_dlc` | DLC + two Nightlords + DLC Nightfarers in vanilla; we replace that with items | Forsaken Hollows |
| Nightlord - Straghess | Dreglord | `include_dlc` | DLC board unlock | Forsaken Hollows |

**Count (default, DLC on):** 10.

Vanilla first-clear relics (`Night of the Beast`, `Night of the Baron`, …) become **items**, not extra locations. The Nightlord location is the check.

---

## B. Everdark Sovereigns (locations)

Prefix `Everdark - `. Requires `include_everdark`. Client ships an offline unlock unless that proves impossible.

| Location | Pair of | Default if toggle on | Gate |
|---|---|---|---|
| Everdark - Gladius | Nightlord - Gladius | yes | that Nightlord already checked |
| Everdark - Adel | Adel | yes | same |
| Everdark - Gnoster | Gnoster | yes | same |
| Everdark - Maris | Maris | yes | same |
| Everdark - Libra | Libra | yes | same |
| Everdark - Fulghor | Fulghor | yes | same |
| Everdark - Caligo | Caligo | yes | same |
| Everdark - Heolstor | Heolstor | **unknown** | record if the fight exists |
| Everdark - Harmonia | Harmonia | `include_dlc` | record if present |
| Everdark - Straghess | Straghess | `include_dlc` | record if present |

**Count (base everdarks known):** 7. Heolstor / DLC everdarks TBD.

`Dark Night of the …` Signboard relics stay shop locations or items, not a second check on the same kill.

---

## C. Tiered Night 1 / Night 2 (locations)

One cell per expedition per night. The inner boss name is ignored.

| Location pattern | Per expedition | Default |
|---|---|---|
| `{Expedition} Night 1 Boss` | first night boss of that expedition | yes |
| `{Expedition} Night 2 Boss` | second night boss | yes |

Expedition names used in the location string:

- Tricephalos
- Gaping Jaw
- Sentient Pest
- Augur
- Equilibrious Beast
- Darkdrift Knight
- Fissure in the Fog
- Night Aspect
- Balancers (`include_dlc`)
- Dreglord (`include_dlc`)

Examples: `Tricephalos Night 1 Boss`, `Night Aspect Night 2 Boss`.

**Count:** 16 base + 4 DLC = 20.

A wipe after the kill still keeps the check.

---

## D. Counter locations

Unique kills only. A boss is tagged with **one** family. Nightlords never increment these.

| Location | Threshold | Default |
|---|---|---|
| World Bosses (1) | 1 | yes |
| World Bosses (3) | 3 | yes |
| World Bosses (6) | 6 | yes |
| Church Bosses (1) | 1 | yes |
| Church Bosses (3) | 3 | yes |
| Fortress Bosses (1) | 1 | yes |
| Fortress Bosses (3) | 3 | yes |
| Castle Bosses (1) | 1 | yes |
| Castle Bosses (2) | 2 | yes |
| Shifting Earth Bosses (1) | 1 | yes |
| Shifting Earth Bosses (2) | 2 | yes |
| Raid Bosses (1) | 1 | yes |

**Count:** 12.

Family rules land in `data/boss_families.json` after we have entity IDs. Until then the names are the feedback surface.

Possible yaml later: `counter_depth: short | default | deep` (drop the high thresholds on short).

---

## E. Hold shop locations (unique SKUs only)

`shop_checks: unique_only` is the default. Repeatable Flatstones are **not** locations.

### E1. Default shop checks

| Location | Shop | Cost (vanilla) | Default |
|---|---|---|---|
| Shop - Besmirched Frame | Bazaar | 1500 Murk | yes |
| Shop - Wylder's Goblet | Bazaar | 1200 | yes |
| Shop - Guardian's Goblet | Bazaar | 1200 | yes |
| Shop - Ironeye's Goblet | Bazaar | 1200 | yes |
| Shop - Raider's Goblet | Bazaar | 1200 | yes |
| Shop - Recluse's Goblet | Bazaar | 1200 | yes |
| Shop - Executor's Goblet | Bazaar | 1200 | yes |
| Shop - Duchess' Goblet | Bazaar | 1200 | yes (needs Duchess unlocked) |
| Shop - Revenant's Goblet | Bazaar | 1200 | yes (needs Revenant unlocked) |
| Shop - Scholar's Goblet | Bazaar | TBD | `include_dlc` |
| Shop - Undertaker's Goblet | Bazaar | TBD | `include_dlc` |
| Shop - Spirit Shelter Grail | Bazaar | 3000 | yes |
| Shop - Giant's Cradle Grail | Bazaar | 3000 | yes |
| Shop - Sacred Erdtree Grail | Bazaar | 3000 | yes |
| Shop - Scadutree Grail | Bazaar | 3000 | `include_dlc` (Dreglord in vanilla) |

**Count (DLC on):** 15.

Buying the SKU is the check. The vessel itself is also an **item** that can sit on any location.

### E2. Optional shop checks (`shop_checks: extra` or separate toggles)

Off by default. Too many for a 5–10 hour seed if all are forced.

| Bucket | Examples | Suggested toggle |
|---|---|---|
| Bazaar fixed color relics | Delicate / Polished / Grand Burning, Tranquil, Drizzly, Luminous Scenes | `shop_scene_relics` |
| Gestures | Polite Bow, Strength!, … Golden Order Totality (~30) | `shop_gestures` |
| Prattling Pates | Hello, Thank You, … You're beautiful (8) | fold into `shop_gestures` |
| Signboard soot / sealed urns | Soot-Covered / Sealed Urn × each Nightfarer | `shop_signboard_vessels` |
| Signboard character Grand Scenes | per-Nightfarer 3-sigil relics | `shop_signboard_relics` |
| Signboard Dark Night relics | Dark Night of the Baron / Beast / Wise / … | `shop_dark_night` — only if everdark is on |
| Garbs / skins | Dawn, Darkness, crossover sets | **cut from AP** unless someone argues hard |

Large Scenic Flatstone / Scenic Flatstone / Deep Scenic Flatstone stay **not checks**.

---

## F. Items / unlocks (pool, not checks)

Remembrance **quests are not locations**. Their rewards go in the pool.

### F1. Starting roster

Yaml: `starting_nightfarers` default **1**.

Drawn at random from the enabled roster:

| Nightfarer | Vanilla lock | In start pool when |
|---|---|---|
| Wylder | open | always |
| Guardian | open | always |
| Ironeye | open | always |
| Raider | open | always |
| Recluse | open | always |
| Executor | open | always |
| Duchess | locked | always (as an unlock item if not started) |
| Revenant | locked | always |
| Scholar | DLC + Gladius + chapel | `include_dlc` |
| Undertaker | DLC + Gladius + chapel | `include_dlc` |

Everyone not chosen as a starter becomes an **item**: `Unlock - Duchess`, `Unlock - Revenant`, …

Old Pocketwatch / Besmirched Frame / Night Idol fight are **not** required play. Those objects can still exist as items or shop checks.

### F2. Board / expedition unlock items

Vanilla gating (Gladius opens six, four Nightlords open Heolstor, DLC chapel opens Balancers) is replaced by items so generation can place them.

| Item | Notes |
|---|---|
| Expedition Unlock - Tricephalos | may be a start item |
| Expedition Unlock - Gaping Jaw | |
| Expedition Unlock - Sentient Pest | |
| Expedition Unlock - Augur | |
| Expedition Unlock - Equilibrious Beast | |
| Expedition Unlock - Darkdrift Knight | |
| Expedition Unlock - Fissure in the Fog | |
| Expedition Unlock - Night Aspect | required for `goal: heolstor` |
| Expedition Unlock - Balancers | DLC |
| Expedition Unlock - Dreglord | DLC |

Feedback wanted: **one item per expedition** vs **progressive Board Unlock x1/x2/x3**. Per-expedition is clearer for trackers. Progressive is fewer sphere-1 items.

### F3. Nightlord first-clear relics (useful / progression)

| Item | From vanilla |
|---|---|
| Night of the Beast | Gladius |
| Night of the Baron | Adel |
| Night of the Wise | Gnoster |
| Night of the Fathom | Maris |
| Night of the Demon | Libra |
| Night of the Champion | Fulghor |
| Night of the Miasma | Caligo |
| Night of the Lord | Heolstor |
| The Night of Dregs | Straghess |
| The Will of the Balancers | Harmonia |

### F4. Remembrance rewards (items)

Chalices + unique relics + skins. Skins default to filler. Chalices and named relics default to useful.

| Nightfarer | Items to pool (base list, confirm in play) |
|---|---|
| Wylder | Slate Whetstone, Wylder's Chalice, Silver Tear, Wylder's Earring |
| Guardian | Stone Stake, Guardian's Chalice, Third Volume, Witch's Brooch / Cracked Witch's Brooch |
| Ironeye | Cracked Sealing Wax, Ironeye's Chalice, Edge of Order |
| Duchess | Golden Dew, Duchess' Chalice |
| Revenant | Small Makeup Brush, Revenant's Chalice, Old Portrait |
| Recluse | Recluse's Chalice, Vestige of Night, Bone-Like Stone |
| Raider | (confirm names in-game) Chalice + unique relics |
| Executor | Executor's Chalice + unique relics |
| Scholar | DLC chalice / relics (Headband of the Golden Ones appears in Dreglord step) |
| Undertaker | Leather Monocle Case, Glass Necklace, DLC chalice |

Remembrance garbs: filler, or `include_cosmetics`.

### F5. Filler

- Murk Bundle (S / M / L)
- Relic Pack (random Delicate / Polished)
- Sovereign Sigils (only if everdark is in the seed)
- Individual gestures if they are not locations

---

## G. Goals (not locations)

| `goal` | Completes when |
|---|---|
| `heolstor` | Nightlord - Heolstor checked |
| `count` | `nightlord_count` distinct Nightlord locations checked (default 4) |
| `specific` | that Nightlord location checked |

Everdark is never the goal unless we add `goal: everdark_heolstor` later.

---

## Default seed size (DLC on, everdark on, unique shops only)

| Bucket | Locations |
|---|---|
| Named Nightlords | 10 |
| Everdark | 7 (+TBD) |
| Night 1 / Night 2 | 20 |
| Counters | 12 |
| Default shops | 15 |
| **Total** | **~64** |

That is the intended 5–10 hour shape: most expeditions cash Night 1 + Night 2 + some counters, and you still choose which Nightlords to hunt for the goal.

Adding gestures + every Signboard SKU would push this toward 150+ and should stay optional.

---

## Feedback prompts

Answer these if you can. Short answers are better than essays.

1. Per-expedition board unlock items, or progressive Board Unlock?
2. Are 20 Night 1/2 cells too many? Alternative: only Night 1, or only the expeditions that are in logic for this seed.
3. Counter high thresholds (World 6, Castle 2, Shifting Earth 2) — keep, drop, or yaml `short`?
4. Default shops: vessels + Besmirched Frame only, or also Bazaar fixed Scene relics?
5. Signboard Dark Night relics: shop checks, or just items sitting on Everdark locations?
6. Remembrance skins in the pool at all?
7. Should `starting_nightfarers: 1` draw from the locked roster (Duchess/Revenant/DLC) or only from the vanilla six?
8. Anything missing that would make a multiworld feel empty (key Hold NPCs, Deep of Night, Shifting Earth *unlocks* as items)?

Paste marked-up copies into issue comments on this repo. Do not add named field bosses to this list.
