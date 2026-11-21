"""Read-only license inventory of Cargo.lock dependency trees.

Usage: python audit_licenses.py
Parses two Cargo.lock files, resolves each (name, version) against the locally
extracted crate sources in Cargo's registry src dir, reads the `license` /
`license-file` field of each crate's Cargo.toml, and reports:
  - total package count
  - packages grouped by exact license string
  - flags for copyleft / weak-copyleft / unknown / missing / not-extracted
  - explicit GPL-family evidence
  - license-file packages and whether the referenced file exists / what it says

This script only READS files. It writes nothing.

Resolution order for the `license` field:
  1. Locally extracted <registry>/src/<name>-<version>/Cargo.toml (authoritative).
  2. Else, if a matching .crate archive is cached, read Cargo.toml from the
     archive IN MEMORY (never extracted to disk) -- read-only, no writes.
  3. Else, FIRST-PARTY path/workspace packages are resolved from the workspace
     Cargo.toml files (supporting `license.workspace = true` inheritance).
  4. Otherwise: NOT VERIFIABLE locally (sparse index carries no license field).
"""
import io
import os
import re
import sys
import tarfile
import tomllib
from collections import defaultdict

sys.stdout.reconfigure(encoding="utf-8")

REGISTRY_BASE = r"C:\Users\zjc_9\.cargo\registry"
REGISTRY_SRC = os.path.join(REGISTRY_BASE, "src", "index.crates.io-1949cf8c6b5b557f")
CACHE_DIRS = [os.path.join(REGISTRY_BASE, "cache", "index.crates.io-1949cf8c6b5b557f")]
LOCKS = [
    (r"D:\Projects\dsh\mylock\core\Cargo.lock", "core"),
    (r"D:\Projects\dsh\mylock\desktop\src-tauri\Cargo.lock", "desktop/src-tauri"),
]
# First-party (no `source` in lock) packages -> (manifest path, workspace manifest path)
FIRST_PARTY = {
    "mylock-core": (r"D:\Projects\dsh\mylock\core\crates\mylock-core\Cargo.toml",
                    r"D:\Projects\dsh\mylock\core\Cargo.toml"),
    "mylock-cli": (r"D:\Projects\dsh\mylock\core\crates\mylock-cli\Cargo.toml",
                   r"D:\Projects\dsh\mylock\core\Cargo.toml"),
    "mylock-mount": (r"D:\Projects\dsh\mylock\core\crates\mylock-mount\Cargo.toml",
                     r"D:\Projects\dsh\mylock\core\Cargo.toml"),
    "mylock-desktop": (r"D:\Projects\dsh\mylock\desktop\src-tauri\Cargo.toml", None),
}

# Copyleft / weak-copyleft / non-permissive SPDX tokens (case-insensitive).
COPYLEFT_RE = re.compile(
    r"(?i)\b(?:"
    r"A?GPL(?:-[0-9.]+)?(?:-only|-or-later)?|LGPL(?:-[0-9.]+)?(?:-only|-or-later)?"
    r"|AGPL(?:-[0-9.]+)?(?:-only|-or-later)?|MPL(?:-[0-9.]+)?|EPL(?:-[0-9.]+)?"
    r"|CDDL(?:-[0-9.]+)?|EUPL(?:-[0-9.]+)?|OSL(?:-[0-9.]+)?|CPL(?:-[0-9.]+)?"
    r"|IPL(?:-[0-9.]+)?|SSPL(?:-[0-9.]+)?|BSL(?:-[0-9.]+)?|CC-BY-SA(?:-[0-9.]+)?"
    r"|CC-BY-NC(?:-[0-9.]+)?|CC-BY-ND(?:-[0-9.]+)?|Sleepycat|Artistic(?:-[0-9.]+)?"
    r"|NPL(?:-[0-9.]+)?|SISSL(?:-[0-9.]+)?|RPL(?:-[0-9.]+)?|QPL(?:-[0-9.]+)?"
    r")\b"
)
GPL_RE = re.compile(r"(?i)\b(?:A?GPL|LGPL|AGPL)(?:-[0-9.]+)?(?:-only|-or-later)?\b")
# For LICENSE-file text scan: literal GPL phrases (not the bare word "json")
GPL_TEXT_RE = re.compile(r"(?i)GNU\s+(?:GENERAL\s+PUBLIC|LESSER\s+GENERAL\s+PUBLIC|"
                         r"AFFERO\s+GENERAL\s+PUBLIC)\s+LICENSE|\b(?:LGPL|AGPL|GPLv?[0-9])\b")


