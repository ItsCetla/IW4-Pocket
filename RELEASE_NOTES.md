# IW4 Pocket 0.3.4

Test build aimed at phones with a 3 GB memory grant (iPhone 13 Pro Max, iPhone 15, SideStore with a free account), where `mp_rust` was closed by iOS while the map prepared its graphics. **Not yet tested on a phone.** Please send `iw4l-boot.log` and `iw4l-memory-settings.txt`.

**Memory**
- **Lean shaders**, on by default on the Low profile: each shader frees its compiled data once its pipelines are built, instead of keeping it for the whole run. Measured on a Mac with phone settings: about 790 MB less. By [treuenten](https://github.com/treuenten) (cherry-picked, authorship kept).
- **Replay clip buffer** is off on iPhone (about 150 MB less in a match). By treuenten.

**Display** (by treuenten)
- The display stays at 60 Hz for the first minute of a match, then asks for 120 Hz. This removed freezes of 85-200 ms at the start of matches on a 17 Pro Max.
- Optional **render scale** (0.5 to 0.85 of the screen's resolution).

**Bots and startup commands**
- `iw4l-env.txt` accepts `IW4L_CMDS=map mp_rust; bot add 5` again (only `map`, `bot`, `set`, `seta`; at most 200 characters and 8 commands). Fixes issue #4.
- New row in IW4 Pocket options: **Start a match with bots** (Rust, Favela, Estate, Terminal, 5 bots).

**IW4 Pocket options**: new rows for lean shaders, replay clip buffer, 120 Hz delay, render scale and the bot match. Every row starts on Auto and applies after a restart.

**Logs**
- `window:` line with the drawable size, points and scale, to diagnose a picture that fills only a corner of the screen (issue #8).
- Memory is logged every 8 MiB when less than 300 MiB are left, so the last seconds before iOS closes the app are visible.
- A run closed by iOS keeps its log as `iw4l-boot.crash.log`.
- The problem report asks how you installed the app.
- A rejected `iw4l-env.txt` line now says which setting it was.

Known: on a 3 GB grant, `mp_rust` may still run out of memory. If it does, the new log lines show where.
