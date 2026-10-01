# Elden Ring Nightreign — Archipelago

Runtime Archipelago world for *Elden Ring Nightreign*. Vanilla game files stay on disk. A me3/Seamless-loaded DLL talks to an Archipelago server. Progression lives on the Hold, not on Limveld floor loot.

**Status: Phase 0.** Checks, expedition unlocks, and Nightfarer grants work. The regulation package that hides default Nightfarers can crash the expedition board. That package is not in the tester zip.

Repo: https://github.com/SplenectomY/nightreign-archipelago

## Testers: do not install Rust

Download the latest zip from [Releases](https://github.com/SplenectomY/nightreign-archipelago/releases). It already contains `nightreign_ap.dll`.

You still need Nightreign, Seamless Coop, and me3. You do not need Visual Studio or `cargo`.

## Install

1. Install [Seamless Coop for Nightreign](https://www.nexusmods.com/eldenringnightreign) and [me3](https://github.com/garyttierney/me3).
2. Unzip the release into `C:/Mods/nightreign-ap/`. `nightreign_ap.dll` and `flags.toml` must stay in that same folder.
3. Copy `nightreign-ap.me3` to the me3 profiles folder. Edit the two paths if your install is not the default. Paths must use forward slashes. A backslash is an escape character and the profile will not load.
4. Launch with me3 using that profile. Do not use `nrsc_launcher.exe` at the same time, and do not use Steam's Play button.
5. The console must print `NRAP attached` and `NRAP AP connected`.
6. Copy `nightreign.apworld` to `%USERPROFILE%\Archipelago\custom_worlds\` and `Nightreign.yaml` to the Archipelago Players folder. Generate, then host the output with MultiServer before launching the game.

The zip includes `regulation/regulation.bin`, and the profile loads it. Do not open the expedition board unless the Nightfarer you are walking around as has been granted. A locked active Nightfarer exits the game without a Windows crash dump. Remove the `[[packages]]` block to run without that override.

## Locked design

- **Offline first.** Generate locally, host `MultiServer` on localhost. Official From matchmaking is never used.
- **Launch path is Seamless Coop.** `nrsc.dll` loads from the me3 profile and writes `.co2` saves.
- **One AP slot** when a Seamless party exists. Host client submits checks. Friends in BK in other worlds can join the Seamless session and help.
- **Named boss checks = Nightlords (and optional Everdark / DLC Nightlords) only.**
- **Checks fire on the kill / the purchase.** A wipe after a Night 1 kill still keeps that check.
- **Start with 1 random Nightfarer.** Revenant is excluded from that pick until the active-character slot can be set. Remembrance quest items go in the item pool; the quests themselves are not checks.
- **Heolstor stays out of the pool** unless `heolstor_in_pool` is set. He unlocks locally after `heolstor_unlock_count` expedition unlocks.

## Layout

```
world/nightreign/     apworld source
client/               Rust cdylib loaded into nightreign.exe
profiles/             me3 profile
players/              template YAML
docs/                 design + tester brief
data/                 flag table the client reads
```

Planned check list: [`docs/LOCATIONS_AND_UNLOCKS.md`](docs/LOCATIONS_AND_UNLOCKS.md).

## Requirements

- Nightreign PC (Steam), 1.03.3.0
- Seamless Coop for Nightreign
- me3
- The zip from Releases
- Archipelago 0.6.7+ for the server half

Developers who want to change the DLL still use Rust + MSVC. That is optional.
