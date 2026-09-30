# Releasing tester builds

Testers should not install Rust. GitHub Actions builds the MSVC DLL and attaches a zip to [Releases](https://github.com/SplenectomY/nightreign-archipelago/releases).

## Cut a release from the site

1. Actions → **Release** → Run workflow.
2. Tag: `v0.1.0-phase0` (bump the number when the DLL changes).
3. Leave prerelease checked until Phase 0 is closed.
4. Wait for the Windows job. The zip lands on the Releases page.

## Cut a release from git

```
git tag v0.1.1-phase0
git push origin v0.1.1-phase0
```

Same workflow runs on `v*` tags.

## What is in the zip

- `nightreign_ap.dll`
- `flags.toml` (sit this next to the DLL)
- `nightreign-ap.me3`
- `nightreign.apworld`
- `Nightreign.yaml`
- `PHASE0_TESTER.md`

Do not commit the DLL to `main`. The zip is the distribution.
