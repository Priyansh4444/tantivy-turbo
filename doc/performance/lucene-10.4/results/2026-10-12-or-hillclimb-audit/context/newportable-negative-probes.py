#!/usr/bin/env python3
"""Root-only negative probes against an extracted, immutable evidence packet.

Run separately from the successful full audit, with stdout outside the packet.
No engines/subprocesses are run. Real artifacts are read once for common/setup
checks; mutations live in overlays or tiny temporary inventory fixtures. These
probes check rejection behavior, not historical performance or authorship.
"""
import argparse
import contextlib
import copy
import gzip
import hashlib
import itertools
import json
from pathlib import Path
import runpy
import sys
import tempfile


def sha(data):
    return hashlib.sha256(data).hexdigest()


class Overlay:
    """Replace selected reads without touching any published artifact bytes."""
    def __init__(self, packet, objects=None, lines=None, raws=None, hashes=None):
        self.base = packet
        self.objects = objects or {}
        self.line_views = lines or {}
        self.raws = raws or {}
        self.hashes = hashes or {}

    def __getattr__(self, key):
        return getattr(self.base, key)

    def obj(self, name):
        return copy.deepcopy(self.objects[name]) if name in self.objects else self.base.obj(name)

    def raw(self, name):
        return self.raws[name] if name in self.raws else self.base.raw(name)

    def hash(self, name):
        return self.hashes.get(name, self.base.hash(name))

    def lines(self, name):
        rows = self.base.lines(name)
        return self.line_views[name](rows) if name in self.line_views else rows

    def ref(self, reference):
        # References must use the overlay rather than a bound base Packet.ref.
        value = self.obj(reference["artifact"])
        for key in reference.get("selector", []):
            value = value[key]
        return value


class Probes:
    def __init__(self, error_type):
        self.error_type = error_type
        self.results = []

    def run(self, name, action, rejection=None):
        try:
            action()
        except self.error_type as error:
            text = str(error)
            passed = rejection is not None and rejection in text
            result = {"name": name, "pass": passed, "expected": rejection,
                      "observed": "AuditError", "invariant": text}
        except Exception as error:
            result = {"name": name, "pass": False, "expected": rejection,
                      "observed": type(error).__name__, "error": str(error)}
        else:
            result = {"name": name, "pass": rejection is None,
                      "expected": rejection, "observed": "accepted"}
        self.results.append(result)


def first_change(change):
    def transform(rows):
        first = copy.deepcopy(next(rows))
        change(first)
        yield first
        yield from rows
    return transform


def reorder(rows):
    first, second = next(rows), next(rows)
    yield second
    yield first
    yield from rows


def duplicate(rows):
    first = next(rows)
    yield first
    yield copy.deepcopy(first)
    yield from rows


def missing(rows):
    next(rows)
    yield from rows


def inserted(rows):
    first = next(rows)
    yield first
    extra = copy.deepcopy(first)
    extra["id"] = 1226  # Correctness-only ID cannot be a timed schedule cell.
    yield extra
    yield from rows


@contextlib.contextmanager
def packet_root(api, path):
    # runpy function globals own ROOT; changing the returned mapping alone is
    # not a reliable way to relocate class methods across Python versions.
    globals_ = api["Packet"].__init__.__globals__
    prior = globals_["ROOT"]
    globals_["ROOT"] = path
    try:
        yield
    finally:
        globals_["ROOT"] = prior


def mini_packet(api, corruption=None):
    """Tiny inventory-only positive control, then one targeted corruption."""
    with tempfile.TemporaryDirectory(prefix="portable-audit-probe-") as temporary:
        root = Path(temporary)
        files = {"recompute.py": b"# inventory fixture only\n",
                 "README.md": b"inventory fixture\n", "attempts.json": b"{}\n",
                 "payload.json": b'{"value":1}\n'}
        entries = {}
        for name, raw in files.items():
            (root / name).write_bytes(raw)
            entries[name] = {"encoding": "identity", "stored_bytes": len(raw),
                             "stored_sha256": sha(raw), "raw_bytes": len(raw),
                             "raw_sha256": sha(raw), "role": "negative-probe-fixture"}
        manifest = {"schema": "tantivy-or-hillclimb-audit-v1",
                    "files": entries, "origins": {}}
        if corruption is not None:
            corruption(root, manifest)
        manifest_path = root / "packet-manifest.json"
        manifest_path.write_text(json.dumps(manifest))
        with packet_root(api, root):
            api["Packet"]()


