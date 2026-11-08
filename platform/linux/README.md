# Linux packaging and shell integration

## Velopack AppImage

Release CI builds an x86_64 AppImage with Velopack (`packId` `com.shellzip.ShellZip`, channel `linux`). Users download the AppImage, run `chmod +x`, and start it. Auto-updates use the same GitHub Releases feed as Windows/macOS.

Stage and pack locally:

```bash
make release
make velopack-pack    # VELOPACK_CHANNEL=linux
```

Format icons ship beside the binary as `icons/` (freedesktop tree from `grx/shellzip_file_icons`). First launch runs file-manager integrate.

### GPG signatures

When release secrets `GPG_PRIVATE_KEY` / `GPG_KEY_ID` are set, CI publishes `SHA256SUMS`, `SHA256SUMS.asc`, and `*.AppImage.asc`. Verify with the public key documented in [docs/BUILDING.md](../../docs/BUILDING.md) (published as `docs/signing-key.asc` once the release key exists).

## Integrate

```bash
shellzip --integrate
shellzip --unintegrate
```

Writes `~/.local/share/applications/shellzip.desktop`, hicolor MIME icons, and KDE/Nemo/Thunar actions. Optional Nautilus Python extension: [nautilus/shellzip.py](nautilus/shellzip.py) (GPL-2.0+, not installed by `--integrate`).
