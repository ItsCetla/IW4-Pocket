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
     "Auto picks it from your memory.",
     [("Auto", "auto"), ("Low", "low"), ("Medium", "mid"), ("High", "high")]),
    ("ui_pocket_scale", "Render scale",
     "Lower is faster and blurrier.",
     [("Auto", "auto"), ("0.5", "0.5"), ("0.67", "0.67"), ("0.75", "0.75"), ("0.85", "0.85")]),
    ("ui_pocket_pace", "Frame limit",
     "Caps frames per second.",
     [("Auto", "auto"), ("30", "30"), ("40", "40"), ("60", "60")]),
    ("ui_pocket_texture", "Texture size",
     "Smaller uses less memory.",
     [("Auto", "auto"), ("128", "128"), ("256", "256"), ("512", "512"), ("1024", "1024")]),
    ("ui_pocket_lean", "Lean shaders",
     "Frees shader memory at load.",
     [("Auto", "auto"), ("On", "on"), ("Off", "off")]),
    ("ui_pocket_camo", "Weapon camo skins",
     "Skip did not save memory.",
     [("Auto", "auto"), ("Skip", "skip"), ("Keep", "keep")]),
    ("ui_pocket_latency", "Frames in flight",
     "2 is smoother, uses more.",
     [("Auto", "auto"), ("1", "1"), ("2", "2")]),
    ("ui_pocket_promotion", "120 Hz in matches",
     "Smoother on 120 Hz phones.",
     [("Auto", "auto"), ("On", "on"), ("Off", "off")]),
    ("ui_pocket_promodelay", "120 Hz delay (s)",
     "Secs at 60 Hz before 120.",
     [("Auto", "auto"), ("0", "0"), ("30", "30"), ("60", "60"), ("120", "120")]),
    ("ui_pocket_bc", "Compressed textures",
     "Off uses more memory.",
     [("Auto", "auto"), ("Off", "off")]),
    ("ui_pocket_shadows", "Dynamic shadows",
     "Auto: off if not supported.",
     [("Auto", "auto"), ("Off", "off"), ("On", "on")]),
    ("ui_pocket_clip", "Replay clip buffer",
     "Last 45 s for clips.",
     [("Auto", "auto"), ("On", "on"), ("Off", "off")]),
    ("ui_pocket_touch", "On-screen controls",
     "Auto: shown with no controller.",
     [("Auto", "auto"), ("On", "on"), ("Off", "off")]),
]


DESC_X = 122      # where the grey description starts, after the longest name
DESC_RIGHT = 74   # room kept free for the value on the right


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
    # Same row as the name, starting after it and stopping before the value on the right.
    hf["rect"] = {"x": 232 + DESC_X, "y": y, "w": 344 - DESC_X - DESC_RIGHT, "h": 20}
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


PAGE = 6  # rows on the first page
LABELS = {1: "More options >", 2: "< Back to main options"}


def page_menu(number, rows, first_index):
    copies = []
    for i, option in enumerate(rows):
        copies += option_rows(*option, i)
    y_info = 28 + 23 * len(rows) + 4
    copies.append(dynamic_label(f"pocket_info{number}", "ui_pocket_info", y_info))
    nxt = copy.deepcopy(multi_t)
    nf = nxt["fields"]
    nf.update({"item_type": 1, "dvar": "", "choices": [], "text_key": LABELS[number],
               "name": f"pocket_page{number}",
               "rect": {"x": 232, "y": y_info + 22, "w": 344, "h": 20}})
    target = "options_pocket2" if number == 1 else "options_pocket"
    nf["handlers"]["action"] = [{"Script": f"play mouse_click; close self; open {target};"}]
    copies.append(nxt)
    y_next = y_info + 44
    if number == 1:
        button = copy.deepcopy(multi_t)
        bf = button["fields"]
        bf.update({"item_type": 1, "dvar": "", "choices": [], "text_key": "Send logs >",
                   "name": "pocket_send_logs",
                   "rect": {"x": 232, "y": y_next, "w": 344, "h": 20}})
        bf["handlers"]["action"] = [{"Script": "play mouse_click; setdvar ui_pocket_send_logs 1;"}]
        copies.append(button)
        caption = copy.deepcopy(label_t)
        caption["fields"].update({"text_key": "Report a problem on GitHub",
                                  "name": "pocket_send_logs_label",
                                  "rect": {"x": 232, "y": y_next, "w": 344, "h": 20}})
        copies.append(caption)
        y_next += 22
    copies.append(dynamic_label(f"pocket_hint{number}", "ui_pocket_hint", y_next))
    menu = copy.deepcopy(ctrl)
    menu["name"] = "options_pocket" if number == 1 else "options_pocket2"
    for override in menu["item_overrides"]:
        if override["name"] == "video_title":
            override["fields"]["text_key"] = "IW4 Pocket options" if number == 1 else "IW4 Pocket options (more)"
    menu["item_copies"] = copies
    return menu


menus = [page_menu(1, OPTIONS[:PAGE], 0), page_menu(2, OPTIONS[PAGE:], PAGE)]
with open(f"{ROOT}/crates/ui/menus/pocket.json", "w") as out:
    json.dump(menus, out, indent=1, ensure_ascii=False)
    out.write("\n")
print(f"{len(menus[0]['item_copies'])} + {len(menus[1]['item_copies'])} items")
