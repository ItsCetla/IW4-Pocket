# IW4 Pocket 0.3.7b — build 20

**Experimental prerelease. Not verified on a phone.** This changes snapshot archive memory accounting; it is not a confirmed fix for iOS closing a match. No device memory saving or stability improvement has been measured yet.

## Snapshot archive
- The authority archive now estimates owned snapshot memory recursively: inline structs, vector/string capacities, client metadata, HUD/configstrings, objectives, world/entity state, model and animation descriptors. These descriptors are not full copies of mesh or texture assets.
- The archive retains at most 1,200 ticks and 32 MiB of **accounted payload and container capacity**. Allocator metadata/rounding are not measured; BTreeMap nodes use a conservative allowance. This is not a process-footprint cap.
- Old frames are evicted before cloning a new snapshot. A single frame exceeding the budget is rejected rather than retained without a bound. Duplicate/old ticks are rejected before cloning. Clearing releases archive container capacity.
- Killcam/spectator lookup continues to use the retained history. A heavier snapshot may shorten that history; requests older than it clamp to the oldest retained frame. This does not remove killcam or spectator support.
- `frame_archive:` samples report frames, records, accounted/peak bytes, evictions, oversized rejections, retained duration and tick range, alongside the existing heap/footprint memory tag on iOS. Samples occur on the first accepted/new-tick submission, every 200 submissions (about 10 simulated seconds at 20 Hz), and before clearing. They are estimates, not allocator measurements.

## Private match and bots
- Rust is preferred on the initial map selection when installed. If unavailable, the first installed IW4 map is used. An existing map choice is preserved.
- The 60 existing bot names remain, with all 88 approved English nicknames added. Name order is randomized once per app session and remains stable across lobby refreshes and match loading; lobby and in-game bots share the order. If OS randomness fails, the original stable order is used and a warning is logged.
- The game protocol allows 15 name bytes plus a NUL terminator. The pool retains the full approved names, but both lobby and game use the same ASCII prefix when necessary: `NoScopeNoBitches` → `NoScopeNoBitche`; `ColossalSkillIssue` → `ColossalSkillIs`.

## Version and verification scope
- Display/release label: **0.3.7b**; Cargo: `0.3.7-beta.1`; iOS numeric short version: `0.3.7`; bundle build: **20**.
- Regression tests were added for recursive memory estimates, archive budgets/eviction/oversized frames/clear/lookup, and bot names/order. Source-contract checks are separate from compiled Rust tests; passing them does not prove device gameplay.
- No allocator replacement, speculative asset unloading, or on-demand weapon loading is included. Existing touch controls and IW4 Pocket options are unchanged.

For phone verification, play for at least one simulated minute with bots, test death/killcam and return to the lobby, then send `iw4l-boot.log` (and `iw4l-boot.crash.log` if iOS closes the app). Archive telemetry must be correlated with heap and footprint before attributing memory pressure or claiming savings.
