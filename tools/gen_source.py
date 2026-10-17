#!/usr/bin/env python3
"""Regenerates source.json, the SideStore/AltStore source for IW4 Pocket.

Reads the GitHub releases and lists the ones that have IW4.Pocket.ipa.

* The newest stable release is always listed first: that is the one SideStore offers as
  the latest, so a test build never hides it.
* Pre-releases are listed after it, marked TEST in their notes, but only the ones newer
  than that stable release. Once a stable release comes out, older pre-releases and older
  stable releases are dropped, so nobody installs an old build by mistake.
* If there is no stable release yet, only the newest pre-release is listed.
Run from the repository root: python3 tools/gen_source.py
"""
import json
import re
import urllib.request

REPO = "MarkusSela/IW4-Pocket"
RAW = f"https://raw.githubusercontent.com/{REPO}/main"


def get(url):
    req = urllib.request.Request(url, headers={"Accept": "application/vnd.github+json", "User-Agent": "gen-source"})
    return json.load(urllib.request.urlopen(req))


def order_key(version):
    """0.3.7b -> (0, 3, 7, 'b'); 0.3.7 -> (0, 3, 7, ''); 0.3.7c sorts after 0.3.7b after 0.3.7."""
    m = re.match(r"^v?(\d+)\.(\d+)\.(\d+)([A-Za-z]*)", version)
    if not m:
        return (0, 0, 0, "")
    return (int(m[1]), int(m[2]), int(m[3]), m[4])


def pick(candidates):
    """Which releases the source lists. Each candidate: dict with `tag`, `test`, `date`."""
    stable = sorted((c for c in candidates if not c["test"]), key=lambda c: order_key(c["tag"]), reverse=True)
    tests = sorted((c for c in candidates if c["test"]), key=lambda c: order_key(c["tag"]), reverse=True)
    if not stable:
        return tests[:1]
    newest = stable[0]
    newer_tests = [c for c in tests if order_key(c["tag"]) > order_key(newest["tag"])]
    return [newest] + newer_tests


if __name__ == "__main__":
    releases = get(f"https://api.github.com/repos/{REPO}/releases?per_page=30")
    by_tag = {}
    for rel in releases:
        ipa = next((a for a in rel["assets"] if a["name"] == "IW4.Pocket.ipa"), None)
        if ipa is None or rel["draft"]:
            continue
        by_tag[rel["tag_name"]] = (rel, ipa)
    chosen = pick([{"tag": t, "test": r["prerelease"], "date": r["published_at"]} for t, (r, _) in by_tag.items()])
    versions = []
    for c in chosen:
        rel, ipa = by_tag[c["tag"]]
        test = rel["prerelease"]
        note = (rel.get("body") or "").strip().splitlines()
        summary = next((l.strip() for l in note[1:] if l.strip() and not l.startswith("#")), "")
        summary = summary.replace("**", "").replace("`", "")
        versions.append({
            "version": rel["tag_name"].lstrip("v"),
            "date": rel["published_at"],
            "size": ipa["size"],
            "downloadURL": ipa["browser_download_url"],
            "localizedDescription": ("TEST VERSION - not checked on enough phones. " if test else "") + (summary[:297] + "..." if len(summary) > 300 else summary),
            "minOSVersion": "15.0",
        })
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
