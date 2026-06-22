# Installing freemkv (CLI)

The `freemkv` CLI ships as a **single static binary** — no runtime, no
container, no shared-library dependencies. Download one file, make it
executable, run it. Docker is **not** required for the CLI.

Release assets are attached to every tagged release at
<https://github.com/freemkv/freemkv/releases/latest>.

## Asset names

Each release carries, per platform:

| Asset | Platform |
|-------|----------|
| `freemkv-x86_64-linux` | Linux x86_64 (static musl) |
| `freemkv-aarch64-linux` | Linux arm64 (static musl) |
| `freemkv-x86_64-macos` | macOS Intel |
| `freemkv-aarch64-macos` | macOS Apple Silicon |
| `freemkv-x86_64-windows.exe` | Windows x86_64 |

Each binary has a matching `<asset>.sha256` checksum file. The same
release also still carries the `*.tar.gz` / `*.zip` archives if you
prefer those.

## Linux / macOS

```bash
# Pick the asset for your platform (Linux x86_64 shown).
ASSET=freemkv-x86_64-linux

curl -sLO "https://github.com/freemkv/freemkv/releases/latest/download/${ASSET}"
curl -sLO "https://github.com/freemkv/freemkv/releases/latest/download/${ASSET}.sha256"

# Verify (optional but recommended).
shasum -a 256 -c "${ASSET}.sha256"     # macOS / BSD
# sha256sum -c "${ASSET}.sha256"       # Linux GNU coreutils

chmod +x "${ASSET}"
sudo mv "${ASSET}" /usr/local/bin/freemkv

freemkv --version
```

## Windows

Download `freemkv-x86_64-windows.exe` from the releases page, rename to
`freemkv.exe`, and place it somewhere on your `PATH`.

## Using it

Every operation is `freemkv <source> <dest>` over stream URLs:

```bash
freemkv disc:// mkv://Movie.mkv             # Disc → MKV
freemkv disc:// m2ts://Movie.m2ts            # Disc → raw transport stream
freemkv m2ts://Movie.m2ts mkv://Movie.mkv    # Remux m2ts → MKV
freemkv info disc://                         # Show disc info
```

### Decryption keys

- **DVD (CSS):** works out of the box, no setup.
- **Blu-ray + UHD (AACS):** require a `keydb.cfg`
  (default `~/.config/freemkv/keydb.cfg`). Fetch one and drop it there,
  or point `update-keys` at a URL:

  ```bash
  freemkv update-keys --url <keydb-url>
  ```

### Reading the optical drive without root

On Linux the CLI reads the drive via SCSI generic (`/dev/sr0` / the
matching `/dev/sg*`). Membership in the `cdrom` group is normally enough:

```bash
sudo usermod -aG cdrom "$USER"
# log out / back in (or `newgrp cdrom`) for the group to take effect
```

If your distro doesn't grant the `cdrom` group access to the SCSI
generic node, install a udev rule (see autorip's INSTALL.md for the rule
text — the same rule covers the CLI).

## macOS in CI / note on builds

macOS CLI binaries are produced in CI on `macos-latest` runners for both
Intel (`x86_64-apple-darwin`) and Apple Silicon
(`aarch64-apple-darwin`). The CLI is portable across platforms; only
**autorip** is Linux-only (it talks to the kernel SCSI generic layer and
udev for live-drive detection), so there is no macOS autorip build.
