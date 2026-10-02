# Local Archipelago test

Slot name **Player1** and host **127.0.0.1:38281** match `data/flags.toml`.

## 1. Install Archipelago

If you do not already have it:

1. Download the latest Windows installer from
   https://github.com/ArchipelagoMW/Archipelago/releases
2. Install it. Default path is `C:\ProgramData\Archipelago`.
3. Confirm these folders exist:
   - `C:\ProgramData\Archipelago\custom_worlds`
   - `C:\ProgramData\Archipelago\Players`
   - `C:\ProgramData\Archipelago\output`

If ProgramData is hidden: Archipelago Launcher → **Browse Files**.

## 2. Pack and install the Nightreign world

In PowerShell:

```powershell
cd C:\Dev\nightreign-archipelago
git pull

$stage = Join-Path $env:TEMP "nrap-apworld"
Remove-Item $stage -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path (Join-Path $stage "nightreign") | Out-Null
Copy-Item .\world\nightreign\* (Join-Path $stage "nightreign") -Recurse
$zip = Join-Path $stage "nightreign.zip"
Compress-Archive -Path (Join-Path $stage "nightreign") -DestinationPath $zip -Force

$destDir = "C:\ProgramData\Archipelago\custom_worlds"
if (-not (Test-Path $destDir)) { New-Item -ItemType Directory -Path $destDir | Out-Null }
$dest = Join-Path $destDir "nightreign.apworld"
Copy-Item $zip $dest -Force
Write-Host "Installed $dest"
```

Or: Archipelago Launcher → **Install APWorld** → pick that `nightreign.apworld`.

The zip must contain `nightreign\__init__.py`, not `world\nightreign\__init__.py`.

## 3. Player yaml

```powershell
Copy-Item C:\Dev\nightreign-archipelago\data\Player1.yaml C:\ProgramData\Archipelago\Players\Player1.yaml -Force
```

Leave only this yaml in `Players` for the first seed (move other yamls out).

## 4. Generate

Archipelago Launcher → **Generate**.

Success: a new `AP_*.zip` in `C:\ProgramData\Archipelago\output`.

Failure: paste the Generate console. Common causes:
- `nightreign.apworld` missing or zip layout wrong
- extra yamls for games you do not have worlds for
- option name mismatch (use the repo `Player1.yaml` as-is)

## 5. Host locally

Archipelago Launcher → **Host** → open the new `AP_*.zip`.

Leave that window open. Default port is **38281**.

You should see the server accept connections and list slot `Player1` / game `Elden Ring Nightreign`.

## 6. Launch Nightreign with NRAP

`flags.toml` is already:

```toml
[ap]
host = "127.0.0.1:38281"
slot = "Player1"
password = ""
```

Continue the post-Gladius save through the `.me3` profile.

Wanted NRAP lines:

```text
NRAP AP targeting 127.0.0.1:38281 slot Player1
NRAP AP connected
NRAP AP LocationChecks 839000001
```

`839000001` is Gladius. `839000100` (board unlock) is not in the Phase 0 location table; the server may ignore it.

On the Host console you should see Player1 check **Nightlord - Gladius**.

## 7. If Connect is refused

- Host window closed → start Host again
- Slot name not `Player1` → rename yaml `name:` or change `flags.toml` slot to match
- Password set on the room → put it in `flags.toml`
- Generate used a different game name → Connect packet uses `Elden Ring Nightreign`
