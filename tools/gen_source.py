#!/usr/bin/env python3
"""Regenerate the SideStore/AltStore source: one app, stable and nightly tracks.

Legacy `versions` contains only the newest stable (or newest test if no stable
exists). Tags label release notes/URLs, never IPA versions. Info.plist inside the
published IPA is authoritative; older verification reports contain Cargo versions.
Run from the repository root: python3 tools/gen_source.py
"""
import hashlib
import io
import json
import plistlib
import re
import sys
import urllib.request
import zipfile

REPO = "MarkusSela/IW4-Pocket"
RAW = f"https://raw.githubusercontent.com/{REPO}/main"
BUNDLE = "com.markussela.iw4l"


def get_bytes(url):
    req = urllib.request.Request(url, headers={"User-Agent": "gen-source"})
    with urllib.request.urlopen(req, timeout=120) as response:
        return response.read()


def get(url):
    return json.loads(get_bytes(url))


def order_key(version):
    """Tag ordering is only used to discard releases older than newest stable."""
    match = re.fullmatch(r"v?(\d+)\.(\d+)\.(\d+)([A-Za-z]*)(?:-[0-9A-Za-z.-]+)?", version)
    if not match:
        raise ValueError(f"Unsupported release tag: {version}")
    return (*map(int, match.group(1, 2, 3)), match[4])


def pick(candidates):
    """Keep newest stable and newer tests; metadata decides nightly ordering."""
    stable = sorted((c for c in candidates if not c["test"]),
                    key=lambda c: order_key(c["tag"]), reverse=True)
    tests = [c for c in candidates if c["test"]]
    if not stable:
        return tests
    newest = stable[0]
    return [newest] + [c for c in tests if order_key(c["tag"]) > order_key(newest["tag"])]


def ipa_metadata(data):
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        names = [n for n in archive.namelist()
                 if re.fullmatch(r"Payload/[^/]+\.app/Info\.plist", n)]
        if len(names) != 1:
            raise ValueError("Expected exactly one application Info.plist")
        info = plistlib.loads(archive.read(names[0]))
    if info.get("CFBundleIdentifier") != BUNDLE:
        raise ValueError("IPA bundle identifier changed")
    for key in ("CFBundleShortVersionString", "CFBundleVersion", "MinimumOSVersion"):
        if not isinstance(info.get(key), str) or not re.fullmatch(r"\d+(?:\.\d+){0,2}", info[key]):
            raise ValueError(f"Invalid IPA {key}")
    return info


def version_key(version):
    return tuple(map(int, version.split("."))) + (0,) * (3 - len(version.split(".")))


def release_key(version):
    return (version_key(version["version"]), version_key(version["buildVersion"]), version["date"])


def release_version(rel, ipa, load_bytes):
    data = load_bytes(ipa["browser_download_url"])
    info = ipa_metadata(data)
    digest = hashlib.sha256(data).hexdigest()
    if len(data) != ipa["size"]:
        raise ValueError("IPA asset size mismatch")
    if ipa.get("digest") and ipa["digest"] != "sha256:" + digest:
        raise ValueError("IPA asset digest mismatch")
    report_asset = next((a for a in rel["assets"] if a["name"] == "ipa-verification.json"), None)
    if report_asset:
        report = json.loads(load_bytes(report_asset["browser_download_url"]))
        # `version` in legacy reports is the Cargo version, NOT the IPA version.
        if (report.get("bundle_identifier") != info["CFBundleIdentifier"]
                or report.get("build") != info["CFBundleVersion"]
                or report.get("sha256") != digest):
            raise ValueError("IPA verification report mismatch")
    note = (rel.get("body") or "").strip().splitlines()
    summary = next((line.strip() for line in note[1:] if line.strip() and not line.startswith("#")), "")
    summary = summary.replace("**", "").replace("`", "")
    label = rel["tag_name"].removeprefix("v")
    prefix = f"TEST {label} — Experimental prerelease. Not verified on a phone. No confirmed fix for the memory crash. " if rel["prerelease"] else ""
    return {
        "version": info["CFBundleShortVersionString"],
        "buildVersion": info["CFBundleVersion"],
        "date": rel["published_at"],
        "size": ipa["size"],
        "downloadURL": ipa["browser_download_url"],
        "localizedDescription": prefix + (summary[:297] + "..." if len(summary) > 300 else summary),
        "minOSVersion": info["MinimumOSVersion"],
        "sha256": digest,
    }


def build_source(releases, load_bytes=get_bytes):
    by_tag = {}
    for rel in releases:
        ipa = next((a for a in rel["assets"] if a["name"] == "IW4.Pocket.ipa"), None)
        if ipa is not None and not rel["draft"]:
            by_tag[rel["tag_name"]] = (rel, ipa)
    chosen = pick([{"tag": tag, "test": rel["prerelease"]}
                   for tag, (rel, _) in by_tag.items()])
    stable, nightly = [], []
    for candidate in chosen:
        rel, ipa = by_tag[candidate["tag"]]
        try:
            version = release_version(rel, ipa, load_bytes)
        except (ValueError, KeyError, OSError, zipfile.BadZipFile) as exc:
            if not rel["prerelease"]:
                raise ValueError(f"Cannot verify stable {rel['tag_name']}: {exc}") from exc
            print(f"Skipping unverified TEST {rel['tag_name']}: {exc}", file=sys.stderr)
            continue
        (nightly if rel["prerelease"] else stable).append(version)
    nightly.sort(key=release_key, reverse=True)
    if stable:
        nightly = [v for v in nightly if release_key(v)[:2] > release_key(stable[0])[:2]]
    if not stable and not nightly:
        raise ValueError("No verified IPA releases; source was not regenerated")
    channels = [{"track": "stable", "releases": stable}] if stable else []
    if nightly:
        channels.append({"track": "nightly", "releases": nightly})
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
            "bundleIdentifier": BUNDLE,
            "developerName": "MarkusSela",
            "subtitle": "Modern Warfare 2 on iPhone",
            "localizedDescription": "A port of MW2 (2009) to iPhone. You need your own copy of the PC game files.",
            "iconURL": f"{RAW}/ios/icons/AppIcon-1024.png",
            "tintColor": "#2f3a2f",
            "category": "games",
            "versions": stable or nightly[:1],
            "releaseChannels": channels,
        }],
        "news": [],
    }
    return source


if __name__ == "__main__":
    releases = get(f"https://api.github.com/repos/{REPO}/releases?per_page=100")
    source = build_source(releases)
    with open("source.json", "w", encoding="utf-8") as out:
        json.dump(source, out, indent=2, ensure_ascii=False)
        out.write("\n")
    for channel in source["apps"][0]["releaseChannels"]:
        print(channel["track"], [(v["version"], v["buildVersion"]) for v in channel["releases"]])