def replace_file(root, manifest, name, raw):
    (root / name).write_bytes(raw)
    manifest["files"][name].update(stored_bytes=len(raw), stored_sha256=sha(raw),
                                 raw_bytes=len(raw), raw_sha256=sha(raw))


def inventory_probes(api, probes):
    probes.run("inventory/minimal-good", lambda: mini_packet(api))
    probes.run("inventory/extra-file", lambda: mini_packet(api, lambda root, m:
               (root / "unexpected").write_bytes(b"extra")), "exact packet inventory")
    probes.run("inventory/missing-file", lambda: mini_packet(api, lambda root, m:
               (root / "payload.json").unlink()), "exact packet inventory")

    def symlink(root, manifest):
        (root / "payload.json").unlink()
        (root / "payload.json").symlink_to(root / "README.md")
    probes.run("inventory/symlink", lambda: mini_packet(api, symlink), "packet contains symlink")

    def stored_hash(root, manifest):
        manifest["files"]["payload.json"]["stored_sha256"] = "0" * 64
    probes.run("inventory/stored-hash", lambda: mini_packet(api, stored_hash), "stored bytes/hash")

    def raw_hash(root, manifest):
        manifest["files"]["payload.json"]["raw_sha256"] = "0" * 64
    probes.run("inventory/raw-hash", lambda: mini_packet(api, raw_hash), "raw bytes/hash")

    def encoded(mtime):
        def mutate(root, manifest):
            raw = (root / "payload.json").read_bytes()
            stored = gzip.compress(raw, mtime=mtime)
            (root / "payload.json").write_bytes(stored)
            manifest["files"]["payload.json"].update(
                encoding="gzip", stored_bytes=len(stored), stored_sha256=sha(stored))
        return mutate
    probes.run("inventory/gzip-zero-mtime-good", lambda: mini_packet(api, encoded(0)))
    probes.run("inventory/gzip-nonzero-mtime", lambda: mini_packet(api, encoded(1)), "gzip mtime must be zero")
    probes.run("inventory/duplicate-json-key", lambda: mini_packet(api, lambda root, m:
               replace_file(root, m, "attempts.json", b'{"x":1,"x":2}')), "duplicate JSON key")
    probes.run("inventory/nonfinite-json", lambda: mini_packet(api, lambda root, m:
               replace_file(root, m, "attempts.json", b'{"x":NaN}')), "nonfinite JSON")
    def traversal(root, manifest):
        manifest["files"]["../outside"] = manifest["files"].pop("payload.json")
    probes.run("inventory/traversal-entry", lambda: mini_packet(api, traversal), "exact packet inventory")
    for name in ("../outside", "/absolute", "a/../payload.json", "a//payload.json", "a\\payload.json", "a:payload.json"):
        probes.run("path/" + name, lambda name=name: api["relative"](name), "relative path")
    probes.run("path/canonical-good", lambda: api["relative"]("a/payload.json"))


def attempt_view(api, audit, original, packet=None, descriptor=None):
    view = copy.copy(audit)
    view.packet = packet or audit.packet
    trial = api["Attempt"](view, copy.deepcopy(descriptor or original.d))
    # These are already established by original.setup(), not fabricated exact
    # values. Individual probes intentionally invoke the owning boundary next.
    trial.exact = original.exact
    trial.compiled = original.compiled
    trial.source_files = original.source_files
    return trial


