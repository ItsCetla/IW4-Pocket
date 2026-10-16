# IW4 Pocket 0.3.7

**Not tested on a phone yet.** This build adds touch controls, a BOTS row in the Private Match menu, and more memory measurements. It does not fix the memory limit: on a phone with about 3 GB for the app, a match can still end when iOS closes the app. Please say what you see.

**On-screen controls**
- Options -> IW4 Pocket options -> **On-screen controls**: Auto, On or Off. Auto shows them during a match when no controller is connected.
- Left thumb: move stick (push it all the way forward to sprint). Drag anywhere on the right to look. Round buttons: FIRE, AIM, JUMP, RELOAD, CROUCH, FRAG, TACT, KNIFE, SWAP, BOARD and MENU.
- They send keyboard and mouse input, not a fake controller, so no controller icons appear. The button positions are a first guess and may need moving: send a screenshot.

**Bots in the lobby**
- Private Match menu: a **BOTS** row under GAME SETUP (host only). Tap to add one, 0 to 10. The bots are listed in the player list on the right. The earlier BOT and BOT COUNT rows inside Game Setup overlapped the original menu and are gone.

**Memory log (for finding what uses memory)**
- The log now has `keep xmodel`, `keep xanim` and `keep weapon` rows: how much of the weapon growth is the models and animations they point to.
- The spawn and killcam lines now end with the footprint and heap at that moment, so each life can be compared.
- A small fix for muzzle flashes and effects that stopped after the first minutes with bots, and one that frees the last match's body animations before the next map loads. Neither is measured yet.

**Not in this release:** a different memory allocator (not tried on iOS), and weapons loaded on demand (the weapon rows will say where the memory is first).

Please send `iw4l-boot.log` (and `iw4l-boot.crash.log` if the app closes) and say how you installed the app.
