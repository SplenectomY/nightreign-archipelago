# Flag protocol

Historical discovery steps. Current install is [INSTALL.md](INSTALL.md). This is not the feature list.


You are the only person on a real Nightreign install. I cannot see your game. Work the steps in order. Paste results back as the report template at the bottom. Do not skip the “before kill” snapshot.

this pass succeeds when all five of these are true:

1. `nightreign_ap.dll` is loaded in `nightreign.exe` launched through **Seamless**.
2. The AP console line `NRAP attached` appears.
3. Defeating Gladius flips a recorded flag on the same frame as “Nightlord Slain” (or immediately after the results banner).
4. That flag is still set after you return to the Hold **and** after you fully close the game and reopen the same `.co2` save.
5. With a local MultiServer running the this pass yaml, the client logs `NRAP check: Nightlord - Gladius`.

Item grant (Murk poke) is bonus, not required to close this pass.

## 0. Software

Install, do not mix versions mid-test:

- Nightreign, current Steam build. Write the exact exe product version into the report (`nightreign.exe` → Properties → Details).
- Seamless Coop for Nightreign. Write the Nexus file version.
- me3, current release.
- Cheat Engine 7.5+.
- This repo, `main`.
- Archipelago 0.6.7+ if you are doing the server half today. You can defer AP to after the flag is found.

Save family: Seamless `.co2` only. Do not test on `NR0000.sl2`. Copy the live save before every session.

Default paths:

```
Game:  ...\Steam\steamapps\common\ELDEN RING NIGHTREIGN\Game\
Save:  %AppData%\Nightreign\<steam_id>\NR0000.co2
```

## 1. Build the DLL

From `client/`:

```
cargo build --release
```

Output: `client/target/release/nightreign_ap.dll`

If Rust is not on the machine yet, say so and stop. Do not grab a random From DLL as a stand-in.

## 2. Launch path (Seamless + our DLL)

Two acceptable injections. Prefer A.

### A. me3 profile that also loads Seamless (preferred)

1. Copy `profiles/nightreign-ap.me3` somewhere writable.
2. Edit the two `[[natives]]` paths:
   - `nrsc.dll` inside your `SeamlessCoop` folder
   - `nightreign_ap.dll` from the cargo output
3. Confirm `nrsc_settings.ini` has whatever player count you use (1 is fine for this pass).
4. Launch through me3 with that profile, **not** through `nightreign.exe` and **not** through Steam’s Play button.

### B. Seamless launcher + extra DLL

If A fails to start a session:

1. Launch with `nrsc_launcher.exe` as you already do.
2. Use whatever “additional DLL” mechanism your Seamless build documents. If it has none, use A. Do not inject with a random third-party injector.

Either way, a console titled `NRAP` must open. First lines:

```
NRAP attached
NRAP nightreign.exe base = 0x...
NRAP config = <path>\flags.toml
```

If the console never appears, the DLL did not load. Stop and send the me3 / Seamless log, not a flag scan.

## 3. Fresh character

New Seamless save. Finish tutorial only as far as you must to sit in the Hold and ready Tricephalos. Do not beat Gladius on this save before the scan.

Note the Nightfarer you picked.

## 4. Attach Cheat Engine

1. CE → `nightreign.exe` (the game process, not the launcher).
2. `Memory Scan Options` → uncheck Fast Scan if the first pass is empty.
3. Keep CE attached for the whole expedition.

## 5. Pre-kill snapshot

At the Hold, before Commence Expedition:

- CE: first scan, **Value Type: 4 Bytes**, **Value: 0**, **Scan Type: Exact**. This is *not* the real hunt yet; it only warms you.
- Real hunt starts in step 7.

Also write down Nightreign version, Seamless version, and that you are not on From servers.

## 6. Run Tricephalos to Gladius

Play normally. You may die on Day 1/2; that is fine. Do **not** Alt+F4 after the Nightlord dies until CE has a candidate.

When Gladius dies, the on-screen cue is `Nightlord Slain` / the expedition results screen. That is T0.

## 7. Flag hunt (do this, in this order)

FromSoft games usually store completion as **event flags** (a bit in a large array), not as a single `int = 1` named “gladius”. We try the cheap scans first anyway.

### 7a. 4-byte 0→1

1. First scan for Unknown initial value, 4 Bytes, at the Hold on the Gladius-kill save.
2. At T0, Next Scan → **Increased value** or **Exact Value 1**.
3. If millions of results, Next Scan → **Unchanged** on the results screen.
4. Return to the Hold. Next Scan → **Unchanged**.
5. Exit, relaunch Seamless, load the same save, attach CE, pointer-scan anything that is still `1` and was `0` on a brand-new save.

This pass often fails. Record hit counts after each Next Scan.

### 7b. Byte 0→1

Same as 7a with **1 Byte**. Event-flag bytes are the usual win.

### 7c. Event flag array (the pass we actually expect to work)

Nightreign is a From title. Look for the same pattern as Elden Ring’s `CSEventFlagMan`.

In CE:

1. Search for the string `CSEventFlag` or `EventFlag` as **String** in `nightreign.exe` + loaded modules.
2. Note every hit. Paste the module + offset list into the report.
3. In Smithbox (Nightreign regulation / event files):
   - Open the Tricephalos / Gladius event scripts.
   - Search event flag IDs written on Nightlord death (`BOSS_DEAD`, `DEFEAT`, `NIGHTLORD`, `GLADIUS`, `TRICEPHALOS`).
   - List every flag ID set in the death/award handler.
4. Try those IDs against Nightreign’s flag manager once you have a pointer to the array.

We do not need the full array mapped in this pass. We need **one** persistent bit that is 0 before Gladius and 1 after, on this save.

### 7d. Confirm uniqueness

On a **second** new Seamless save that has **not** beaten Gladius, the same address / flag ID must read 0. If it reads 1 on a fresh save, discard it.

## 8. What to send back

Fill this literally. Blank fields are worse than “unknown”.

```
## Report
Date:
nightreign.exe version:
Seamless version:
me3 version:
Launch path used: A / B
NRAP console appeared: yes/no
NRAP first 20 lines:
Nightfarer:
Save file name / copy path:

### Attach
DLL path loaded:
Other natives loaded:

### 7a 4-byte
Hits after first Increased/Exact 1:
Hits after Hold return:
Hits after relaunch:
Candidate addresses:

### 7b 1-byte
(same)

### 7c event flags
Smithbox file names opened:
Flag IDs set on Gladius death:
CSEventFlag string hits (module+offset):
Pointer to flag array if found:
Tested flag ID:
Value before kill:
Value after kill:
Value after Hold:
Value after relaunch:
Value on fresh save:

### Persistence
Survives Hold return:
Survives process restart:
Survives wipe-after-kill? (if you tested Night 1, write it; Gladius win always returns you)

### AP half (if attempted)
MultiServer connected:
Log line "NRAP check: Nightlord - Gladius": yes/no
```

Put the finished report in a GitHub issue on this repo or paste it in chat. I will write the ID into `data/flags.toml` and wire the client poller.

## 9. Optional same-session extras (only after 7 is green)

1. Buy one unique Bazaar item you have never bought on that save. Repeat 7b/7c for that purchase.
2. Kill a Night 1 boss and wipe on purpose. Confirm a *different* flag from Gladius went 0→1 and stayed 1 after the wipe.

## Do not

- Do not test on official online.
- Do not convert `.co2` back to `.sl2`.
- Do not dump the whole regulation into the repo.
- Do not start a named-boss spreadsheet. Nightlords only for named checks.