def parse_toml_bytes(raw):
    return tomllib.loads(raw.decode("utf-8", errors="replace"))


def find_cargo_toml(name, version):
    """Return path to extracted crate's Cargo.toml, or None."""
    if not os.path.isdir(REGISTRY_SRC):
        return None
    target = f"{name}-{version}"
    p = os.path.join(REGISTRY_SRC, target, "Cargo.toml")
    if os.path.isfile(p):
        return p
    for d in os.listdir(REGISTRY_SRC):
        if d.lower() == target.lower():
            q = os.path.join(REGISTRY_SRC, d, "Cargo.toml")
            if os.path.isfile(q):
                return q
    return None


def find_crate_archive(name, version):
    """Return path of cached .crate archive (tar.gz), or None. Read-only lookup."""
    target = f"{name}-{version}.crate"
    for c in CACHE_DIRS:
        if not os.path.isdir(c):
            continue
        for f in os.listdir(c):
            if f.lower() == target.lower():
                return os.path.join(c, f)
    return None


def license_from_archive(path, name, version):
    """Read <name>-<version>/Cargo.toml from .crate tar.gz IN MEMORY."""
    try:
        with tarfile.open(path, "r:gz") as tf:
            member = f"{name}-{version}/Cargo.toml"
            try:
                f = tf.extractfile(member)
            except KeyError:
                # case/prefix fallback
                cand = [m for m in tf.getnames()
                        if m.lower().endswith("cargo.toml") and "/" in m]
                if not cand:
                    return None, None, "archive has no Cargo.toml"
                f = tf.extractfile(cand[0])
            raw = f.read()
        data = parse_toml_bytes(raw)
        pkg = data.get("package", {})
        return pkg.get("license"), pkg.get("license-file"), None
    except Exception as e:
        return None, None, f"archive read error: {e}"


def license_from_workspace(name):
    """Resolve first-party manifest, following license.workspace = true."""
    if name not in FIRST_PARTY:
        return None, None, None, "not a known first-party crate"
    manifest, ws = FIRST_PARTY[name]
    if not os.path.isfile(manifest):
        return None, None, None, "first-party manifest not found"
    with open(manifest, "rb") as f:
        data = tomllib.load(f)
    pkg = data.get("package", {})
    lic = pkg.get("license")
    licf = pkg.get("license-file")
    src = manifest
    if isinstance(lic, dict) and lic.get("workspace") is True:
        if ws and os.path.isfile(ws):
            with open(ws, "rb") as f:
                wdata = tomllib.load(f)
            lic = wdata.get("workspace", {}).get("package", {}).get("license")
            src = f"{manifest} (license.workspace=true -> {ws})"
        else:
            return None, None, None, "license.workspace=true but workspace manifest missing"
    return lic, licf, src, None


def resolve(name, version, has_source):
    """Return (lic, licf, source_desc, crate_dir_or_None, note)."""
    if not has_source:
        lic, licf, src, err = license_from_workspace(name)
        if err:
            return None, None, None, None, err
        return lic, licf, src, os.path.dirname(FIRST_PARTY[name][0]), None
    ct = find_cargo_toml(name, version)
    if ct:
        with open(ct, "rb") as f:
            data = tomllib.load(f)
        pkg = data.get("package", {})
        return pkg.get("license"), pkg.get("license-file"), ct, os.path.dirname(ct), None
    arc = find_crate_archive(name, version)
    if arc:
        lic, licf, err = license_from_archive(arc, name, version)
        if err:
            return None, None, arc, None, err
        return lic, licf, f"{arc} (Cargo.toml read in-memory from .crate)", None, None
    return None, None, None, None, "no extracted Cargo.toml and no cached .crate"


def inspect_license_file(crate_dir, licf):
    if crate_dir is None:
        return None, [], "<cannot locate LICENSE text (no extracted dir)>"
    p = os.path.join(crate_dir, licf)
    if not os.path.isfile(p):
        for cand in ("LICENSE", "LICENSE.txt", "LICENSE.md", "LICENCE", "COPYING"):
            q = os.path.join(crate_dir, cand)
            if os.path.isfile(q):
                p = q
                break
    if not os.path.isfile(p):
        return False, [], None
    try:
        with open(p, "r", encoding="utf-8", errors="replace") as f:
            text = f.read()
    except Exception as e:
        return True, [], f"<unreadable: {e}>"
    hits = sorted(set(GPL_TEXT_RE.findall(text)))
    preview = " / ".join(text.strip().splitlines()[:2])[:200]
    return True, hits, preview