def parent_lane_probes(api, probes, audit, trial):
    packet, lane = audit.packet, trial.d["parent_lane"]
    if lane["status"] != "retained":
        raise ValueError("Attempt11 must have its authentic complete parent lane")
    raw_name = trial.name(lane["folder"] + "/samples.jsonl")
    first_raw = next(packet.lines(raw_name))
    probes.run("timing/authentic-first-cell-good", lambda: api["validate_run"](
        first_raw, first_raw["id"], first_raw["mode"], 16,
        trial.exact["default"][first_raw["id"]]))

    def lane_overlay(lines=None, objects=None):
        view = attempt_view(api, audit, trial, Overlay(packet, objects=objects, lines=lines))
        return view.lane(lane)

    for label, value in (("boolean-ns", True), ("zero-ns", 0)):
        probes.run("timing/" + label, lambda value=value: lane_overlay(lines={
            raw_name: first_change(lambda row: row["ns"].__setitem__(0, value))}), "positive integer ns")
    probes.run("timing/wrong-checksum", lambda: lane_overlay(lines={raw_name:
               first_change(lambda row: row.__setitem__("checksum", (row["checksum"] + 1) % 2**64))}), "timed checksum")
    for label, transform in (("reordered", reorder), ("duplicate", duplicate),
                             ("missing", missing), ("extra-inserted", inserted)):
        probes.run("timing/" + label + "-cell", lambda transform=transform:
                   lane_overlay(lines={raw_name: transform}), "raw complete schedule/seed/row order")
    schedule_name = trial.name(lane["folder"] + "/schedule.json")
    schedule = packet.obj(schedule_name)
    schedule["queries"] = schedule["queries"][:-1]
    probes.run("timing/omitted-schedule-query", lambda: lane_overlay(objects={schedule_name: schedule}),
               "frozen schedule/queries")

    # Corrupt both guards identically so the before/after equality does not
    # conceal the stronger source and parent identity boundaries.
    before_name = trial.name(lane["folder"] + "/before.json")
    after_name = trial.name(lane["folder"] + "/after.json")
    guard = packet.obj(before_name)
    wrong_guard = copy.deepcopy(guard)
    path = trial.candidate["changed_paths"][0]
    wrong_guard["source"][trial.candidate["snapshot_key"]]["files"][path] = "0" * 64
    probes.run("timing/wrong-compiled-source-guards", lambda: lane_overlay(objects={
               before_name: wrong_guard, after_name: wrong_guard}), "guard full compiled source snapshot")
    wrong_guard = copy.deepcopy(guard)
    wrong_guard["binaries"]["port"] = "0" * 64
    probes.run("timing/wrong-parent-binary-guards", lambda: lane_overlay(objects={
               before_name: wrong_guard, after_name: wrong_guard}), "guard binaries")

    protocol_name = trial.name(lane["protocols"]["0"]["A"])
    probes.run("protocol/wrong-ready", lambda: lane_overlay(lines={protocol_name:
               first_change(lambda row: row["response"].__setitem__("max_doc", 999999))}), "READY/max_doc")

    def wrong_request(rows):
        yield next(rows)
        request = copy.deepcopy(next(rows))
        request["request"]["id"] = 1226
        yield request
        yield from rows
    probes.run("protocol/wrong-request-order", lambda: lane_overlay(lines={protocol_name: wrong_request}),
               "protocol request order")

    def wrong_response(rows):
        yield next(rows)
        yield next(rows)
        response = copy.deepcopy(next(rows))
        response["response"]["checksum"] = (response["response"]["checksum"] + 1) % 2**64
        yield response
        yield from rows
    probes.run("protocol/wrong-warmup-checksum", lambda: lane_overlay(lines={protocol_name: wrong_response}),
               "timed checksum")

    # End-of-stream checking does not need to rescan 87,912 cells. Use an actual
    # bounded READY/request/response prefix; only the appended record is new.
    prefix = list(itertools.islice(packet.lines(protocol_name), 3))
    request = prefix[1]["request"]
    exact = trial.exact["default"][request["id"]]
    def prefix_protocol(extra=False):
        rows = prefix + ([copy.deepcopy(prefix[2])] if extra else [])
        overlay = Overlay(packet, lines={protocol_name: lambda ignored: iter(copy.deepcopy(rows))})
        reader = api["Protocol"](overlay, protocol_name)
        response = reader.exchange(request)
        api["validate_run"](response, request["id"], request["mode"], request["iterations"], exact)
        reader.end()
    probes.run("protocol/authentic-prefix-good", prefix_protocol)
    probes.run("protocol/extra-trailing-record", lambda: prefix_protocol(True), "extra protocol records")


