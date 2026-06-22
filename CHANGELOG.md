# Changelog

All notable changes to the `freemkv` CLI are documented here. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and the
project follows semantic versioning.

## [1.0.0-rc.1] - UNRELEASED

First release candidate for 1.0.

### Added

- Keyless DVD/CSS ripping: a CSS-protected DVD now decrypts with no key
  database. The title key is recovered from the scrambled disc data via
  libfreemkv's Stevenson attack, so `disc://` to `mkv://` works out of the
  box on a DVD.
- Muxing a raw, still-scrambled CSS ISO (`iso://`): `Disc::scan_image`
  recovers the title key from the image directly, so a raw CSS ISO no longer
  needs to be pre-decrypted before muxing.
- `--log-level N` sets the log verbosity: `1` = warnings/errors (default),
  `2` = info, `3` = debug, `4` = trace. `--log-file PATH` adds a non-blocking
  file sink alongside stderr. `RUST_LOG` overrides `--log-level` if set.
  Logs go to stderr so stdout stays pipe-clean for `mkv://`/`m2ts://` piping.
- Static-binary releases: each tagged release attaches a single static binary
  per platform (Linux x86_64/aarch64, macOS Intel/Silicon, Windows) with a
  `.sha256` checksum, alongside the source archives. See `INSTALL.md`.

### Changed

- Correct DVD video output: the MPEG-2 Program-Stream access-unit reassembler
  in libfreemkv buffers elementary-stream bytes across PES packets and emits
  exactly one coded picture per MKV block with reconstructed presentation
  timestamps, replacing the previous per-PES framing that produced corrupted
  DVD video.
- MKV output records `freemkv <version>` in the Muxing/Writing application
  fields.
- Built on libfreemkv 1.0.0-rc.1: HEVC/H.264/VC-1 param-set keyframe
  correctness, short-read rejection, `BlockDuration` timescale fix, and CSS
  key redaction in logs.

## [0.31.0] - 2026-06-08

Hardening and correctness release across argument parsing, the `info`
drive-report writer, output formatting, and interrupt handling.

### Fixed

- `info`: escape vendor/firmware strings written into `drive.toml` so an
  unusual INQUIRY / GET CONFIGURATION value cannot produce invalid TOML.
- Argument parsing: a flag immediately before a URL is no longer mistaken for
  the URL; unknown flags now error instead of being silently ignored; and
  `--language` no longer consumes a following flag as its value.
- Reject a multi-title disc rip to a single-file destination instead of
  silently creating a directory; corrected the main-title loss estimate.
- SIGINT now uses `sigaction`, so a second Ctrl-C reliably exits 130 on musl.

### Changed

- Release profile now builds with thin LTO and a single codegen unit.

## [0.29.0] - 2026-06-05

### Fixed

- A resolved AACS key is now verified against disc content before muxing, so a
  stale or wrong key can no longer silently produce garbage output.
- `iso://` mux fails fast with a clear message when no usable AACS key is
  available, instead of writing an unusable MKV.
- `info iso://` lists titles without requiring a key, instead of failing.
- `--raw` is rejected for mux output and dropped from the no-key error message.

## [0.28.1] - 2026-06-04

### Changed

- AACS keys are resolved via the key-source layer rather than an inline keydb
  reader, decoupling key handling from the mux path. A live-drive scan supplies
  the AACS handshake credentials, and an `iso://` to `mkv://` remux resolves and
  passes through the unit keys.

## [0.27.4] - 2026-06-04

### Changed

- AACS unit-encryption detection reads the raw MPEG-TS sync bytes instead of
  header flag bits, so the decrypt gate and key validation agree on what
  "encrypted" means.

### Added

- Unit tests for `info` formatting helpers (base64, date, hex dump).

## [0.26.1] - 2026-05-22

### Changed

- Synced to the matching `libfreemkv` recovery and mux improvements; CLI option
  structs updated to track new library fields.

## [0.25.0] - 2026-05-19

### Changed

- Mux throughput pipeline ("highway") in `libfreemkv` — a threaded
  read/decrypt, demux, and codec-parse pipeline that substantially raises
  file-backed mux speed.

## [0.23.0] - 2026-05-16

### Changed

- Multipass recovery refinements: targeted retry passes over bad ranges with
  per-sector recovery timeout and range bisection.

## [0.20.0] - 2026-05-13

### Added

- Stream-URL CLI: `freemkv <source> <dest>` over `disc://`, `iso://`, `mkv://`,
  `m2ts://`, `network://`, `stdio://`, and `null://`, with `info` for disc and
  file metadata and `update-keys` for fetching a keydb.