def parse_lock(path):
    with open(path, "rb") as f:
        data = tomllib.load(f)
    return data.get("package", [])


def audit(lock_path, label):
    print("=" * 78)
    print(f"WORKSPACE: {label}")
    print(f"LOCK FILE: {lock_path}")
    print("=" * 78)
    pkgs = parse_lock(lock_path)
    print(f"TOTAL PACKAGES IN LOCK: {len(pkgs)}")

    src_kinds = defaultdict(int)
    for p in pkgs:
        s = p.get("source")
        if not s:
            src_kinds["(workspace/path, no source)"] += 1
        elif s.startswith("registry+"):
            src_kinds["registry+crates.io"] += 1
        else:
            src_kinds[s] += 1
    print("SOURCE DISTRIBUTION: " + "; ".join(f"{k}={v}" for k, v in sorted(src_kinds.items())))

    groups = defaultdict(list)
    flags = defaultdict(list)
    gpl_hits = []
    archive_resolved = []
    first_party_resolved = []
    license_file_pkgs = []
    not_verified = []
    for p in pkgs:
        name = p.get("name")
        version = p.get("version")
        tag = f"{name} {version}"
        has_source = bool(p.get("source"))
        lic, licf, src_desc, crate_dir, note = resolve(name, version, has_source)

        if not has_source and src_desc and not note:
            first_party_resolved.append(f"{tag} -> {src_desc}")
        if src_desc and ".crate" in str(src_desc) and "in-memory" in str(src_desc):
            archive_resolved.append(f"{tag} -> {src_desc}")

        if note:
            not_verified.append(tag)
            flags["NOT VERIFIABLE LOCALLY (no extracted Cargo.toml, no cached .crate)"].append(tag)
            continue

        if licf:
            exists, hits, preview = inspect_license_file(crate_dir, licf)
            license_file_pkgs.append((tag, licf, exists, hits, preview, src_desc))
            groups[f"[license-file] {licf}"].append(tag)
            if not exists:
                flags["LICENSE-FILE missing on disk (cannot verify terms)"].append(
                    f"{tag} -> license-file={licf} in {src_desc}")
            if hits:
                flags["LICENSE-FILE text contains GPL/copyleft phrases"].append(
                    f"{tag} -> {licf} phrases={hits} in {src_desc}")
            if not hits and exists:
                flags["LICENSE-FILE present, full terms not classified (first lines only)"].append(
                    f"{tag} -> {licf} preview='{preview}' in {src_desc}")
        if lic:
            groups[lic].append(tag)
            if COPYLEFT_RE.search(lic):
                flags["COPYLEFT / WEAK-COPYLEFT / NON-PERMISSIVE LICENSE EXPRESSION"].append(
                    f'{tag} -> license = "{lic}"  [{src_desc}]')
            if GPL_RE.search(lic):
                gpl_hits.append(f'{tag} -> license = "{lic}"  [{src_desc}]')
            if re.search(r"(?i)\b(UNLICENSED|PROPRIETARY|UNKNOWN)\b", lic):
                flags["UNKNOWN / PROPRIETARY LICENSE EXPRESSION"].append(
                    f'{tag} -> license = "{lic}"  [{src_desc}]')
        elif not licf:
            flags["MISSING LICENSE (no `license` and no `license-file` field)"].append(
                f"{tag} -> {src_desc}")
            groups["[no license field]"].append(tag)
        elif not lic and licf:
            flags["NO `license` EXPRESSION (only license-file)"].append(
                f'{tag} -> license-file = "{licf}"  [{src_desc}]')

    print("\n--- LICENSE GROUPS (exact license string from Cargo.toml) ---")
    for lic in sorted(groups, key=lambda x: (-len(groups[x]), x.lower())):
        vals = groups[lic]
        print(f"[{len(vals):4d}]  {lic}")
        print("      " + ", ".join(sorted(vals, key=str.lower)))

    print("\n--- FLAGS ---")
    if not flags:
        print("(none)")
    for k in sorted(flags):
        print(f"* {k}  ({len(flags[k])})")
        for d in flags[k]:
            print(f"    - {d}")

    print("\n--- GPL-FAMILY EVIDENCE (license expressions in resolved tree) ---")
    if gpl_hits:
        for h in gpl_hits:
            print("  !!! " + h)
    else:
        print("  NONE: no resolved package's Cargo.toml `license` expression contains GPL/LGPL/AGPL tokens.")

    print(f"\n--- RESOLVED VIA CACHED .crate (read in memory, no extraction): {len(archive_resolved)} ---")
    for a in archive_resolved:
        print("  - " + a)
    print(f"\n--- FIRST-PARTY PATH PACKAGES RESOLVED FROM WORKSPACE MANIFESTS: {len(first_party_resolved)} ---")
    for a in first_party_resolved:
        print("  - " + a)

    print(f"\n--- NOT VERIFIABLE LOCALLY: {len(not_verified)} package(s) ---")
    for t in not_verified:
        print("  - " + t)

    print("\n--- LICENSE-FILE PACKAGES ---")
    if not license_file_pkgs:
        print("  (none)")
    for tag, licf, exists, hits, preview, src_desc in license_file_pkgs:
        print(f"  - {tag}: license-file=\"{licf}\" exists={exists} gpl_phrases={hits}")
        print(f"      source: {src_desc}")
        print(f"      first lines: {preview}")

    return {
        "total": len(pkgs),
        "groups": dict(groups),
        "flags": dict(flags),
        "gpl_hits": gpl_hits,
        "not_verified": not_verified,
        "archive_resolved": archive_resolved,
        "first_party_resolved": first_party_resolved,
        "license_file_pkgs": license_file_pkgs,
    }