def source_gate_probes(api, probes, audit, trial):
    probes.run("source/authentic-binding-good", lambda: trial.bind_source(trial.compiled, trial.candidate))
    wrong = copy.deepcopy(trial.compiled)
    source_path = trial.candidate["changed_paths"][0]
    wrong[trial.candidate["snapshot_key"]]["files"][source_path] = "0" * 64
    probes.run("source/wrong-source-sha", lambda: trial.bind_source(wrong, trial.candidate),
               "compiled changed source binding")
    descriptor = copy.deepcopy(trial.d)
    descriptor["parent"]["binary_sha256"] = "0" * 64
    probes.run("parent/wrong-binary", lambda: attempt_view(api, audit, trial, descriptor=descriptor).check_parent(),
               "parent chain binary")
    descriptor = copy.deepcopy(trial.d)
    descriptor["parent"]["source_commit"] = "0" * 40
    probes.run("parent/wrong-commit", lambda: attempt_view(api, audit, trial, descriptor=descriptor).check_parent(),
               "parent chain commit")
    probes.run("parent/authentic-proof-good", trial.check_parent)

    gate = next(g for g in trial.d["exact_gates"] if g["profile"] == "default")
    dumps_name = trial.name(gate["folder"] + "/dumps.json")
    dumps = audit.packet.obj(dumps_name)
    row = next(r for r in dumps if r["tantivy"]["hits"] and
               0 < r["tantivy"]["hits"][-1][1] < 0x7f800000)
    # Lower the final positive score by one representable step, preserving
    # finite range and hit order. The independent oracle remains authentic.
    row["tantivy"]["hits"][-1][1] -= 1
    probes.run("exact/wrong-raw-score-bit", lambda: attempt_view(api, audit, trial,
               Overlay(audit.packet, objects={dumps_name: dumps})).gate(gate), "optimized/exhaustive raw bits")


def historical_probes(api, probes, audit):
    for qid in ("08", "09", "10", "12", "16", "19"):
        original = api["Attempt"](audit, copy.deepcopy(audit.descriptors[qid]))
        failed = [s for s in original.d["stages"]
                  if s["revision"] != "final" and s["expected_returncode"] != 0]
        if not failed:
            raise ValueError("Missing authentic failure revision " + qid)
        probes.run("failure/" + qid + "/authentic-history-good", original.historic_failures)
        for stage in failed:
            probes.run("failure/" + qid + "/authentic-stage-good/" + stage["folder"],
                       lambda stage=stage, original=original: original.stage(stage))
        omitted = copy.deepcopy(original.d)
        omitted["stages"] = [s for s in omitted["stages"] if s not in failed]
        probes.run("failure/" + qid + "/omitted-history", lambda omitted=omitted:
                   api["Attempt"](audit, omitted).historic_failures(), "historic expected failure may not be omitted")
        stage = copy.deepcopy(failed[0])
        stage["expected_returncode"] = 0
        probes.run("failure/" + qid + "/relabelled-success", lambda stage=stage, original=original:
                   original.stage(stage), "stage expected status")
        stage = copy.deepcopy(failed[0])
        stage["required_log_fragments"] = {"stderr": ["negative probe: absent authentic signature"]}
        probes.run("failure/" + qid + "/wrong-signature", lambda stage=stage, original=original:
                   original.stage(stage), "required retained failure/log signature")
        if qid in ("09", "12", "16", "19"):
            stage = copy.deepcopy(failed[0])
            if type(stage["tests"]) is not dict:
                raise ValueError("Authentic test-failure totals unavailable " + qid)
            stage["tests"]["failed"] = 0
            probes.run("failure/" + qid + "/erased-test-failures", lambda stage=stage, original=original:
                       original.stage(stage), "retained test totals")


