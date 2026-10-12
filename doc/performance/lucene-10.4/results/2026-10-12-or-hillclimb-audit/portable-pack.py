#!/usr/bin/env python3
"""Root-only retained-evidence packer. Does not build or run an engine.

The destination must not exist. Archive bytes and historical receipts are never
rewritten. gzip encoding is declared in the manifest, with unchanged logical
filenames so conventional stage references remain valid.
"""
import argparse
import csv
import difflib
import gzip
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys

OUT = Path(__file__).resolve().parent
SEARCH = OUT.parents[1]
FROZEN = OUT.parent / "lucene-rs-balanced-oct06"
REFINEMENT = OUT.parent / "lucene-rs-or-refinement-oct07"
REPO = SEARCH / "tantivy-pr2937"
SNAPSHOT_KEY = "tantivy-or-hillclimb-oct07"
BASE_COMMIT = "0e9329dfe782187a8feda0024102e4aabf517c28"
BASE_BINARY = "2efc80ae6cc57c4163cab798f50bfb35ceb085c039bc4dd4b8fe31d22d387bbf"
PORT_COMMIT = "e5d1f81bff42c87886b12770f3c79648bb1963ed"
PORT_BINARY = "b0923daad72d662a056496ab115a57519ec474216fe05b1522e6529ae2154a2a"
TEST_RE = re.compile(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out")
HEADER_RE = re.compile(r"^diff --git ([ai])/(\S+) ([bw])/(\S+)$", re.MULTILINE)
HUNK_RE = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")
PLAIN = {".py", ".rs", ".md", ".toml", ".lock", ".html", ".patch"}


def require(value, message):
    if not value:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_sha(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def obj(path):
    return json.loads(path.read_bytes())


def json_bytes(value):
    return (json.dumps(value, indent=2, allow_nan=False) + "\n").encode()


def safe_name(name):
    p = PurePosixPath(name)
    require(not p.is_absolute() and str(p) == name and all(x not in ("", ".", "..") for x in name.split("/")), "unsafe packet path " + name)
    require("\\" not in name and ":" not in name, "noncanonical packet path " + name)
    return name


def apply_file_patch(base, patch, path):
    """Apply exact unified context; this is not a fuzzy patch operation."""
    headers = list(HEADER_RE.finditer(patch))
    section = None
    for i, match in enumerate(headers):
        a, before_path, b, after_path = match.groups()
        require((a, b) in (("a", "b"), ("i", "w")) and before_path == after_path, "unsupported original patch header")
        if before_path == path:
            require(section is None, "duplicate original patch section")
            section = patch[match.end():headers[i + 1].start() if i + 1 < len(headers) else len(patch)]
    if section is None:
        return None
    old = base.decode().splitlines(keepends=True)
    lines = section.splitlines(keepends=True)
    output, cursor, number = [], 0, 0
    while number < len(lines):
        match = HUNK_RE.match(lines[number])
        if match is None:
            number += 1
            continue
        start, old_count, new_start, new_count = match.groups()
        start = int(start) - 1 if int(start) else 0
        old_count = int(old_count) if old_count is not None else 1
        new_count = int(new_count) if new_count is not None else 1
        require(cursor <= start <= len(old), "patch hunk position")
        output.extend(old[cursor:start])
        require(len(output) == (int(new_start) - 1 if int(new_start) else 0), "patch new hunk position")
        cursor, removed, added = start, 0, 0
        number += 1
        while number < len(lines) and not HUNK_RE.match(lines[number]):
            line = lines[number]
            if not line or line[0] not in " +-":
                break
            marker, text = line[0], line[1:]
            if number + 1 < len(lines) and lines[number + 1].startswith("\\ No newline at end of file"):
                text = text.removesuffix("\n")
                number += 1
            if marker in " -":
                require(cursor < len(old) and old[cursor] == text, "original patch context differs")
                cursor += 1
                removed += 1
            if marker in " +":
                output.append(text)
                added += 1
            number += 1
        require((removed, added) == (old_count, new_count), "patch hunk counts")
    output.extend(old[cursor:])
    return "".join(output).encode()


class Packer:
    def __init__(self, destination):
        self.destination = destination
        destination.mkdir(parents=True, exist_ok=False)
        self.files = {}
        self.origins = {}
        self.derivations = []
        self.exclusions = []
        self.source_cache = {}
        self.git_cache = {}
        self.archives = {}

    def bytes(self, data, name, role, origin=None):
        name = safe_name(name)
        require(name not in self.files, "duplicate packet file " + name)
        target = self.destination / name
        target.parent.mkdir(parents=True, exist_ok=True)
        compressed = len(data) > 65536 and Path(name).suffix not in PLAIN and name != "attempts.json"
        stored = gzip.compress(data, compresslevel=6, mtime=0) if compressed else data
        target.write_bytes(stored)
        self.files[name] = {"encoding": "gzip" if compressed else "identity", "stored_bytes": len(stored), "stored_sha256": sha(stored), "raw_bytes": len(data), "raw_sha256": sha(data), "role": role}
        if origin is not None:
            self.origins[name] = {"original_path": str(origin), "original_bytes": len(data), "original_sha256": sha(data)}
        return name

    def copy(self, source, name, role="retained-evidence"):
        source = Path(source)
        require(source.is_file() and not source.is_symlink(), "missing/symlink input " + str(source))
        # Repeated proof references share one packet entry, never overwrite it.
        if name in self.files:
            require(self.files[name]["raw_sha256"] == file_sha(source), "duplicate name differs " + name)
            return name
        size = source.stat().st_size
        if size <= 8 * 1024 * 1024 or source.suffix in PLAIN:
            return self.bytes(source.read_bytes(), name, role, source)
        name = safe_name(name)
        target = self.destination / name
        target.parent.mkdir(parents=True, exist_ok=True)
        raw_hash, raw_size = hashlib.sha256(), 0
        # Streaming keeps full raw samples and perf traces without giant buffers.
        with source.open("rb") as input_stream, target.open("xb") as output:
            with gzip.GzipFile(filename="", mode="wb", fileobj=output, compresslevel=6, mtime=0) as encoded:
                for block in iter(lambda: input_stream.read(1024 * 1024), b""):
                    raw_hash.update(block)
                    raw_size += len(block)
                    encoded.write(block)
        require(raw_size == size and file_sha(source) == raw_hash.hexdigest(), "input changed while packing " + str(source))
        self.files[name] = {"encoding": "gzip", "stored_bytes": target.stat().st_size, "stored_sha256": file_sha(target), "raw_bytes": raw_size, "raw_sha256": raw_hash.hexdigest(), "role": role}
        self.origins[name] = {"original_path": str(source), "original_bytes": raw_size, "original_sha256": raw_hash.hexdigest()}
        return name

    def tree(self, source, prefix):
        for path in sorted(source.rglob("*")):
            require(not path.is_symlink(), "symlink input " + str(path))
            if path.is_dir():
                continue
            relative = path.relative_to(source)
            if any(part in ("target", "test-target", "__pycache__", ".git") for part in relative.parts) or path.name in ("tantivy_worker", "configured_worker", "port_worker") or path.suffix == ".pyc":
                self.exclusions.append({"original_path": str(path), "reason": "engine/build/cache payload excluded"})
                continue
            with path.open("rb") as stream:
                magic = stream.read(4)
            require(magic != b"\x7fELF" and magic[:2] != b"MZ", "unexpected executable payload " + str(path))
            self.copy(path, prefix + "/" + relative.as_posix())

    def git(self, repo, commit, path):
        key = (str(repo), commit, path)
        if key not in self.git_cache:
            result = subprocess.run(["git", "show", commit + ":" + path], cwd=repo, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
            self.git_cache[key] = result.stdout if result.returncode == 0 else None
        return self.git_cache[key]

    def source(self, path, expected, commit, archive, repo=REPO):
        key = (path, expected)
        if key in self.source_cache:
            return self.source_cache[key]
        candidates = [archive / path, repo / path]
        candidates += [p / path for p in sorted(SEARCH.glob("tantivy*")) if p.is_dir()]
        candidates += list(archive.rglob("*.rs")) + list(OUT.glob("*source*.rs"))
        for source in candidates:
            if source.is_file() and not source.is_symlink() and file_sha(source) == expected:
                artifact = self.copy(source, "sources/" + expected + "/" + Path(path).name, "source-blob")
                self.source_cache[key] = artifact
                return artifact
        for reference in (commit, BASE_COMMIT):
            data = self.git(repo, reference, path)
            if data is not None and sha(data) == expected:
                artifact = self.bytes(data, "sources/" + expected + "/" + Path(path).name, "source-blob")
                self.derivations.append({"artifact": artifact, "kind": "Git-object", "repository": str(repo), "commit": reference, "path": path, "raw_sha256": expected})
                self.source_cache[key] = artifact
                return artifact
        # Root approved this explicit recovery for the retained08 failed
        # revision. Both authentic inputs stay in the packet, and the guarded
        # output SHA must match exactly; no guessed or fuzzy source is accepted.
        base = self.git(repo, commit, path)
        if base is not None:
            patches = sorted(archive.rglob("*.patch")) + sorted(OUT.glob("*.patch"))
            for patch_path in patches:
                try:
                    data = apply_file_patch(base, patch_path.read_text(), path)
                except (ValueError, UnicodeError):
                    continue
                if data is None or sha(data) != expected:
                    continue
                patch_sha = file_sha(patch_path)
                patch_artifact = self.copy(patch_path, "common/reconstruction-inputs/" + patch_sha + "/" + patch_path.name, "authentic-source-patch")
                base_artifact = "sources/" + sha(base) + "/" + Path(path).name
                if base_artifact not in self.files:
                    self.bytes(base, base_artifact, "Git-base-source")
                artifact = self.bytes(data, "sources/" + expected + "/" + Path(path).name, "reconstructed-source-blob")
                self.derivations.append({"artifact": artifact, "kind": "exact-original-patch-reconstruction", "original_raw_source_unavailable": True, "base_commit": commit, "base_path": path, "base_artifact": base_artifact, "original_patch": patch_artifact, "guarded_output_sha256": expected, "reason": "Literal preserved original patch applied to preserved Git base, with exact context/counts and final guarded source digest. This source is reconstructed, not an original copied file."})
                self.source_cache[key] = artifact
                return artifact
        raise ValueError("Missing exact source blob " + path + " SHA=" + expected + "; retain it before packing. No source is guessed.")

    def patch_paths(self, path):
        headers = HEADER_RE.findall(path.read_text())
        require(headers and all((a, b) in (("a", "b"), ("i", "w")) and x == y for a, x, b, y in headers), "unsupported patch headers " + str(path))
        paths = [x for _, x, _, _ in headers]
        require(len(paths) == len(set(paths)), "duplicate patch file")
        for path in paths:
            safe_name(path)
        return paths

    def candidate(self, archive, prefix, snapshot_local, patch_path, commit=None, binaries=None):
        snapshot = obj(archive / snapshot_local)
        key = next(k for k in snapshot if k.startswith("tantivy-") and type(snapshot[k]) is dict and "files" in snapshot[k])
        state = snapshot[key]
        changed = self.patch_paths(patch_path)
        reference = commit or state["head"]
        bindings = {p: self.source(p, state["files"][p], reference, archive) for p in changed}
        patch_relative = patch_path.relative_to(archive).as_posix()
        patch = self.copy(patch_path, prefix + "/" + patch_relative, "source-patch")
        return {"snapshot": snapshot_local, "snapshot_key": key, "source_bindings": bindings, "patch": patch, "changed_paths": changed, "binary_sha256": None if binaries is None else binaries["tantivy"], "configured_binary_sha256": None if binaries is None else binaries["configured"]}

    def reconstructed_patch(self, archive, prefix, label, snapshot_local, paths):
        snapshot = obj(archive / snapshot_local)[SNAPSHOT_KEY]
        pieces = []
        for path in paths:
            old = self.git(REPO, snapshot["head"], path)
            require(old is not None, "missing Git base for derived revision patch")
            artifact = self.source(path, snapshot["files"][path], snapshot["head"], archive)
            entry = self.files[artifact]
            stored = (self.destination / artifact).read_bytes()
            current = gzip.decompress(stored) if entry["encoding"] == "gzip" else stored
            pieces.append("diff --git a/" + path + " b/" + path + "\n")
            pieces.extend(difflib.unified_diff(old.decode().splitlines(keepends=True), current.decode().splitlines(keepends=True), fromfile="a/" + path, tofile="b/" + path))
        local = "derived/" + label + ".patch"
        artifact = self.bytes("".join(pieces).encode(), prefix + "/" + local, "derived-source-diff")
        self.derivations.append({"artifact": artifact, "kind": "derived-source-diff", "base_commit": snapshot["head"], "source_snapshot": prefix + "/" + snapshot_local, "reason": "Original run-specific patch was not retained; derive readable diff from retained exact source and Git base. Original other patches remain unchanged in inventory."})
        return artifact, paths

    def failed_candidate(self, archive, prefix, label, snapshot_local, final_paths):
        patch_candidates = sorted((archive / label).glob("*.patch"))
        if patch_candidates:
            # The prepared patch is the run revision; *-original.patch records
            # an earlier design and remains inventoried without this binding.
            prepared = next((p for p in patch_candidates if not p.name.endswith("-original.patch")), patch_candidates[0])
            return self.candidate(archive, prefix, snapshot_local, prepared)
        artifact, paths = self.reconstructed_patch(archive, prefix, label, snapshot_local, final_paths)
        snapshot = obj(archive / snapshot_local)[SNAPSHOT_KEY]
        return {"snapshot": snapshot_local, "snapshot_key": SNAPSHOT_KEY, "source_bindings": {p: self.source(p, snapshot["files"][p], snapshot["head"], archive) for p in paths}, "patch": artifact, "changed_paths": paths, "binary_sha256": None, "configured_binary_sha256": None}

    def stage(self, archive, folder, revision):
        receipt = obj(archive / folder / "receipt.json")
        stdout = (archive / folder / "stdout.log").read_text()
        stderr = (archive / folder / "stderr.log").read_text()
        matches = TEST_RE.findall(stdout)
        totals = None if not matches else dict(zip(("passed", "failed", "ignored", "measured", "filtered_out"), map(sum, zip(*(tuple(map(int, row)) for row in matches)))))
        name = Path(folder).name
        role = "targeted" if name.startswith("tests-targeted") or name == "duplicate-unfixed-tests" else "build" if name.startswith("build") else "formatting" if name.startswith("formatting") else "clippy" if name.startswith("clippy") else "layout" if name == "layout" else "tests" if name.startswith("tests") else None
        require(role is not None, "unclassified retained preparation stage " + folder)
        fragments = {}
        if receipt["returncode"] != 0:
            codes = re.findall(r"error\[(E[0-9]+)\]", stderr)
            fragments = {"stderr": sorted(set(codes))} if codes else {"stdout": ["test result: FAILED."]}
            if archive.name == "attempt-19" and folder == "pre-initial-seek-fix/tests-targeted":
                signature = "assertion failed: target >= self.doc()"
                require(signature in stderr, "19 original forward-seek failure signature missing")
                fragments["stderr"] = [signature]
        return {"role": role, "folder": folder, "expected_returncode": receipt["returncode"], "expected_receipt": {k: receipt[k] for k in ("compiler", "rustflags", "jobs", "profile", "command")}, "tests": totals, "revision": revision, "required_log_fragments": fragments}

    def parent(self, guard, baseline):
        selected = guard.get("selected_parent")
        if selected is None:
            require(guard["binaries"]["port"] == BASE_BINARY, "unknown baseline lane identity")
            return {"kind": "baseline", "binary_sha256": BASE_BINARY, "source_commit": BASE_COMMIT, "descriptor": None, "compiled_snapshot": baseline["compiled_snapshot"], "exact_receipt": baseline["exact_receipt"]}
        original = Path(selected["descriptor"])
        receipt = obj(original)
        require(file_sha(original) == selected["descriptor_sha256"], "selected-parent descriptor changed")
        qid = re.fullmatch(r"parent-([0-9]+)\.json", original.name).group(1)
        require(receipt["binary_sha256"] == guard["binaries"]["port"], "parent binary guard")
        descriptor = self.copy(original, "common/" + original.name, "accepted-parent")
        compiled = Path(receipt["compiled_sources"])
        exact = Path(receipt["exact_gate"])
        require(file_sha(compiled) == receipt["compiled_sources_sha256"] and file_sha(exact) == receipt["exact_gate_sha256"], "accepted parent proof changed")
        return {"kind": "accepted", "attempt_id": qid, "binary_sha256": receipt["binary_sha256"], "source_commit": receipt["source_commit"], "descriptor": descriptor, "compiled_snapshot": "attempts/" + qid + "/build-final/after.json", "exact_receipt": "attempts/" + qid + "/verify/receipt.json"}

    def lane(self, archive, prefix, qid, port=False):
        folder = "port-measure" if port else "measure"
        if not (archive / folder / "receipt.json").exists():
            require(not (archive / folder).exists(), "partial timing lane must be resolved before publication " + str(archive / folder))
            return {"status": "not_run", "reason": "This lane was not run for the retained rejected trial."}
        csv_name, summary_name = ("port-per-query.csv", "port-summary.json") if port else ("per-query.csv", "summary.json")
        metric_name = "attempt-" + qid + "-metrics.json"
        metric = None if port else obj(archive / metric_name)
        bindings = {}
        if not port:
            for original, digest in obj(archive / summary_name)["files_sha256"].items():
                local = Path(original)
                relocated = "common/queries.jsonl" if local.name == "queries.jsonl" else prefix + "/" + (folder + "/" + local.name if local.name != csv_name else csv_name)
                require(self.files[relocated]["raw_sha256"] == digest, "summary relocated hash mismatch " + original)
                bindings[original] = relocated
        group = "port-group-audit.json"
        has_group = port and (archive / group).exists()
        controller = self.controller(archive, "port_group_audit.py") if has_group else None
        return {"status": "retained", "folder": folder, "csv": csv_name, "summary": summary_name, "protocols": {str(i): {s: folder + "/pair" + str(i) + "-" + s + ".protocol.jsonl" for s in "AB"} for i in range(2 if port else 3)}, "summary_hash_bindings": bindings, "metrics": None if port else metric_name, "metrics_format": None if port else "legacy-arithmetic" if qid == "01" else "group-controls", "metrics_metadata": {} if port else {k: metric[k] for k in ("efficiency", "profile_overwrite_guard") if k in metric}, "metric_hash_bindings": [], "port_group_audit": group if has_group else None, "port_group_controller": controller}

    def controller(self, archive, name, expected=None):
        options = [archive / "controllers" / name, OUT / name]
        source = next((p for p in options if p.is_file() and (expected is None or file_sha(p) == expected)), None)
        require(source is not None, "matching controller missing " + name)
        return self.copy(source, "common/controllers/" + file_sha(source) + "/" + name, "controller-source")

    def profiles(self, archive, prefix, qid, parent_binary, candidate_binary):
        result = []
        if qid == "01":
            result.append({"kind": "profile", "status": "overwritten_unavailable", "role": "original-baseline", "reason": "Legacy output-name reuse overwrote the original baseline raw profile. Recreated profiles do not recover it.", "correction": "profile-evidence-correction.json"})
        for receipt_path in sorted(archive.rglob("*-profile-receipt.json")):
            receipt = obj(receipt_path)
            relative = receipt_path.relative_to(archive).as_posix()
            base = relative.removesuffix("-profile-receipt.json")
            binary = receipt["binary_sha256"]
            require(binary in (parent_binary, candidate_binary), "profile binary role not grounded")
            role = "candidate" if binary == candidate_binary else "baseline"
            legacy = "script_sha256" not in receipt
            status = "retained_excluded_own_clippy" if legacy else "retained_recreated" if qid == "01" else "retained_diagnostic"
            controller = self.controller(archive, "profile-two-legacy.py" if legacy else "profile-two.py", receipt.get("script_sha256")) if qid == "01" else self.copy(archive / (base + "-controller.py"), prefix + "/" + base + "-controller.py", "profile-controller")
            expected_cases = receipt.get("cases_sha256")
            case_paths = sorted(archive.rglob("*cases*.json")) + sorted(archive.rglob("profile-*.json")) + sorted(OUT.glob("*cases*.json"))
            source = next((p for p in case_paths if expected_cases is None or file_sha(p) == expected_cases), None)
            if source is not None:
                cases = self.copy(source, "common/profile-cases/" + file_sha(source) + "/" + source.name, "profile-cases")
            else:
                data = json_bytes(receipt["cases"])
                require(sha(data) == expected_cases, "original casefile unavailable and exact reconstruction differs")
                cases = self.bytes(data, "common/profile-cases/" + expected_cases + "/reconstructed-cases.json", "reconstructed-profile-cases") if "common/profile-cases/" + expected_cases + "/reconstructed-cases.json" not in self.files else "common/profile-cases/" + expected_cases + "/reconstructed-cases.json"
                self.derivations.append({"artifact": cases, "kind": "reconstructed_hash_matched", "original_unavailable": True, "source_receipt": prefix + "/" + relative, "selector": ["cases"], "serializer": "json.dumps(indent=2) + newline", "raw_sha256": expected_cases})
            limitations = ["Diagnostic cycles only; CPU contention and profile sampling remain. No causal cycle or latency acceptance proof."]
            if legacy:
                limitations.append("Surviving candidate profile overlapped own clippy and is excluded from clean paired interpretation; legacy controller has fewer input guards.")
            if source is None:
                limitations.append("Original casefile is unavailable. Cases bytes reconstructed from retained receipt match its historical guarded SHA; this is reconstructed evidence, not an original copied artifact.")
            result.append({"kind": "profile", "status": status, "role": role, "receipt": relative, "receipt_format": "legacy" if legacy else "guarded", "replies": base + "-profile-replies.json", "controller": controller, "cases": cases, "binary_sha256": binary, "trace": {"status": "retained", "artifact": prefix + "/" + base + "-perf.data"}, "report": base + "-perf-report.txt", "stderr": base + "-perf.stderr", "limitations": limitations})
        return result

    def attempt(self, qid, outcome, baseline, cases):
        archive = OUT / ("attempt-" + qid)
        require(archive.is_dir(), "decision exists without completed archive " + qid)
        self.archives[qid] = archive
        prefix = "attempts/" + qid
        self.tree(archive, prefix)
        default = obj(archive / "verify/before.json")
        configured = obj(archive / "configured-k0.9-b0.4/before.json")
        binaries = {"tantivy": default["binaries"]["tantivy"], "configured": configured["binaries"]["tantivy"]}
        decision_name = "attempt-" + qid + "-acceptance.json" if outcome == "accepted" else "rejection.json"
        decision = obj(archive / decision_name)
        commit = decision.get("commit") if outcome == "accepted" else None
        patch = archive / ("accepted.patch" if outcome == "accepted" else "rejected.patch")
        candidate = self.candidate(archive, prefix, "build-final/after.json", patch, commit, binaries)
        parent = self.parent(default, baseline)
        stages, revisions = [], {}
        for command in sorted(archive.rglob("command.json")):
            folder = command.parent.relative_to(archive).as_posix()
            if not (command.parent / "receipt.json").exists():
                raise ValueError("partial preparation stage " + str(command.parent))
            if not (command.parent / "compiler.txt").exists():
                continue  # A retained controller command is not a stage.py run.
            revision = "final" if "/" not in folder else folder.split("/")[0]
            if revision != "final" and revision not in revisions:
                revisions[revision] = self.failed_candidate(archive, prefix, revision, folder + "/before.json", candidate["changed_paths"])
            stages.append(self.stage(archive, folder, revision))
        pilots = []
        for pilot_folder in sorted(archive.glob("pilot*")):
            if not pilot_folder.is_dir():
                continue
            require((pilot_folder / "receipt.json").exists(), "partial pilot archive")
            folder = pilot_folder.name
            pilots.append({"folder": folder, "protocols": {"A": folder + "/tantivy.protocol.jsonl", "B": folder + "/port.protocol.jsonl"}, "controller": self.controller(archive, "pilot.py", "28e34a3482ccbe0dbbad943c7bdcc13c93e56ff8b7ed063609cf20d7a8a6446e"), "schedule": {"queries": [q["id"] for q in cases if q["id"] < 1221 and (q["kind"] == "OR" or "wiki20" in q["tags"] or q["id"] % 17 == 0)], "warmup": 4, "samples": 12, "orders": ["AB", "BA"], "seed": 20261006, "mode": "top10", "cpu": 4}, "limitations": ["Pilot has no matched self-control lane; all retained query cases and both orders remain, including background-load effects."]})
        bindings = []
        for field, group in (("two_term_primary_ratios", "OR_2"), ("longer_or_primary_ratios", "OR_3PLUS"), ("all_or_primary_ratios", "OR"), ("raw_or3_ratios", "OR_3PLUS")):
            if field in decision:
                bindings.append({"receipt_selector": [field], "result_selector": ["parent_lane", "groups", group, "primary"]})
        if "target_met" in decision:
            bindings.append({"receipt_selector": ["target_met"], "result_selector": ["actual_port_lane", "actual_port_target_met"]})
        if "queries_per_profile" in decision:
            bindings.append({"receipt_selector": ["queries_per_profile"], "result_selector": ["exact_profiles", "default", "queries"]})
        if "all_groups" in decision:
            bindings.append({"receipt_selector": ["all_groups"], "result_selector": ["parent_lane", "groups"]})
        groups = ["OR_2"] if qid == "01" else ["OR_3PLUS", "OR"] if qid in ("11", "22") else ["OR_3PLUS"]
        checks = [{"selector": [k], "expected": decision[k]} for k in ("commit", "candidate_binary_sha256", "correctness_tests", "tests_passed", "ignored_tests", "tests_ignored", "exact_profiles", "queries_per_profile", "source_sha256") if k in decision]
        descriptor = {"id": qid, "prefix": prefix, "outcome": outcome, "parent": parent, "candidate": candidate, "revisions": revisions, "guard_proofs": [{"selector": ["controller"], "artifact": self.controller(archive, "balanced.py", default["controller"])}], "stages": stages, "exact_gates": [{"profile": profile, "folder": folder, "protocol": folder + "/candidate.protocol.jsonl"} for profile, folder in (("default", "verify"), ("configured-k0.9-b0.4", "configured-k0.9-b0.4"), ("configured-k2-b1", "configured-k2-b1"))], "pilots": pilots, "parent_lane": self.lane(archive, prefix, qid), "actual_port_lane": self.lane(archive, prefix, qid, True), "diagnostics": self.profiles(archive, prefix, qid, parent["binary_sha256"], binaries["tantivy"]), "decision": {"artifact": prefix + "/" + decision_name, "recorded_verdict": decision["verdict"], "checks": checks, "result_bindings": bindings, "acceptance_rule": {"kind": "parent-group-controls" if outcome == "accepted" else "not-accepted", "groups": groups if outcome == "accepted" else []}}, "limitations": ["Recorded under user background load. Configured profiles are own-oracle correctness only. All retained source and guard hashes prove byte consistency, not source-to-binary authenticity or full Lucene API parity."]}
        if commit is not None:
            descriptor["accepted_commit"] = commit
        return descriptor

    def common(self):
        queries = self.copy(FROZEN / "queries.jsonl", "common/queries.jsonl", "frozen-queries")
        self.tree(FROZEN / "verify", "common/frozen-verify")
        self.tree(FROZEN / "audit-port", "common/payload-audit")
        self.tree(OUT / "baseline-gates", "common/baseline-gates")
        baseline_snapshot = self.copy(REFINEMENT / "build-reviewed/after.json", "common/baseline-compiled-snapshot.json", "baseline-compiled-snapshot")
        provenance = self.copy(OUT / "baseline-provenance.json", "common/baseline-provenance.json", "baseline-provenance-receipt")
        provenance_obj = obj(OUT / "baseline-provenance.json")
        provenance_bindings = []
        for path, expected in provenance_obj["ruler_sha256"].items():
            artifact = self.copy(OUT / path, "common/baseline-rulers/" + path, "frozen-ruler-source")
            require(self.files[artifact]["raw_sha256"] == expected, "baseline ruler changed " + path)
            provenance_bindings.append({"selector": ["ruler_sha256", path], "artifact": artifact})
        gate = obj(FROZEN / "verify/before.json")
        current = obj(OUT / "attempt-01/verify/before.json")
        harness = {path: self.copy(OUT / path, "common/hillclimb-harness/" + path, "harness-source") for path in current["harness"]}
        fixed_harness = {path: self.copy(FROZEN / path, "common/fixed-port-harness/" + path, "harness-source") for path in gate["harness"]}
        port_source = gate["source"]["lucene-rs-idf-fix-oct06"]
        port_bindings = {}
        # Full source hashes remain in the guard; retain executable Rust source
        # and package configuration rather than duplicating unrelated fixtures.
        port_repo = SEARCH / "lucene-rs-idf-fix-oct06"
        for path, expected in port_source["files"].items():
            if not (path.endswith(".rs") or Path(path).name in ("Cargo.toml", "Cargo.lock")):
                continue
            data = self.git(port_repo, PORT_COMMIT, path)
            if data is not None and sha(data) == expected:
                artifact = "sources/" + expected + "/" + Path(path).name
                if artifact not in self.files:
                    self.bytes(data, artifact, "fixed-port-source")
                    self.derivations.append({"artifact": artifact, "kind": "Git-object", "repository": str(port_repo), "commit": PORT_COMMIT, "path": path, "raw_sha256": expected})
                port_bindings[path] = artifact
            else:
                port_bindings[path] = self.source(path, expected, PORT_COMMIT, port_repo, port_repo)
        for name in ("stage.py", "balanced.py", "compare.py", "configured_gate.py", "pilot.py", "port_measure.py", "analyze.py", "analyze_port.py", "measure_audit.py", "port_group_audit.py"):
            self.copy(OUT / name, "common/controllers/" + file_sha(OUT / name) + "/" + name, "controller-source")
        return {"queries": queries, "frozen_gate": {**{key: "common/frozen-verify/" + name for key, name in (("dumps", "dumps.json"), ("receipt", "receipt.json"), ("failures", "failures.json"), ("before", "before.json"), ("after", "after.json"))}, "protocols": {"tantivy": "common/frozen-verify/tantivy.protocol.jsonl", "port": "common/frozen-verify/port.protocol.jsonl"}}, "baseline": {"binary_sha256": BASE_BINARY, "source_commit": BASE_COMMIT, "compiled_snapshot": baseline_snapshot, "exact_receipt": "common/baseline-gates/verify/receipt.json", "provenance": provenance, "provenance_bindings": provenance_bindings}, "fixed_port": {"binary_sha256": PORT_BINARY, "source_commit": PORT_COMMIT, "source_snapshot": {"artifact": "common/frozen-verify/before.json", "selector": ["source", "lucene-rs-idf-fix-oct06"]}, "source_bindings": port_bindings, "binary_proof": {"artifact": "common/frozen-verify/before.json", "selector": ["binaries", "port"]}, "harness": fixed_harness}, "indices": {"tantivy": {"artifact": "common/frozen-verify/before.json", "selector": ["indices", "tantivy"]}, "actual_port": {"artifact": "common/frozen-verify/before.json", "selector": ["indices", "port"]}}, "harness": harness, "guard_proofs": [{"selector": ["payload_audit"], "artifact": "common/payload-audit/stdout.log"}], "payload_audit": {"stdout": "common/payload-audit/stdout.log", "receipt": "common/payload-audit/receipt.json", "index_after": "common/payload-audit/index-after.json"}, "run_contract": self.copy(OUT / "run-contract.json", "common/run-contract.json", "frozen-contract"), "decision_trail": self.copy(OUT / "decision.tsv", "common/decision.tsv", "decision-trail"), "pilot_controller_sha256": file_sha(OUT / "pilot.py"), "comparison_controller": self.controller(OUT, "compare.py")}

    def pending(self, completed):
        ids = set()
        for path in OUT.glob("attempt-*"):
            match = re.match(r"attempt-([0-9]{2,})(?:-|\.|$)", path.name)
            if match and match.group(1) not in completed:
                ids.add(match.group(1))
        result = []
        for qid in sorted(ids, key=int):
            prefix = "attempts/" + qid
            archive = OUT / ("attempt-" + qid)
            if archive.is_dir():
                self.tree(archive, prefix)
            for path in sorted(OUT.glob("attempt-" + qid + "-*")):
                if path.is_file() and path.suffix in (".rs", ".patch", ".json", ".md"):
                    self.copy(path, prefix + "/source-only/" + path.name, "pending-evidence")
            reason = "No completed decision exists. Actual available source/evidence is inventoried without inventing a completed runtime or performance claim."
            result.append({"id": qid, "prefix": prefix, "outcome": "pending", "parent": None, "candidate": None, "revisions": {}, "guard_proofs": [], "stages": [], "exact_gates": [], "pilots": [], "parent_lane": {"status": "pending", "reason": reason}, "actual_port_lane": {"status": "pending", "reason": reason}, "diagnostics": [], "decision": None, "limitations": [reason]})
        return result

    def finish(self):
        self.bytes(json_bytes({"derivations": self.derivations, "exclusions": self.exclusions, "limits": ["Original absolute paths are provenance only. Engines/corpus/index payload excluded.", "Full archive raw telemetry, schedules, samples, logs and available profiles retained; unavailable historical profile01 remains unavailable.", "This assembly command is not an audit or a performance run. Root must run recompute.py and a separate clean committed archive check."]}), "packing-receipt.json", "packing-provenance")
        manifest = {"schema": "tantivy-or-hillclimb-audit-v1", "files": dict(sorted(self.files.items())), "origins": dict(sorted(self.origins.items()))}
        (self.destination / "packet-manifest.json").write_bytes(json_bytes(manifest))
        return {"destination": str(self.destination), "files": len(self.files) + 1, "stored_bytes": sum(v["stored_bytes"] for v in self.files.values()), "raw_bytes": sum(v["raw_bytes"] for v in self.files.values()), "manifest_sha256": file_sha(self.destination / "packet-manifest.json"), "audit_executed": False}

    def context(self):
        # These source/design/diagnostic artifacts sit outside attempt archives.
        # Keep them as context, without inventing a successful stage descriptor.
        for path in sorted(OUT.iterdir()):
            if path.is_file() and path.name not in ("portable-pack.py",) and (path.suffix in (".py", ".md", ".json", ".rs", ".patch", ".data", ".stderr", ".txt", ".html") or path.name == "decision.tsv"):
                self.copy(path, "context/" + path.name, "retained-context")
        for folder in ("region-diagnostic", "required-reach-run", "required-reach-run-retry-oct11", "batch-codegen-inspection", "batch-caller-inspection", "accepted11-long-or-annotation", "attempt-20-pre-review-strengthening"):
            if (OUT / folder).is_dir():
                self.tree(OUT / folder, "context/" + folder)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path, help="New immutable packet directory; must not exist")
    args = parser.parse_args()
    require(not args.destination.exists(), "destination already exists; inspect prior assembly rather than overwrite")
    packer = Packer(args.destination.resolve())
    common = packer.common()
    cases = [json.loads(line) for line in (FROZEN / "queries.jsonl").read_text().splitlines()]
    with (OUT / "decision.tsv").open() as stream:
        trail = list(csv.DictReader(stream, delimiter="\t"))
    completed = [row["id"] for row in trail]
    require(len(completed) == len(set(completed)) and all(f"{i:02d}" in completed for i in range(1, 15)), "complete historic decision inventory01..14 required")
    attempts = [packer.attempt(row["id"], "accepted" if row["verdict"] == "kept" else "rejected", common["baseline"], cases) for row in trail]
    attempts.extend(packer.pending(set(completed)))
    packer.context()
    descriptor = {"schema": "tantivy-or-hillclimb-attempts-v1", "common": common, "attempts": attempts, "required_attempt_ids": [d["id"] for d in attempts]}
    packer.bytes(json_bytes(descriptor), "attempts.json", "packet-relative-descriptor")
    packer.copy(OUT / "portable-audit-draft/recompute.py", "recompute.py", "standalone-auditor")
    readme = (OUT / "portable-audit-draft/README.md").read_text()
    original_intro = readme.split("\n\n", 2)
    require(len(original_intro) == 3, "auditor README intro shape changed")
    readme = original_intro[0] + "\n\nThis packet contains assembled retained hillclimb evidence. Assembly did not execute a benchmark, engine, correctness test or the standalone audit. Run the retained auditor to verify its bytes and recorded arithmetic; rejected trials and pending source-only work remain explicit.\n\n" + original_intro[2]
    readme += "\nAssembly note: large artifacts keep their original logical filenames while manifest.encoding records gzip; do not infer encoding from filename. Read through the auditor or gzip.open as declared. packing-receipt.json explicitly distinguishes original copies, Git-object sources, and any hash-matched reconstruction. Outputs from verification belong outside this exact-inventory directory.\n"
    packer.bytes(readme.encode(), "README.md", "packet-documentation")
    packer.copy(OUT / "results-current.md", "results.md", "readable-results")
    packer.copy(OUT / "search-architecture-current.html", "search-architecture-current.html", "accepted-architecture")
    packer.copy(Path(__file__), "portable-pack.py", "assembly-source")
    require(file_sha(OUT / "decision.tsv") == packer.files[common["decision_trail"]]["raw_sha256"], "decision trail changed during assembly; do not publish a moving inventory")
    print(json.dumps(packer.finish(), indent=2))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, StopIteration) as error:
        print(json.dumps({"assembled": False, "error": str(error), "note": "Partial destination is retained for inspection; no historical inputs changed."}), file=sys.stderr)
        sys.exit(1)
