# IW4 Pocket 0.3.7c — build 21

**Experimental prerelease. Not verified on a phone.** Menu, bot names and options changes. It does not fix the memory crash: that is still unproven and the 0.3.7b log showed about the same footprint (~3067 MiB).

## IW4 Pocket options menu
- Rows are spaced further apart and each description sits under its option, over two lines, saying what it does and what it costs.
- Render scale now offers 0.4, 0.3 and 0.2 besides the earlier values (Auto/On/Off style, adjustable by hand). Lower values draw fewer pixels and use less GPU memory, and look blurrier. The saving is an estimate from pixel counts, not a measurement.

## Bots
- Names are drawn again every time the number of bots changes (lobby add/remove); the same number keeps the same names. Lobby and match use the same order.
- Joke clan tags (83) such as `[420]` are put in front of some names. A tag is added only if `[TAG]name` still fits the 15 bytes the protocol allows; the name itself is never cut or changed. Existing names are untouched.

## Menu and SideStore
- Main menu: the release name is shown small in the bottom-right corner.
- SideStore source: the newest stable release stays first, newer pre-releases are listed after it marked TEST, and older pre-releases and older stable releases are hidden.

## Diagnostics
- `mem:` lines now include the memory outside the Rust heap (footprint minus reserved heap), to tell whether growth comes from graphics memory.

## Fix
- Test `nested_client_and_objective_strings_grow_the_estimate` compared against the wrong baseline; the assertion was corrected. The estimator itself is unchanged.

## Not included
- Weapons on demand: on Low the weapon cache is already 0 MiB (`IW4L_FPV_RETAIN_MIB=0`), so there is nothing proven to free.
- No allocator replacement.

## Version
- Label **0.3.7c**; Cargo `0.3.7-beta.2`; bundle build **21**.

Verification scope: Python source-contract checks, `sim` memory tests and the bot-name logic were run on the server. `bots`, `console` and `launcher` are compiled only by the iOS CI. Nothing here has been tried on a device.
