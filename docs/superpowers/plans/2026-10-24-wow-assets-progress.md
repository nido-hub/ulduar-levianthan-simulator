# SDD ledger — plan: docs/superpowers/plans/2026-10-24-wow-assets.md

Ruling: work in place — `git rev-parse` confirms this is not a Git repository, so worktrees/commit-based helpers cannot apply; preserve existing files and use new `assets/wow` paths — no Git rollback available.

Pre-flight: Task 1 emits `wow/*.glb` and `wow/ui/*`; Tasks 2 and 3 consume these relative asset paths. Keep existing procedural `models/demolisher.glb` untouched. No interface conflict.

Baseline: `cargo test --workspace --locked` passed 65 tests.
Task 1: in progress. StormLib 9.40 downloaded to approved temporary directory with published SHA-256 verification. Unicode DLL requires wide archive paths; internal MPQ names remain narrow strings. Archive listing confirmed all four model families and neutral Ulduar textures.

Task 1: conversion implemented; 8 Python tests pass (local MPQ integration separately enabled and passed). All four GLBs contain embedded images, skinning and Idle/Drive clips; local font and genuine SpellIcon resources copied. Runtime resource total approximately 5 MB for models. Native ABI failure traced to C++ `bool` (8-bit return), fixed and covered by an actual archive missing-file test.
Task 2: native integration implemented. `cargo test --workspace --locked`: 67 passed. First GPU combat smoke (demo): PASS, 10/10 scenes and animation players loaded, 57.7 FPS smoothed; inspected screenshot. Other roles and final HUD still to verify.
Ruling: retain standard diffuse/emissive rendering, omit environment-reflection passes and projected smoke cards — these depend on WoW-specific shaders and duplicating them as opaque geometry is incorrect — subtle reflection/particle fidelity is lower than the original client.

Task 1: complete — `python -m unittest discover -s tools -v` with WOW_STORMLIB set: 10/10 pass, including real archive access, malformed ranges, alias cycles and independent bind-pivot translations. Converter executed successfully; PNG previews and all GLB metadata inspected.
Task 2: complete — five GPU combat role tests passed, 10 scenes/animation players each. Demo, passenger, siege and moto captures inspected. Legacy fallback and native menu tests also PASS.
Task 3: implemented and verified — 70/70 Rust tests pass. The copied executable/assets/settings ran from an independent temporary package at 960x640, working directory outside both source and client; report confirms temporary asset path. Native mouse window messages activated the third moto ability (cooldown 3.5 seconds at capture), proving action-bar picking and dispatch. Ordinary desktop injection attempts were unreliable because of Windows focus handling; this is a harness limitation, not accepted as click evidence. Source settings were compared byte-for-byte and preserved. Documentation updated; final independent review pending, then final rebuild/format check.

Final review: independent read-only reviewer found no critical issue and one important importer boundary issue (NaN skeletal values and missing required animation were accepted). Reproduced with new fixtures before fixing. Final: fixed invalid model conversion — finite accessor validation, strict JSON, required skeleton/Idle/Drive and positive clip duration; regression fixtures RED→GREEN, Python suite 13/13 (real MPQ included). Four actual models reimported successfully. Rust suite remains 70/70; formatting check passes.
Final verification: delivered executable rebuilt and separately launched from temporary package, exact delivered executable SHA-256 recorded in `dist/validation-standalone-final.txt`; 10/10 scenes and players, PASS. Final art preview PASS. Extended passenger GPU combat: 76.1 seconds, one shutdown, 10/10 loaded, PASS. No review minors deferred.
Final: Ruling: do not change combat balance or AI — visual-only request, preserve established mechanics — existing pedagogical fidelity limits remain.
Final: Ruling: support the existing Windows/3.3.5a target only — local resources and current application target — other client versions/platforms are unsupported, not validated.
Final: Ruling: do not imply redistribution permission — preserve provenance and warn about Blizzard rights; legal clearance is outside a technical import — redistribution requires the appropriate authorization.

Task 3: complete — final `Construire.ps1` passed the full 70-test Rust suite and build. Final `cargo fmt --all -- --check` passed. Post-build hash verification confirms all 38 manifest-listed resources match in source and delivery, manifests match, and the delivered executable is exactly the standalone-tested binary. Independent review's sole important finding fixed and covered; 13 Python tests passed. No Git integration/cleanup applies; retain design, plan and ledger as the local execution record.
