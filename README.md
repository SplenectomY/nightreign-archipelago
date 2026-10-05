# Nightreign Archipelago (NRAP)

Archipelago world for *Elden Ring Nightreign*

## Features

- Seamless Coop launch path, tested with more than one Archipelago slot in the same session. `nrsc.dll` loads from the me3 profile and writes `.co2` saves. Each connected client submits its own checks. A player who is not on the Archipelago server can still join the Seamless session and help a slot.
- One random expedition and one random Nightfarer are granted at start. Heolstor stays out of the pool unless `heolstor_in_pool` is set, and unlocks locally after `heolstor_unlock_count` Nightlord defeats.
- Nightlord and Everdark defeat checks. A Nightlord defeat counts only if that expedition is unlocked, so helping another player when you don't have that Nightlord unlocked will grant no credit to you.
- Shop checks (with auto hints!) for the Small Jar Bazaar and Garb Shop. Rows stay hidden until the item is found, then the purchase is the check. A random 3 to 6 bazaar shop items start unlocked.
- Murk and starting runes as items
- Checks (as counters): Day 1 bosses, Day 2 bosses, evergaol seals, magician tower unlocks, invader kills, max flask uses increases, treasure map discoveries, character unlocks (Duchess stopwatch turn in, Revenant battle, talking to Undertaker and Scholar), defeating tutorial Margit
- Goal: Kill Heolstor, kill X Nightlords or kill a specific Nightlord
- Custom launcher GUI with built-in AP client
- Configurable onscreen overlay (requires Borderless Window display setting)
- Fresh save game per seed

## Notes
- Garb shop purchase checks are logically gated behind having access to two non-dlc expeditions. Per normal game logic, you must defeat two nightlords to access the garb shop.
- Defeating tutorial Margit right off the rip has a chance to give you some murk to immediately buy the starting shop purchases for some early sphere 0 checks, pre-first-expedition
- Invader kill checks past the first handful are logically gated behind having the Deep of Night expedition unlocked (easy Calamity spawns)
- Logical spheres for counter checks are equally divided by the number of nightlords you need to kill to win. This is to help prevent other players' early sphere items from being buried in your late checks
- It is perfectly valid and might be helpful to other players to spam day-1-only expeditions where you rush churches, fort map rooms, gaols, towers and invader camps, if they hint something they need that's buried under those checks

## Planned features

1. Deal with known issues (see below)
2. Optional Remembrance Quest checks
3. DLC map specific checks
4. Collector Signboard currency items and checks

## Install

Requires steam install of Nightreign, Seamless Coop mod, and Mod Engine 3

1. Install [Seamless Coop for Nightreign](https://www.nexusmods.com/eldenringnightreign/mods/3) and [me3](https://github.com/garyttierney/me3).
2. Run `NightreignArchipelago.msi`. The default folder is `C:/NRAP` and can be changed. SmartScreen will warn that the publisher is unknown. Click **More info**, then **Run anyway**. See [docs/INSTALL.md](docs/INSTALL.md).
3. Open `nrap.exe`. Check the paths, configure your server and slot, then Launch.

### Servers/Hosts:
Copy the apworld to your Archipelago installation's custom_worlds folder and the yaml to the Players folder (or give these files to whoever is generating your world). See https://archipelago.gg/tutorial/Archipelago/setup_en#playing-with-custom-worlds for more information. Generate, then host.

Catalog: [`docs/LOCATIONS_AND_UNLOCKS.md`](docs/LOCATIONS_AND_UNLOCKS.md).

## Known issues
1. Rare crash to desktop when going to the title menu or loading the game due to eventflag read/write contention. Minimal impact, just relaunch from the NRAP console.
2. Even rarer crashes during other loading sequences, such as starting an expedition
3. If you don't have Wylder unlocked at game start, NRAP will force your Nightfarer to one that's unlocked. However, Wylder's character model will remain applied, so it will look weird until you launch an expedition or manually switch the character in the normal menu
