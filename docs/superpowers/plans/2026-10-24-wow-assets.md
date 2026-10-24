# WoW Local Assets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Use copied local WoW vehicles and HUD resources in the standalone trainer.
**Architecture:** Build-time Python conversion using StormLib (read-only MPQ access) and installed Pillow; embedded GLB textures, PNG UI images. Native Bevy loading with original procedural fallback; no runtime WoW dependency.
**Tech Stack:** Rust/Bevy 0.20, Python 3.11, Pillow, build-time StormLib.
**Spec:** `docs/superpowers/specs/2026-10-24-wow-assets-design.md`

## Global Constraints
- Do not modify the WoW client or overwrite user settings.
- Runtime assets must reside under the executable's assets directory.
- Preserve driving/combat behavior and original assets.
- No new runtime dependency for extraction; retain provenance and redistribution warning.

## Review Focus
- Missing or malformed archives/models must fail conversion explicitly, not produce corrupt output.
- Correct patch precedence and replaceable creature textures.
- Coordinates, ground height, scale, UVs, transparency and alternate geosets must render correctly.
- Role changes and asynchronous failures must not show two models or hide the fallback.
- Packaging must work outside the source tree; UI picking and cooldown/resource feedback remain functional.

### Task 1: Targeted asset conversion
**Files:** `tools/import_wow.py`, `tools/test_import_wow.py`, `Importer-WoW.ps1`, `assets/wow/*`.
**Interfaces:** CLI `python tools/import_wow.py --client PATH --stormlib DLL`; outputs `assets/wow/{demolisher,siege,moto,leviathan}.glb`, UI images and provenance manifest.
- [x] Inspect archive lists and model structures; select original Ulduar vehicles and their texture variants.
- [x] Add failing tests for bounded binary reads, SKIN indices, coordinate conversion and self-contained GLB output; run tests.
- [x] Implement targeted MPQ/M2/SKIN/BLP conversion; embedded textures and selected model geometry. Use existing Pillow for BLP decoding.
- [x] Run converter and tests, inspect output geometry/material metadata and image previews.

### Task 2: Native model integration
**Files:** `crates/client/src/{models,scene,raid,main}.rs`.
**Interfaces:** Load converted GLB scenes for all actor types; hide procedural visuals only after successful load. Existing `ModelStatus` remains compatible with validation.
- [x] Add failing tests validating the four GLBs and embedded images/no external URIs.
- [x] Integrate player, allied and boss scenes while preserving actor roots and fallback behavior.
- [x] Run `cargo test --workspace --locked` and a native render test for each playable vehicle.

### Task 3: HUD and standalone delivery
**Files:** `crates/client/src/ui.rs`, `README.md`, `assets/README.md`, `Construire.ps1`, validation scripts as needed.
**Interfaces:** Local copied font/images; compact clickable five-slot action bar, role-dependent icons, health/resource bars and cooldown labels.
- [x] Add mapping/ratio tests for every role, empty slots, depleted health/resource and cooldown state.
- [x] Integrate WoW textures and icons, readable labels and actual health/resource fills; preserve options and keyboard actions.
- [x] Format, run unit tests, build and run GPU tests including menu and original fallback.
- [x] Copy the package to a separate temporary directory, launch with unrelated working directory and confirm copied resources load without client lookup.
- [x] Update documentation with actual results, limits and provenance; review final diff and mark completion.

No commits: this workspace is not a Git repository. User explicitly requested inline implementation without another approval round.
