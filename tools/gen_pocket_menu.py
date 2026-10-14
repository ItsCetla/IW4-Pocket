#!/usr/bin/env python3
"""Regenerates crates/ui/menus/pocket.json (the "IW4 Pocket options" menu).

Rows copy the layout of controller.json, so the menu looks like the game's own.
The option list must match OPTIONS in crates/console/src/pocket_options.rs
(tools/test_ios_memory_contract.py checks that they agree).
Run from the repository root: python3 tools/gen_pocket_menu.py
"""
import copy
import json

ROOT = "."
ctrl = json.load(open(f"{ROOT}/crates/ui/menus/controller.json"))[0]
multi_t = ctrl["item_copies"][0]
label_t = ctrl["item_copies"][1]

# dvar, label, one-line description, [(shown, value)]
OPTIONS = [
    ("ui_pocket_tier", "Device profile",
     "Auto picks how much to load from your phone's memory.",
     [("Auto", "auto"), ("Low", "low"), ("Medium", "mid"), ("High", "high")]),
    ("ui_pocket_texture", "Texture size",
     "Smaller uses less memory and looks blurrier.",
     [("Auto", "auto"), ("128", "128"), ("256", "256"), ("512", "512"), ("1024", "1024")]),
    ("ui_pocket_latency", "Frames in flight",
     "2 is smoother but uses more graphics memory.",
     [("Auto", "auto"), ("1", "1"), ("2", "2")]),
    ("ui_pocket_promotion", "120 Hz in matches",
     "Smoother on 120 Hz phones, uses more battery.",
     [("Auto", "auto"), ("On", "on"), ("Off", "off")]),
    ("ui_pocket_pace", "Frame limit",
     "Caps frames per second. Auto means no cap.",
     [("Auto", "auto"), ("30", "30"), ("40", "40"), ("60", "60")]),
    ("ui_pocket_bc", "Compressed textures",
     "Auto uses them when the GPU can. Off uses more memory.",
     [("Auto", "auto"), ("Off", "off")]),
    ("ui_pocket_shadows", "Dynamic shadows",
     "Auto turns them off on phones that cannot draw them.",
     [("Auto", "auto"), ("Off", "off"), ("On", "on")]),
    ("ui_pocket_lean", "Lean shaders",
     "Frees shader memory while loading. Auto is on for the Low profile.",
     [("Auto", "auto"), ("On", "on"), ("Off", "off")]),
    ("ui_pocket_clip", "Replay clip buffer",
     "Keeps the last 45 s for the clip command. Uses memory. Auto is off.",
     [("Auto", "auto"), ("On", "on"), ("Off", "off")]),
    ("ui_pocket_promodelay", "120 Hz delay (s)",
     "Seconds at 60 Hz after a match starts, before 120 Hz. Auto is 60.",
     [("Auto", "auto"), ("0", "0"), ("30", "30"), ("60", "60"), ("120", "120")]),
    ("ui_pocket_scale", "Render scale",
     "Draws below screen resolution. Faster, blurrier. Auto is full size.",
     [("Auto", "auto"), ("0.5", "0.5"), ("0.67", "0.67"), ("0.75", "0.75"), ("0.85", "0.85")]),
    ("ui_pocket_relief", "Free spare memory",
     "Gives unused memory back to iOS when it runs low. Auto is on.",
     [("Auto", "auto"), ("On", "on"), ("Off", "off")]),
    ("ui_pocket_bots", "Bots in every match",
     "Bots added when a match starts. Also in the lobby. Auto is none.",
     [("Auto", "auto"), ("1", "1"), ("2", "2"), ("3", "3"), ("4", "4"), ("5", "5"), ("6", "6"), ("7", "7"), ("8", "8"), ("9", "9"), ("10", "10")]),
    ("ui_pocket_camo", "Weapon camo skins",
     "Skip leaves the camos out and saves memory. Auto keeps them.",
     [("Auto", "auto"), ("Skip", "skip"), ("Keep", "keep")]),
]


def hexname(name):
    return name.encode().hex()


def dyn(name):
    return f"op 38 s:{hexname(name)} op 1"


def option_rows(dvar, label, desc, choices, index):
    y = 28 + 23 * index
    control = copy.deepcopy(multi_t)
    f = control["fields"]
    f["dvar"] = dvar
    f["name"] = dvar
    f["choices"] = [list(c) for c in choices]
    f["rect"] = {"x": 232, "y": y, "w": 344, "h": 20}
    name = copy.deepcopy(label_t)
    nf = name["fields"]
    nf["text_key"] = label
    nf["name"] = f"{dvar}_label"
    nf["rect"] = {"x": 232, "y": y, "w": 344, "h": 20}
    hint = copy.deepcopy(label_t)
    hf = hint["fields"]
    hf["text_key"] = desc
    hf["name"] = f"{dvar}_desc"
    hf["text_scale"] = 0.22
    hf["rect"] = {"x": 232, "y": y + 15, "w": 344, "h": 12}
    hf["fore_color"] = [0.75, 0.75, 0.75, 1]
    return [control, name, hint]


def dynamic_label(name, dvar, y, scale=0.25):
    item = copy.deepcopy(label_t)
    f = item["fields"]
    f["text_key"] = ""
    f["text_exp"] = dyn(dvar)
    f["name"] = name
    f["text_scale"] = scale
    f["rect"] = {"x": 232, "y": y, "w": 344, "h": 20}
    f["fore_color"] = [0.75, 0.75, 0.75, 1]
    return item


copies = []
for i, option in enumerate(OPTIONS):
    copies += option_rows(*option, i)

y_info = 28 + 23 * len(OPTIONS) + 4
copies.append(dynamic_label("pocket_info", "ui_pocket_info", y_info))

button = copy.deepcopy(multi_t)
bf = button["fields"]
bf.update({"item_type": 1, "dvar": "", "choices": [], "text_key": "Send logs >",
           "name": "pocket_send_logs",
           "rect": {"x": 232, "y": y_info + 22, "w": 344, "h": 20}})
bf["handlers"]["action"] = [{"Script": "play mouse_click; setdvar ui_pocket_send_logs 1;"}]
copies.append(button)
caption = copy.deepcopy(label_t)
caption["fields"].update({"text_key": "Report a problem on GitHub",
                          "name": "pocket_send_logs_label",
                          "rect": {"x": 232, "y": y_info + 22, "w": 344, "h": 20}})
copies.append(caption)
copies.append(dynamic_label("pocket_hint", "ui_pocket_hint", y_info + 44))

menu = copy.deepcopy(ctrl)
menu["name"] = "options_pocket"
for override in menu["item_overrides"]:
    if override["name"] == "video_title":
        override["fields"]["text_key"] = "IW4 Pocket options"
menu["item_copies"] = copies
with open(f"{ROOT}/crates/ui/menus/pocket.json", "w") as out:
    json.dump([menu], out, indent=1, ensure_ascii=False)
    out.write("\n")
print(f"{len(copies)} items")
