# Design

This is the contract. Code follows it. If a later idea fights this file, change the file first.

## Game name

Archipelago game string: `Elden Ring Nightreign`

## Two processes, two networks

| Layer | Job | Transport |
|---|---|---|
| Seamless Coop (`nrsc.dll`) | EAC off, private session, shared expedition | Steam P2P |
| This client (`nightreign_ap.dll`) | Checks, items, goal | Archipelago WebSocket |

Do not invent a third Nightreign game server. AP does not sync boss HP.

## Slot model

- Solo offline: one slot, local MultiServer.
- Seamless party: multiple Archipelago slots in one session, tested. Each connected client submits its own checks.
- A player who is not connected to the Archipelago server can join the Seamless session normally and help a slot. They do not submit checks.

## Locations

### Named (always unique)

- Each base Nightlord
- Each DLC Nightlord (Harmonia / Balancers, Straghess / Dreglord) when `include_dlc` is on
- Each Everdark Sovereign when `include_everdark` is on
- Unique Hold shop SKUs (vessels, key relics, Besmirched Frame, Signboard uniques)

### Tiered pools (not named inner bosses)

One location per cell, not per inner boss name:

- `{Expedition} Night 1 Boss`
- `{Expedition} Night 2 Boss`

Expeditions: Tricephalos, Gaping Jaw, Sentient Pest, Augur, Equilibrious Beast, Darkdrift Knight, Fissure in the Fog, Night Aspect, plus DLC when enabled.

### Counters

Separate locations that complete when the running unique-kill count reaches the threshold. Thresholds are YAML.

Default thresholds (v1 target, not Phase 0):

- World bosses: 1, 3, 6
- Church bosses: 1, 3
- Fortress bosses: 1, 3
- Castle bosses: 1, 2
- Shifting Earth bosses: 1, 2
- Raid bosses: 1

A kill is attributed to **at most one** counter family. Classification table lives in `data/boss_families.json` once we have entity IDs.

Nightlord kills never increment counters.

### When a check fires

On the moment of defeat or purchase. Persistence must survive:

- returning to the Hold after a win
- returning to the Hold after a wipe
- process restart, reading the save

If a Night 1 boss dies and the run later wipes, that Night 1 location is still checked.

## Items

Progression:

- Nightlord unlock tokens (replace vanilla board unlocks as needed)
- Nightfarer unlocks (Duchess, Revenant, Scholar, Undertaker, and any other locked class)
- Vessel / Goblet / Grail unlocks
- Remembrance key items (the rewards, not the quest steps)
- Shop-stock unlocks if vanilla gating is too tight for generation

Filler:

- Murk bundles
- Relic packs
- Sovereign Sigils (only if Everdark is in logic)
- Gestures

Start inventory:

- `starting_nightfarers`: int, default **1**, chosen at random from `starting_nightfarer_pool`.
- `starting_nightfarer_pool`: string list, default all ten. Possible values: wylder, guardian, ironeye, duchess, raider, revenant, recluse, executor, scholar, undertaker. DLC names are ignored unless `include_dlc` is on. Names left out stay in the item pool.
- The player does not begin with the full vanilla six unless they set the count that high.

## Goals (YAML)

```yaml
goal: heolstor          # heolstor | count | specific
nightlord_count: 4      # used when goal is count
specific_nightlord: heolstor
include_everdark: true
include_dlc: true
shop_checks: unique_only
starting_nightfarers: 1
```

`heolstor` still requires whatever unlock items the region graph uses so the seed is beatable.

## Everdark offline

Vanilla Everdark is session/online gated. This repo will ship an offline unlock with the client (flag write and/or regulation/event patch) so `include_everdark: true` is valid on a localhost seed.

If discovery shows the fight data is simply missing offline and cannot be enabled without a live directory, we flip the default to `false` and document it. Until that is proven, treat offline Everdark as in-scope.

## Shipped in 0.5.0

Seamless Coop is the tested launch path, including more than one Archipelago slot in one session. Expedition and Nightfarer gating, shop checks, murk, day-boss and evergaol and tower counters, and the Heolstor gate are in.

## Planned

1. Tutorial Margit as an optional check.
2. Garb shop checks.
3. Optional remembrance quest checks.
4. More overworld checks, where a stable flag exists.
5. Walking model update when the active Nightfarer is forced off Wylder.

## Death Link

On sends flag 9017 to other Nightreign slots with Death Link on. Incoming links are ignored outside an expedition.
instant sets HP to 0. percent removes a yaml percent of max HP. dice kills on a yaml chance and does nothing on a miss.

## Out of scope

- Named catalog of every field boss
- In-run weapon shuffle
- Official online