def global_gpl_scan():
    """Independent raw scan: GPL-family phrases in every extracted file text."""
    print("\n" + "=" * 78)
    print("INDEPENDENT RAW SCAN: GPL-family phrases in extracted crate sources")
    print(f"ROOT: {REGISTRY_SRC}")
    print("=" * 78)
    if not os.path.isdir(REGISTRY_SRC):
        print("registry src dir not found")
        return
    scanned = 0
    toml_hits = []
    license_hits = []
    exts = (".toml", ".md", ".txt", "", ".html", ".rst")
    for d in sorted(os.listdir(REGISTRY_SRC)):
        crate = os.path.join(REGISTRY_SRC, d)
        if not os.path.isdir(crate):
            continue
        for root, dirs, files in os.walk(crate):
            for fn in files:
                p = os.path.join(root, fn)
                ext = os.path.splitext(fn)[1].lower()
                if ext not in exts and not (fn.upper().startswith("LICENSE") or fn.upper().startswith("COPYING")):
                    continue
                if os.path.getsize(p) > 2_000_000:
                    continue
                try:
                    with open(p, "r", encoding="utf-8", errors="replace") as f:
                        text = f.read()
                except Exception:
                    continue
                scanned += 1
                found = GPL_TEXT_RE.findall(text)
                if found:
                    rel = os.path.relpath(p, REGISTRY_SRC)
                    rec = (rel, sorted(set(found))[:5])
                    if fn.lower() == "cargo.toml":
                        toml_hits.append(rec)
                    else:
                        license_hits.append(rec)
    print(f"text files scanned: {scanned}")
    print(f"Cargo.toml files with GPL phrases: {len(toml_hits)}")
    for rel, found in toml_hits:
        print(f"  {rel}: {found}")
    print(f"LICENSE/README-style files with GPL phrases: {len(license_hits)}")
    for rel, found in license_hits:
        print(f"  {rel}: {found}")


def main():
    results = {}
    for lock, label in LOCKS:
        if not os.path.isfile(lock):
            print(f"MISSING LOCK FILE: {lock}")
            continue
        results[label] = audit(lock, label)
    global_gpl_scan()
    print("\n" + "=" * 78)
    print("SUMMARY OF TOTALS")
    for label, r in results.items():
        print(f"  {label}: {r['total']} packages, GPL-expression hits={len(r['gpl_hits'])}, "
              f"not-verifiable={len(r['not_verified'])}, "
              f"archive-resolved={len(r['archive_resolved'])}, "
              f"first-party-resolved={len(r['first_party_resolved'])}, "
              f"license-file={len(r['license_file_pkgs'])}, groups={len(r['groups'])}")


if __name__ == "__main__":
    main()
