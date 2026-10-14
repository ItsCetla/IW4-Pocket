# IW4 Pocket 0.3.5

For phones with a 3 GB memory grant. On an iPhone 13 Pro Max, 0.3.4 loaded `mp_rust` and a match ran for about a minute, then iOS closed the app: the allocator was holding about 450 MiB of free pages that the app was not using but iOS still counted. **Not yet tested on a phone.**

**Memory**
- **Free spare memory** (new, on by default on iPhone). When iOS leaves less than 600 MiB, the app asks the allocator to hand its unused pages back, at most every 3 seconds. The log shows each call: `heap relief: footprint 3040 -> 2990 MiB (allocator says N MiB) in 4 ms | calls, total gained`. If it causes a hitch, turn it off in IW4 Pocket options > Free spare memory, or with `IW4L_HEAP_RELIEF=0`.

**Settings and advice**
- Phones with about 3 GB for the app: set **Render scale** to 0.67 or 0.5 (IW4 Pocket options). On an iPhone 13 Pro Max, 0.5 leaves 168 MiB at the start of a match against 94 MiB at full size.
- Lean shaders stays on by default on the Low profile. On the same phone it is the difference between a map that loads and one that does not.

**Logs**
- `heap relief:` lines (see above). Please send `iw4l-boot.log`; if the app closes, also `iw4l-boot.crash.log`.
- The problem report asks how you installed the app.

Known: on a 3 GB grant the app may still be closed during a match. If it is, the logs show how long it ran and how much each relief call gave back.
