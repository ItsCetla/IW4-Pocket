#!/usr/bin/env python3
"""Regenerates source.json, the SideStore/AltStore source for IW4 Pocket.

Reads the GitHub releases and lists every one that has IW4.Pocket.ipa. Pre-releases get
"TEST" in their notes so they are easy to tell apart. The newest stable release is listed
first, because that is the one SideStore offers as the latest.
Run from the repository root: python3 tools/gen_source.py
"""
import json
import urllib.request

REPO = "MarkusSela/IW4-Pocket"
RAW = f"https://raw.githubusercontent.com/{REPO}/main"


def get(url):
    req = urllib.request.Request(url, headers={"Accept": "application/vnd.github+json", "User-Agent": "gen-source"})
    return json.load(urllib.request.urlopen(req))


releases = get(f"https://api.github.com/repos/{REPO}/releases?per_page=30")
versions = []
for rel in releases:
    ipa = next((a for a in rel["assets"] if a["name"] == "IW4.Pocket.ipa"), None)
    if ipa is None or rel["draft"]:
        continue
    test = rel["prerelease"]
    note = (rel.get("body") or "").strip().splitlines()
    summary = next((l.strip() for l in note[1:] if l.strip() and not l.startswith("#")), "")
    summary = summary.replace("**", "").replace("`", "")
    versions.append({
        "version": rel["tag_name"].lstrip("v"),
        "buildVersion": None,
        "date": rel["published_at"],
        "size": ipa["size"],
        "downloadURL": ipa["browser_download_url"],
        "localizedDescription": ("TEST VERSION - not checked on enough phones. " if test else "") + (summary[:297] + "..." if len(summary) > 300 else summary),
        "minOSVersion": "15.0",
        "_test": test,
        "_date": rel["published_at"],
    })
for v in versions:
    v.pop("buildVersion")
# stable first (newest first), then tests (newest first)
versions.sort(key=lambda v: (v["_test"], [-ord(c) for c in v["_date"]]))
for v in versions:
    v.pop("_test")
    v.pop("_date")
    if v["localizedDescription"].startswith("TEST"):
        v["version"] = v["version"]  # keeps the real version number
source = {
    "name": "IW4 Pocket",
    "identifier": "com.markussela.iw4l.source",
    "subtitle": "Modern Warfare 2 on iPhone",
    "description": "Releases of IW4 Pocket. Versions marked TEST are for trying new things.",
    "iconURL": f"{RAW}/ios/icons/AppIcon-1024.png",
    "website": f"https://github.com/{REPO}",
    "tintColor": "#2f3a2f",
    "apps": [{
        "name": "IW4 Pocket",
        "bundleIdentifier": "com.markussela.iw4l",
        "developerName": "MarkusSela",
        "subtitle": "Modern Warfare 2 on iPhone",
        "localizedDescription": "A port of MW2 (2009) to iPhone. You need your own copy of the PC game files.",
        "iconURL": f"{RAW}/ios/icons/AppIcon-1024.png",
        "tintColor": "#2f3a2f",
        "category": "games",
        "versions": versions,
    }],
    "news": [],
}
with open("source.json", "w") as out:
    json.dump(source, out, indent=2, ensure_ascii=False)
    out.write("\n")
print(len(versions), "versions:", [v["version"] for v in versions])
