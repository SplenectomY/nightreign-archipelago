# Nightreign Archipelago

Archipelago world for *Elden Ring Nightreign*, version 0.5.0. Vanilla game files stay on disk. A me3 profile loads Seamless Coop and `nightreign_ap.dll`. The DLL talks to an Archipelago server. Progression lives on the Hold.

## In this build

- Seamless Coop launch path, tested with more than one Archipelago slot in the same session. `nrsc.dll` loads from the me3 profile and writes `.co2` saves. Each connected client submits its own checks. A player who is not on the Archipelago server can still join the Seamless session and help a slot.
- Expedition board gated per unlock, including Tricephalos. One random expedition and one random Nightfarer are granted at start. Heolstor stays out of the pool unless `heolstor_in_pool` is set, and unlocks locally after `heolstor_unlock_count` Nightlord defeats.
- Nightlord and Everdark defeat checks. A Nightlord defeat counts only if that expedition is unlocked.
- Shop checks for the Small Jar Bazaar. Rows stay hidden until the item is found, then the purchase is the check. A random 3 to 6 rows start unlocked.
- Murk purses, bundles, coffers, chests, and hoards. Received murk is remembered per seed.
- Counters: Day 1 bosses, Day 2 bosses, evergaols, magician towers. The first of each is check 1.
- Goal: a Nightlord count, or a specific Nightlord. Default is Heolstor.
- Console commands: `!` sends to the server, `/debug on` and `/debug off`. Item and location names are printed, not raw ids.
- `archipelago.gg` uses `wss`. A localhost host uses `ws`.

## Not in this build

1. Tutorial Margit as an optional check.
2. Garb shop checks.
3. Optional remembrance quest checks.
4. More overworld checks, where a stable flag exists.
5. The walking model updating when the active Nightfarer is forced off Wylder.

## Install

You still need Nightreign, Seamless Coop, and me3. You do not need Visual Studio or `cargo`.

1. Install [Seamless Coop for Nightreign](https://www.nexusmods.com/eldenringnightreign/mods/3) and [me3](https://github.com/garyttierney/me3).
2. Unzip a release to `C:/Mods/nightreign-ap/`. `nightreign_ap.dll` and `flags.toml` must stay in that folder.
3. Copy `nightreign-ap.me3` to the me3 profiles folder. Paths must use forward slashes.
4. Copy `nightreign.apworld` to `C:/ProgramData/Archipelago/custom_worlds/` and `Nightreign.yaml` to the Archipelago Players folder. Generate, then host.
5. Launch with me3. Do not also launch `nrsc_launcher.exe`.
6. The console must print `NRAP attached 0.5.0` and `NRAP AP connected`.

The zip includes `regulation/regulation.bin`, and the profile loads it. Do not open the expedition board or character select until the unlock cache is armed. A locked active Nightfarer exits the game.

## Layout

```
world/nightreign/     apworld source
client/               Rust cdylib loaded into nightreign.exe
profiles/             me3 profile
players/              template YAML
docs/                 design and install notes
data/                 flag table the client reads
```

Catalog: [`docs/LOCATIONS_AND_UNLOCKS.md`](docs/LOCATIONS_AND_UNLOCKS.md).

## Requirements

- Nightreign PC (Steam)
- Seamless Coop for Nightreign
- me3
- A release zip
- Archipelago 0.6.7 or newer for the server

Developers who change the DLL need Rust and MSVC. That is optional.
