# IW4 Pocket 0.3.6

**Tested on one phone** (iPhone 13 Pro Max, about 3 GB for the app): the lobby and in-match bot rows work and `mp_rust` loads, but a match still ends when iOS closes the app for memory after a few minutes. This release does not fix that. It adds bots you can add in the lobby and during a match, an option to leave out weapon camo skins, a shorter options menu, and new bot names.

**Bots** (issues #4 and #6)
- **Lobby:** a **BOTS** row in game setup, before the match (0 to 10).
- **During a match:** open the **Options** menu (Start) and tap **Add 1 bot** or **Add 5 bots**. Only the host adds bots; for everyone else the rows do nothing. If the rows do not show up, send `iw4l-boot.log`: it lists the menus it looked at.
- `IW4L_BOTS=5` in `iw4l-env.txt` still sets how many bots a match starts with, and `IW4L_CMDS=map mp_rust; bot add 5` works as before (only `map`, `bot`, and `set`/`seta` on match and bot rules). Neither has a row in the options menu.
- New bot names (2009-2010 style and a few memes), up to 15 characters.
- Credits: `IW4L_BOTS` by [LemonMoo](https://github.com/LemonMoo) (Cade Palmer), the lobby bot row by [AdrianTomasDiez](https://github.com/AdrianTomasDiez). Both cherry-picked with their authorship, changed to share one value.

**Options menu**
- **IW4 Pocket options** now has two pages: the first with the settings that matter most (profile, render scale, frame limit, texture size, lean shaders, weapon camo skins) and a second with the rest. "Free spare memory" is gone: in tests it gave back 0 MiB. Old `IW4L_HEAP_RELIEF` lines in `iw4l-env.txt` are ignored.
- **Weapon camo skins** (Auto keeps them, **Skip** leaves them out). The log has a `camouflage:` line saying what was kept or left out. Measured on the 13 Pro Max with the same map: Skip removed all 8360 + 8360 camo models, but memory after loading was the same (2511 MiB with Skip, 2512 MiB with the camos kept). So it is **not a memory saver**; it only leaves the camos out. Idea and first code by [mk2rs2000](https://github.com/mk2rs2000).

**SideStore source**
- Add `https://raw.githubusercontent.com/MarkusSela/IW4-Pocket/main/source.json` in SideStore (Sources, +). Test versions say TEST.

For phones with about 3 GB for the app: set **Render scale** to 0.67 or 0.5. That helps a little; the app can still be closed by iOS during a match.

Please send `iw4l-boot.log` (and `iw4l-boot.crash.log` if the app closes) and say how you installed the app.