def decision_probes(api, probes, audit):
    def identity_view(qid, change_receipt=None, verdict=None):
        descriptor = copy.deepcopy(audit.descriptors[qid])
        # These probes isolate decision identity, not arithmetic. Removing the
        # self-expected checks makes wrong claims fail their independent source
        # or binary bindings rather than accidentally failing a duplicate check.
        descriptor["decision"]["checks"] = []
        descriptor["decision"]["result_bindings"] = []
        if verdict is not None:
            descriptor["decision"]["recorded_verdict"] = verdict
        packet = audit.packet
        artifact = descriptor["decision"]["artifact"]
        if change_receipt is not None:
            receipt = packet.obj(artifact)
            change_receipt(receipt)
            packet = Overlay(packet, objects={artifact: receipt})
        view = copy.copy(audit)
        view.packet = packet
        trial = api["Attempt"](view, descriptor)
        trial.compiled = packet.obj(trial.name(trial.candidate["snapshot"]))
        trial.source_files = trial.bind_source(trial.compiled, trial.candidate)
        return trial.decision({})  # All selected trials are rejected.

    probes.run("decision/authentic17-identity-good", lambda: identity_view("17"))
    probes.run("decision/unknown-verdict", lambda: identity_view("17", verdict="reverted_unproven_magic"),
               "recorded decision verdict")
    probes.run("decision/rejected-wrong-candidate-binary", lambda: identity_view("17", lambda receipt:
               receipt.__setitem__("candidate_binary_sha256", "0" * 64)), "decision candidate binary")
    probes.run("decision/rejected-wrong-source-map", lambda: identity_view("17", lambda receipt:
               receipt["source_sha256"].__setitem__("src/query/boolean_query/or_maxscore.rs", "0" * 64)),
               "decision compiled source SHA")
    probes.run("decision/authentic02-identity-good", lambda: identity_view("02"))
    probes.run("decision/rejected-wrong-legacy-binary", lambda: identity_view("02", lambda receipt:
               receipt.__setitem__("binary_sha256", "0" * 64)), "decision legacy candidate binary")
    probes.run("decision/rejected-wrong-scalar-source", lambda: identity_view("02", lambda receipt:
               receipt.__setitem__("source_sha256", "0" * 64)), "decision compiled source SHA")

    def accepted_wrong_binary():
        descriptor = copy.deepcopy(audit.descriptors["11"])
        descriptor["decision"]["checks"] = []
        descriptor["decision"]["result_bindings"] = []
        artifact = descriptor["decision"]["artifact"]
        receipt = audit.packet.obj(artifact)
        receipt["candidate_binary_sha256"] = "0" * 64
        view = copy.copy(audit)
        view.packet = Overlay(audit.packet, objects={artifact: receipt})
        trial = api["Attempt"](view, descriptor)
        trial.compiled = view.packet.obj(trial.name(trial.candidate["snapshot"]))
        trial.source_files = trial.bind_source(trial.compiled, trial.candidate)
        trial.decision({})  # Must fail identity before accepted arithmetic.
    probes.run("decision/accepted-wrong-candidate-binary", accepted_wrong_binary, "decision candidate binary")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("packet", type=Path, help="Existing extracted populated packet; never modified")
    args = parser.parse_args()
    packet = args.packet.resolve(strict=True)
    sys.dont_write_bytecode = True
    if not packet.is_dir() or (packet / "recompute.py").is_symlink():
        raise ValueError("Packet must be a real extracted directory with a plain auditor")
    auditor = packet / "recompute.py"
    api = runpy.run_path(str(auditor), run_name="portable_negative_probe_import")
    probes = Probes(api["AuditError"])
    with packet_root(api, packet):
        # Audit.__init__ checks inventory and invokes load_common exactly once.
        audit = api["Audit"]()
        trial = api["Attempt"](audit, copy.deepcopy(audit.descriptors["11"]))
        trial.setup()  # Real source/parent/stages and all three exact gates once.
        source_gate_probes(api, probes, audit, trial)
        parent_lane_probes(api, probes, audit, trial)
        historical_probes(api, probes, audit)
        decision_probes(api, probes, audit)
        inventory_probes(api, probes)
    passed = all(r["pass"] for r in probes.results)
    print(json.dumps({"negative_probes_pass": passed, "count": len(probes.results),
                      "passed": sum(r["pass"] for r in probes.results), "probes": probes.results,
                      "auditor_sha256": sha(auditor.read_bytes()),
                      "packet_manifest_sha256": sha((packet / "packet-manifest.json").read_bytes()),
                      "full_successful_audit": "separate Root run required",
                      "packet_mutations": 0, "engine_or_benchmark_executions": 0},
                     indent=2, allow_nan=False))
    return 0 if passed else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(json.dumps({"negative_probes_pass": False, "error_type": type(error).__name__,
                          "error": str(error)}), file=sys.stderr)
        sys.exit(1)
