# IW4 Pocket 0.3.6

**Not yet tested on a phone.** Adds bots without the console, an option to leave out weapon camo skins, and builds on 0.3.5 (free spare memory).

**Bots** (issues #4 and #6)
- One setting, three places, always in step: **Options > IW4 Pocket options > Bots in every match**, the new **BOTS** row in the lobby before a match (game setup), and `IW4L_BOTS=5` in `iw4l-env.txt`. 0 to 10 bots, added when a match starts.
- `IW4L_CMDS=map mp_rust; bot add 5` also works again (only `map`, `bot`, and `set`/`seta` on match and bot rules).
- Credits: `IW4L_BOTS` by [LemonMoo](https://github.com/LemonMoo) (Cade Palmer), the lobby bot row by [AdrianTomasDiez](https://github.com/AdrianTomasDiez). Both cherry-picked with their authorship. I changed them to share one value and capped the lobby row at 10 bots.

**Memory**
- **Weapon camo skins**: new row *Weapon camo skins* in IW4 Pocket options (Auto keeps them, **Skip** leaves them out). Idea and first code by [mk2rs2000](https://github.com/mk2rs2000), who skipped them on iOS; here it is a setting, off by default, so nobody loses camos without choosing to. The log has `camouflage:` and memory lines to compare.
- From 0.3.5: **Free spare memory** hands unused allocator pages back to iOS when little is left (log line `heap relief:`).
- For phones with about 3 GB for the app: set **Render scale** to 0.67 or 0.5, and try **Weapon camo skins: Skip**.

Please send `iw4l-boot.log` (and `iw4l-boot.crash.log` if the app closes) and say how you installed the app.
