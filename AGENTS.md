# Agent directives

Standing rules for this repo. Follow them on every change.

## Version

Bump `client/Cargo.toml` and `host/Cargo.toml` to the same version in the same commit as any code change. The attach line prints that version. Do not leave a behavior change on the old number.

## Install paths

- Client DLL: `C:\NRAP\nightreign_ap.dll`
- Host exe: `C:\NRAP\nrap.exe`
- Regulation: `C:\NRAP\regulation.bin`
- Apworld: `C:\ProgramData\Archipelago\custom_worlds\nightreign.apworld`

Pull with `git pull` before building. Do not check out a single file and call that an update.

Pack the apworld from the repo root, not from `world`:

```powershell
Compress-Archive -Path world\nightreign -DestinationPath nightreign.apworld -Force
Copy-Item -Force .\nightreign.apworld C:\ProgramData\Archipelago\custom_worlds\nightreign.apworld
```

## Flag work

- Flag 2030 is 0 on the title screen. Read only 2030 there. No other flag reads or writes.
- Flag 7500, 7505, or 7510 means an expedition is running. Do not rewrite shop, nightfarer, or expedition unlocks, and do not scan shop purchase checks. Combat and counter checks still run.
- Debug log is the master switch. Sticky writes, dayflags, flagdiffs, and rewrites log only when their own option is also checked. Flagdiff stays off unless that box is checked.

## Releases

Ship the installer, apworld, and yaml as separate release assets. Do not attach source-code links.
