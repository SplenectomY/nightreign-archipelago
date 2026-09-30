# Elden Ring Nightreign — Archipelago

Runtime Archipelago world for *Elden Ring Nightreign*. Vanilla game files stay on disk. A me3/Seamless-loaded DLL talks to an Archipelago server. Progression lives on the Hold, not on Limveld floor loot.

**Status: Phase 0 (attach + one check).** Not playable as a multiworld yet.

Repo: https://github.com/SplenectomY/nightreign-archipelago

## Locked design

- **Offline first.** Generate locally, host `MultiServer` on localhost. Official From matchmaking is never used.
- **Launch path is Seamless Coop.** `nrsc_launcher.exe` disables EAC and writes `.co2` saves. That is the supported way to run the client. me3-only is a fallback attach test, not the product path.
- **One AP slot** when a Seamless party exists. Host client submits checks. Friends in BK in other worlds can join the Seamless session and help. Same model as Minecraft AP co-op on one slot.
- **Named boss checks = Nightlords (and optional Everdark / DLC Nightlords) only.** Everything else is a tiered pool or a counter (`Gladius Night 1 Boss`, `World Bosses (3)`, `Church Bosses (1)`, …).
- **Checks fire on the kill / the purchase**, including mid-expedition. A wipe after a Night 1 kill still keeps that check.
- **Start with N random Nightfarers** (default 1). Remembrance quest items go in the item pool; the quests themselves are not checks.
- **Everdark Sovereigns unlocked offline** by this mod unless that turns out to be server-authoritative in a way we cannot fake. YAML toggle still exists so offline seeds can exclude them.

## Layout

```
world/nightreign/     apworld source (Phase 0 generates)
client/               Rust cdylib loaded into nightreign.exe
profiles/             me3 profile
players/              template YAML
docs/                 design + tester briefs
data/                 flag table the client reads
```

## Phase 0 goal

One end-to-end loop:

1. DLL attaches under Seamless (or me3) and logs it.
2. You defeat **Gladius**.
3. Client sees the kill flag and sends `LocationChecks`.
4. Local AP server accepts it.
5. Client applies one incoming item (Phase 0: log + optional Murk poke once we have an address).

Tester work is in [`docs/PHASE0_TESTER.md`](docs/PHASE0_TESTER.md). Flag hunt is in [`docs/FLAG_DISCOVERY.md`](docs/FLAG_DISCOVERY.md).

## Requirements (Phase 0)

- Nightreign PC (Steam)
- [Seamless Coop for Nightreign](https://www.nexusmods.com/eldenringnightreign) — **required launch path**
- [me3](https://github.com/garyttierney/me3) — used to inject our DLL next to `nrsc.dll`, or as a solo attach fallback
- [Archipelago 0.6.7+](https://archipelago.gg) for generate + local host
- Rust nightly/stable + MSVC if you build the client
- Cheat Engine 7.5+ and Smithbox for flag work

## Quick generate (apworld skeleton)

```text
python tools/pack_apworld.py
# drops nightreign.apworld
# copy into Archipelago/custom_worlds/
# put players/Nightreign.yaml in Archipelago/Players/
# Generate, then host the output zip with MultiServer
```

## References

- ER AP (pattern, not a port): https://github.com/4laric/er-archipelago
- Nightreign map / seed notes: https://thefifthmatt.github.io/nightreign/
- AP world docs: https://github.com/ArchipelagoMW/Archipelago/blob/main/docs/adding%20games.md
