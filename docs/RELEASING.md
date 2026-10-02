# Releasing

Tags are semantic versions: `v0.5.0`, `v0.5.1`. No phase suffix.

1. Bump `client/Cargo.toml`. The console prints that version.
2. Tag and push. The release workflow builds the DLL and the tester zip.

```
git tag v0.5.0
git push origin v0.5.0
```

The zip contains the DLL, `flags.toml`, the me3 profile, the apworld, the player yaml, `regulation.bin`, and the install notes.
