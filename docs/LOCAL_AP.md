# Local Archipelago test

Slot name **Player1** and host **127.0.0.1:38281** match the host defaults and `data/flags.toml`.

## 1. Install Archipelago

1. Download the latest Windows installer from https://github.com/ArchipelagoMW/Archipelago/releases
2. Install it, or use a portable folder. The default install path is `C:\ProgramData\Archipelago`.
3. Confirm that install has `custom_worlds` and `Players`.

If ProgramData is hidden: Archipelago Launcher, then **Browse Files**.

## 2. Pack and install the world

From the repo:

```powershell
cd C:\Dev\nightreign-archipelago
git pull
Compress-Archive -Path world\nightreign -DestinationPath nightreign.apworld -Force
Copy-Item -Force .\nightreign.apworld C:\ProgramData\Archipelago\custom_worlds\nightreign.apworld
```

Change the copy path if Archipelago is portable. The archive must contain `nightreign\__init__.py`, not `world\nightreign\__init__.py`.

## 3. Player yaml

```powershell
Copy-Item C:\Dev\nightreign-archipelago\players\Nightreign.yaml C:\ProgramData\Archipelago\Players\Player1.yaml -Force
```

Leave only this yaml in `Players` for the first seed. The template name is already `Player1`.

## 4. Generate

Archipelago Launcher, then **Generate**.

Success is a new `AP_*.zip` in the Archipelago `output` folder.

Failure: paste the Generate console. Common causes:

- `nightreign.apworld` missing or the zip layout is wrong
- extra yamls for games whose worlds are not installed
- an option name that is not in `players/Nightreign.yaml`

## 5. Host locally

Archipelago Launcher, then **Host**, then open the new `AP_*.zip`.

Leave that window open. The default port is **38281**. The room should list slot `Player1` and game `Elden Ring Nightreign`.

## 6. Launch with NRAP

Open `nrap.exe`. Host `127.0.0.1:38281`, slot `Player1`, password empty. Launch connects first, swaps the Seamless `.co2` for this seed, then starts me3.

Wanted lines:

```text
NRAP AP connected
NRAP AP LocationChecks 839000001
```

`839000001` is Gladius. `839000100` is the board unlock. The host console should show Player1 check **Nightlord - Gladius** after that expedition.

`flags.toml` is written by the launcher. Hand-editing it is only needed for a local DLL build that is not launched from `nrap.exe`.

## 7. If Connect is refused

- Host window closed: start Host again
- Slot name is not `Player1`: change the yaml `name:` or the host slot field
- The room has a password: put it in the host password field
- Generate used a different game name: the client connects as `Elden Ring Nightreign`
- Invalid Slot: fix the slot name and press Reconnect. NRAP does not keep retrying that error
