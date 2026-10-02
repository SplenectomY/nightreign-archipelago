# Flag discovery cookbook

Targeted memory work for this project. Historical flag-discovery notes. Install is [`INSTALL.md`](INSTALL.md).

## What we will accept as a "flag"

A good check signal is all of:

1. **Boolean.** 0 before the event, non-zero after.
2. **Sticky.** Still set after Hold return and after a full process restart on the same save.
3. **Save-scoped.** A new character reads 0.
4. **Event-scoped.** Beating Adel does not set the Gladius bit.
5. **Kill-scoped, not extract-scoped** for expedition bosses. If the run dies after the kill, the bit stays on.

Prefer an event-flag ID over a raw heap address. Addresses move every patch; flag IDs usually survive.

## Nightlord table (named checks)

| Location name | Expedition | Boss | DLC | Everdark sibling |
|---|---|---|---|---|
| Nightlord - Gladius | Tricephalos | Gladius, Beast of Night | no | yes |
| Nightlord - Adel | Gaping Jaw | Adel, Baron of Night | no | yes |
| Nightlord - Gnoster | Sentient Pest | Gnoster, Wisdom of Night | no | yes |
| Nightlord - Maris | Augur | Maris, Fathom of Night | no | yes |
| Nightlord - Libra | Equilibrious Beast | Libra, Creature of Night | no | yes |
| Nightlord - Fulghor | Darkdrift Knight | Fulghor, Champion of Nightglow | no | yes |
| Nightlord - Caligo | Fissure in the Fog | Caligo, Miasma of Night | no | yes |
| Nightlord - Heolstor | Night Aspect | Heolstor the Nightlord | no | unknown — record if present |
| Nightlord - Harmonia | Balancers | Weapon-Bequeathed Harmonia | yes | record if present |
| Nightlord - Straghess | Dreglord | Traitorous Straghess | yes | record if present |

Everdark locations use the same names with the prefix `Everdark - `.

The first confirmed defeat flag was **Nightlord - Gladius**.

## After Gladius is found

Hunt in this order. One save per Nightlord when you can spare it; otherwise one save and note the order.

1. Other base Nightlords (Adel … Caligo, then Heolstor).
2. Everdark of a Nightlord you have already beaten. First question: can you *see* the Everdark expedition at all while Seamless is in a 1-player lobby with no From directory? If the board entry is missing, that is the offline-unlock problem — dump the board-unlock flags before you fight anything.
3. DLC Nightlords if the DLC is installed.
4. One unique Bazaar purchase (Besmirched Frame or a Goblet).
5. One Night 1 boss kill + intentional wipe.

## Offline Everdark

We want a client-side unlock. Hunt these separately from the kill flags:

- Board visibility bit for the current Everdark row
- "has defeated normal version" bit (likely already the Nightlord flag)
- Any "online-only content" / directory / seasonal rotation bit
- Regulation / event patches thefifthmatt-style file mods already touch

If toggling a local flag makes the expedition selectable and the arena loads, we ship that write in the DLL. If the arena assets themselves are gated on a network message, write that down and stop; do not try to reconstruct a From server.

## Shop purchases

A shop check is the purchase transaction, not "item is in the relic inventory." Random Flatstones are not unique checks. Record:

- Shop param row ID
- Talk ESD command that spends Murk / Sigils
- Flag set on first purchase of that row, if any

## How to record a flag in-repo

Add a block to `data/flags.toml`. Do not invent IDs. Example:

```toml
[[flag]]
location = "Nightlord - Gladius"
kind = "event_flag"
event_flag_id = 0          # replace
pointer_note = "CSEventFlagMan+0x?? / bit offset"
verified_version = "1.xx.x"
survives_hold = true
survives_relaunch = true
zero_on_new_save = true
```

Raw addresses go in `verified_version` notes only. The client must resolve through a pointer or a flag-manager helper, not a frozen heap VA.

## Tools

- Cheat Engine 7.5+
- Smithbox, Nightreign game directory
- Optional: fromsoftware-rs Nightreign bindings, DarkScript-style event dumps
- Do not commit game assets


## Expedition sample: Adel / Gaping Jaw

Recorded 2026-10-01. Expedition was Adel. Day 1 boss was Gaping Dragon. Day 2 boss was Ancient Dragon. These clusters are one sample, not yet known to be the boss id versus the day slot.

Day 1 death window, Gaping Dragon: 887007, 887008, 887010, 887012, 887013, 887014. Earlier 887000-887011 bits came on during the fight.

Day 2 death window, Ancient Dragon: 752021, 752025, 752028, 797003, 797007, 797010, 896002, 896003, 896006, 896009, 896010. The 896xxx set matches the shape of the Day 1 887xxx fight flags.

A second expedition with a different Day 1 boss is required before treating 887xxx as Gaping Dragon rather than "night 1 boss died."
