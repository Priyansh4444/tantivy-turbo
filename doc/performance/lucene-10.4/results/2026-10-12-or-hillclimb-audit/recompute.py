#!/usr/bin/env python3
"""Audit an explicitly described, extracted hillclimb packet.

Python stdlib only; no engines, builds, Git, subprocesses, or absolute proof paths.
This program checks retained evidence. It does not authenticate its author or
prove that historical binaries were compiled from the recorded source.
"""
import collections
import contextlib
import csv
import gzip
import hashlib
import io
import json
import math
from pathlib import Path, PurePosixPath
import random
import re
import statistics
import struct
import sys
import zlib

ROOT = Path(__file__).resolve().parent
ORDERS = ("AB", "BA")
KINDS = ("ALL", "TERM", "AND", "OR", "OR_3PLUS", "OR_2", "PHRASE")
PROFILE_BITS = {
    "default": ("3f99999a", "3f400000"),
    "configured-k0.9-b0.4": ("3f666666", "3ecccccd"),
    "configured-k2-b1": ("40000000", "3f800000"),
}
SHA_RE = re.compile(r"[0-9a-f]{64}\Z")
TEST_RE = re.compile(
    r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; "
    r"(\d+) ignored; (\d+) measured; (\d+) filtered out"
)
COMPILER_TEXT = (
    "rustc 1.99.0 (b940084d7 2026-09-28)\n"
    "binary: rustc\n"
    "commit-hash: b940084d7eb6a299eb4bfeb8e34901bc051e7ac4\n"
    "commit-date: 2026-09-28\n"
    "host: x86_64-unknown-linux-gnu\n"
    "release: 1.99.0\n"
    "LLVM version: 23.1.1\n"
)


class AuditError(Exception):
    pass


def check(ok, context):
    if not ok:
        raise AuditError(context)


def shape(value, required, optional=(), context="object"):
    check(type(value) is dict, context + ": expected object")
    check(set(required) <= set(value), context + ": missing keys")
    check(set(value) <= set(required) | set(optional), context + ": unknown keys")
    return value


def integer(value, minimum=0, context="integer"):
    check(type(value) is int and value >= minimum, context)
    return value


def digest(data):
    return hashlib.sha256(data).hexdigest()


def stream_hash(stream, maximum):
    count, hash_value = 0, hashlib.sha256()
    for block in iter(lambda: stream.read(1024 * 1024), b""):
        count += len(block)
        check(count <= maximum, "artifact exceeds declared byte length")
        hash_value.update(block)
    return count, hash_value.hexdigest()


def sha_value(value, context="SHA256"):
    check(type(value) is str and SHA_RE.fullmatch(value), context)
    return value


def pairs_unique(pairs):
    result = {}
    for key, value in pairs:
        check(key not in result, "duplicate JSON key: " + key)
        result[key] = value
    return result


def parse_json(data, context):
    try:
        return json.loads(
            data, object_pairs_hook=pairs_unique,
            parse_constant=lambda value: (_ for _ in ()).throw(
                AuditError(context + ": nonfinite JSON " + value)
            ),
        )
    except (ValueError, UnicodeError) as error:
        raise AuditError(context + ": invalid JSON") from error


def equal(actual, expected, context):
    """Strict shape/types except finite int/float numerical comparison."""
    if type(expected) is dict:
        check(type(actual) is dict and set(actual) == set(expected), context + ": keys")
        for key in expected:
            equal(actual[key], expected[key], context + "/" + str(key))
    elif type(expected) is list:
        check(type(actual) is list and len(actual) == len(expected), context + ": length")
        for number, (a, b) in enumerate(zip(actual, expected)):
            equal(a, b, context + "/" + str(number))
    elif type(expected) in (int, float):
        check(type(actual) in (int, float), context + ": numeric type")
        check(math.isfinite(actual) and math.isfinite(expected), context + ": finite")
        if type(expected) is int:
            check(type(actual) is int and actual == expected, context + ": integer mismatch")
        else:
            check(math.isclose(actual, expected, rel_tol=1e-12, abs_tol=1e-12),
                  context + ": numerical mismatch")
    else:
        check(type(actual) is type(expected) and actual == expected, context + ": mismatch")


def select(value, keys, context):
    check(type(keys) is list, context + ": selector must be array")
    for key in keys:
        check(type(value) is dict and type(key) is str and key in value,
              context + ": selector missing " + str(key))
        value = value[key]
    return value


def relative(name):
    check(type(name) is str and bool(name) and "\\" not in name, "invalid relative path")
    path = PurePosixPath(name)
    check(not path.is_absolute() and all(p not in ("", ".", "..") for p in name.split("/")),
          "unsafe relative path: " + name)
    check(str(path) == name and ":" not in name, "noncanonical relative path: " + name)
    return name


class Packet:
    def __init__(self):
        manifest_path = ROOT / "packet-manifest.json"
        check(manifest_path.is_file() and not manifest_path.is_symlink(), "manifest missing/symlink")
        self.manifest = shape(parse_json(manifest_path.read_bytes(), "manifest"),
                              ("schema", "files", "origins"), context="manifest")
        check(self.manifest["schema"] == "tantivy-or-hillclimb-audit-v1", "manifest schema")
        files = self.manifest["files"]
        check(type(files) is dict and bool(files), "empty manifest inventory")
        actual = set()
        for path in ROOT.rglob("*"):
            check(not path.is_symlink(), "packet contains symlink: " + str(path))
            if path.is_file():
                actual.add(path.relative_to(ROOT).as_posix())
            else:
                check(path.is_dir(), "packet contains special file")
        check(actual == set(files) | {"packet-manifest.json"}, "exact packet inventory")
        for name, entry in files.items():
            relative(name)
            check(name != "packet-manifest.json", "manifest cannot hash itself")
            shape(entry, ("encoding", "stored_bytes", "stored_sha256", "raw_bytes", "raw_sha256", "role"),
                  context="manifest file " + name)
            check(entry["encoding"] in ("identity", "gzip"), "unsupported encoding")
            integer(entry["stored_bytes"]); integer(entry["raw_bytes"])
            sha_value(entry["stored_sha256"]); sha_value(entry["raw_sha256"])
            check(type(entry["role"]) is str and bool(entry["role"]), "artifact role missing")
            with (ROOT / name).open("rb") as stream:
                stored_count, stored_sha = stream_hash(stream, entry["stored_bytes"])
            check(stored_count == entry["stored_bytes"] and stored_sha == entry["stored_sha256"],
                  "stored bytes/hash: " + name)
            if entry["encoding"] == "gzip":
                with (ROOT / name).open("rb") as stream:
                    header = stream.read(10)
                check(len(header) == 10 and header[:3] == b"\x1f\x8b\x08" and
                      header[4:8] == b"\x00\x00\x00\x00", "gzip mtime must be zero: " + name)
            with self.open_raw(name, entry) as stream:
                raw_count, raw_sha = stream_hash(stream, entry["raw_bytes"])
            check(raw_count == entry["raw_bytes"] and raw_sha == entry["raw_sha256"],
                  "raw bytes/hash: " + name)
        # Both executable auditor and attempt descriptor must be inventoried.
        for name in ("recompute.py", "attempts.json", "README.md"):
            check(name in files and files[name]["encoding"] == "identity", "required plain file: " + name)
        origins = self.manifest["origins"]
        check(type(origins) is dict and set(origins) <= set(files), "origin inventory")
        for name, origin in origins.items():
            shape(origin, ("original_path", "original_bytes", "original_sha256"), context="copy origin")
            check(type(origin["original_path"]) is str and origin["original_path"], "origin path")
            equal(origin["original_bytes"], files[name]["raw_bytes"], "origin raw length")
            equal(origin["original_sha256"], files[name]["raw_sha256"], "origin raw SHA")
        self.files = files
        self.descriptor = self.obj("attempts.json")

    @staticmethod
    @contextlib.contextmanager
    def open_raw(name, entry):
        opener = gzip.open if entry["encoding"] == "gzip" else open
        try:
            with opener(ROOT / name, "rb") as stream:
                yield stream
        except (OSError, EOFError, zlib.error) as error:
            raise AuditError("unreadable/invalid encoded artifact: " + name) from error

    @staticmethod
    def decode(stored, entry, name):
        if entry["encoding"] == "identity":
            return stored
        try:
            with gzip.GzipFile(fileobj=io.BytesIO(stored)) as stream:
                raw = stream.read(entry["raw_bytes"] + 1)
                check(len(raw) <= entry["raw_bytes"], "oversized decoded file: " + name)
                check(stream.read(1) == b"", "extra decoded bytes: " + name)
                return raw
        except (OSError, EOFError, zlib.error) as error:
            raise AuditError("invalid gzip: " + name) from error

    def raw(self, name):
        relative(name)
        check(name in self.files, "undeclared artifact: " + name)
        entry = self.files[name]
        return self.decode((ROOT / name).read_bytes(), entry, name)

    def hash(self, name):
        relative(name)
        check(name in self.files, "undeclared hash artifact: " + name)
        return self.files[name]["raw_sha256"]

    def obj(self, name):
        return parse_json(self.raw(name), name)

    def ref(self, reference):
        shape(reference, ("artifact",), ("selector",), "proof reference")
        return select(self.obj(reference["artifact"]), reference.get("selector", []), "proof reference")

    def lines(self, name):
        relative(name)
        check(name in self.files, "undeclared JSONL artifact: " + name)
        with self.open_raw(name, self.files[name]) as stream:
            for number, line in enumerate(stream, 1):
                check(bool(line.strip()), name + ": blank JSONL row")
                yield parse_json(line, name + ":" + str(number))


def validate_dump(case, value, oracle=False):
    check(type(value) is dict, "dump object")
    if not oracle:
        check(value.get("id") == case["id"] and type(value.get("id")) is int, "dump ID")
    count = integer(value.get("count"), context="dump count")
    check(count <= 1000000, "dump count exceeds physical document count")
    hits = value.get("hits")
    check(type(hits) is list and len(hits) == min(10, count), "dump hits/count")
    previous = None
    documents = set()
    for hit in hits:
        check(type(hit) is list and len(hit) == 2, "hit shape")
        doc, bits = hit
        integer(doc); integer(bits)
        check(doc < 1000000 and bits < 2**32 and doc not in documents, "hit range/duplicate")
        documents.add(doc)
        score = struct.unpack("<f", struct.pack("<I", bits))[0]
        check(math.isfinite(score), "nonfinite score")
        key = (-score, doc)
        check(previous is None or key >= previous, "score/doc order")
        previous = key
    if not oracle:
        check("reported" in value, "missing reported total")
        report = value["reported"]
        if report is not None:
            shape(report, ("relation", "value"), context="reported total")
            total = integer(report["value"])
            check((report["relation"] == "eq" and total == count) or
                  (report["relation"] == "gte" and len(hits) <= total <= count), "reported total")
    return {"count": count, "hits": hits}


def checksum(value, mode, iterations):
    total = value["count"] if mode == "count" else sum(doc ^ bits for doc, bits in value["hits"])
    return (total * iterations) % 2**64


def validate_run(value, qid, mode, n, exact):
    check(type(value) is dict and type(value.get("id")) is int and value["id"] == qid,
          "timed response ID")
    check(value.get("mode") == mode, "timed mode")
    ns = value.get("ns")
    check(type(ns) is list and len(ns) == n, "timing sample count")
    for sample in ns:
        integer(sample, 1, "positive integer ns")
    integer(value.get("checksum"), context="checksum integer")
    check(value["checksum"] < 2**64 and value["checksum"] == checksum(exact, mode, n),
          "timed checksum")
    return ns


def ready(value, profile, actual_port=False):
    check(type(value) is dict, "READY object")
    expected = {"op": "ready", "max_doc": 1000000, "num_docs": 1000000,
                "doc_count": 917578, "sum_total_term_freq": 294827020, "queries": 1227,
                "k1_bits": PROFILE_BITS[profile][0], "b_bits": PROFILE_BITS[profile][1]}
    for key, wanted in expected.items():
        equal(value.get(key), wanted, "READY/" + key)
    check(value.get("engine") == ("lucene-rs" if actual_port else "tantivy"), "READY engine semantics")
    if not actual_port:
        check(value.get("collector") == "top10-no-totals" and value.get("query_cache") == "disabled",
              "Tantivy collector/cache READY")
    else:
        equal(value.get("collector"), "top10-totals-threshold1000", "actual-port collector")
        equal(value.get("total_hits_threshold"), 1000, "actual-port total-hit threshold")
        equal(value.get("query_cache"), "disabled", "actual-port cache")
        equal(value.get("duplicate_boolean_terms"), "coalesced_into_summed_boost", "actual-port duplicate handling")


class Protocol:
    def __init__(self, packet, name, profile="default", actual_port=False):
        self.rows = iter(packet.lines(name))
        first = next(self.rows, None)
        shape(first, ("response",), context="protocol READY")
        self.ready = first["response"]
        ready(self.ready, profile, actual_port)

    def exchange(self, request):
        req = next(self.rows, None)
        res = next(self.rows, None)
        shape(req, ("request",), context="protocol request")
        shape(res, ("response",), context="protocol response")
        equal(req["request"], request, "protocol request order")
        return res["response"]

    def end(self):
        check(next(self.rows, None) is None, "extra protocol records")


def group_match(case, suite, kind):
    belongs = ("wiki20" in case["tags"]) if suite == "wiki20" else ("port-style" in case["tags"])
    return belongs and (kind == "ALL" or case["kind"] == kind or
                       case["kind"] == "OR" and
                       ((kind == "OR_3PLUS" and len(case["terms"]) >= 3) or
                        (kind == "OR_2" and len(case["terms"]) == 2)))


def geo(values):
    return math.exp(statistics.mean(math.log(value) for value in values))


def csv_check(packet, name, expected):
    reader = csv.DictReader(io.StringIO(packet.raw(name).decode("utf-8")))
    check(bool(expected) and reader.fieldnames == list(expected[0]), "CSV header/order: " + name)
    saved = list(reader)
    check(len(saved) == len(expected), "CSV row count: " + name)
    for index, (row, wanted) in enumerate(zip(saved, expected)):
        check(None not in row and all(value is not None for value in row.values()), "CSV shape")
        for key, value in wanted.items():
            text = row[key]
            if type(value) is bool:
                check(text == str(value), "CSV boolean")
            elif type(value) is int:
                check(text == str(value), "CSV integer")
            elif type(value) is float:
                try:
                    number = float(text)
                except ValueError as error:
                    raise AuditError("CSV invalid number") from error
                equal(number, value, "CSV " + str(index) + "/" + key)
            else:
                check(text == value, "CSV text/order")


class Attempt:
    def __init__(self, audit, descriptor):
        self.audit = audit
        self.packet = audit.packet
        self.d = shape(descriptor, ("id", "prefix", "outcome", "parent", "candidate", "stages",
                                    "exact_gates", "pilots", "parent_lane", "actual_port_lane",
                                    "diagnostics", "decision", "limitations", "revisions", "guard_proofs"),
                       ("accepted_commit",), "attempt")
        check(type(self.d["id"]) is str and re.fullmatch(r"[0-9]{2,}", self.d["id"]), "attempt ID")
        self.prefix = relative(self.d["prefix"])
        check(self.d["outcome"] in ("accepted", "rejected", "pending"), "attempt outcome")
        check(type(self.d["limitations"]) is list and all(type(v) is str for v in self.d["limitations"]),
              "attempt limitations")
        for key in ("stages", "exact_gates", "pilots", "diagnostics"):
            check(type(self.d[key]) is list, "attempt list: " + key)
        self.candidate = self.d["candidate"]
        self.parent = self.d["parent"]
        check(type(self.d["revisions"]) is dict and "final" not in self.d["revisions"], "revision map")
        for label in self.d["revisions"]:
            relative(label)
        self.exact = {}
        self.stage_results = []

    def name(self, local):
        return self.prefix + "/" + relative(local)

    def obj(self, local):
        return self.packet.obj(self.name(local))

    def artifact(self, local):
        return self.packet.raw(self.name(local))

    def bind_source(self, snapshot, descriptor):
        shape(descriptor, ("snapshot", "snapshot_key", "source_bindings", "binary_sha256",
                           "configured_binary_sha256", "patch", "changed_paths"), context="candidate")
        key = descriptor["snapshot_key"]
        check(type(key) is str and key in snapshot, "candidate snapshot key")
        source = shape(snapshot[key], ("head", "status", "files"), context="source snapshot")
        check(type(source["head"]) is str and re.fullmatch(r"[0-9a-f]{40}", source["head"]) and
              type(source["status"]) is str, "source HEAD/status provenance")
        check(type(source["files"]) is dict, "source file map")
        for path, hash_value in source["files"].items():
            relative(path); sha_value(hash_value)
        equal(snapshot.get("balanced_harness"), self.audit.harness_hashes, "compiled harness map")
        bindings = descriptor["source_bindings"]
        check(type(bindings) is dict and bool(bindings), "source bindings required")
        for path, artifact in bindings.items():
            relative(path)
            check(path in source["files"] and source["files"][path] == self.packet.hash(artifact),
                  "compiled changed source binding: " + path)
        paths = descriptor["changed_paths"]
        check(type(paths) is list and len(set(paths)) == len(paths) and bool(paths), "changed paths")
        for path in paths:
            relative(path)
        patch = self.packet.raw(descriptor["patch"]).decode("utf-8")
        # Git mnemonicPrefix emitted i/w in several preserved patches.
        # Check its original bytes directly; never rewrite an authentic patch.
        headers = re.findall(r"^diff --git ([ai])/(\S+) ([bw])/(\S+)$", patch, flags=re.MULTILINE)
        all_headers = re.findall(r"^diff --git .*$", patch, flags=re.MULTILINE)
        check(bool(headers) and len(headers) == len(all_headers) and
              all((left, right) in (("a", "b"), ("i", "w")) and a == b
                  for left, a, right, b in headers), "unsupported/missing patch file headers")
        equal([a for _, a, _, _ in headers], paths, "patch changed source inventory")
        check(set(paths) <= set(bindings), "every patched source needs copied bytes")
        for field in ("binary_sha256", "configured_binary_sha256"):
            if descriptor[field] is not None:
                sha_value(descriptor[field])
        return source["files"]

    def setup(self):
        if self.d["outcome"] == "pending":
            check(self.candidate is None or type(self.candidate) is dict, "pending candidate")
            if self.candidate is not None:
                snapshot = self.obj(self.candidate["snapshot"])
                self.bind_source(snapshot, self.candidate)
            if self.parent is not None:
                self.check_parent()
            check(not self.d["stages"] and not self.d["exact_gates"] and not self.d["pilots"],
                  "pending descriptor must not claim completed stages/gates/pilots")
            check(not self.d["revisions"] and not self.d["diagnostics"], "pending revisions/diagnostics")
            for lane in ("parent_lane", "actual_port_lane"):
                self.unrun(self.d[lane], pending=True)
            check(self.d["decision"] is None and "accepted_commit" not in self.d, "pending decision/commit")
            return
        check(type(self.candidate) is dict, "final candidate missing")
        for field in ("binary_sha256", "configured_binary_sha256"):
            sha_value(self.candidate[field], "final candidate binary hash")
        self.compiled = self.obj(self.candidate["snapshot"])
        self.source_files = self.bind_source(self.compiled, self.candidate)
        self.check_parent()
        check(self.d["stages"], "final stages missing")
        roles = set()
        folders = set()
        for stage in self.d["stages"]:
            shape(stage, ("role", "folder", "expected_returncode", "expected_receipt", "tests",
                          "revision", "required_log_fragments"), context="stage descriptor")
            check(stage["role"] in ("build", "targeted", "tests", "clippy", "formatting", "layout"), "stage role")
            check(stage["revision"] == "final" or stage["revision"] in self.d["revisions"], "stage revision")
            check(stage["folder"] not in folders, "duplicate stage folder")
            folders.add(stage["folder"])
            if stage["revision"] == "final":
                roles.add(stage["role"])
                equal(stage["expected_returncode"], 0, "final stage must succeed")
            self.stage_results.append(self.stage(stage))
        check("build" in roles, "final build stage required")
        self.historic_failures()
        used_revisions = {s["revision"] for s in self.d["stages"] if s["revision"] != "final"}
        equal(used_revisions, set(self.d["revisions"]), "all failed/static revisions must have a retained stage")
        if self.d["outcome"] == "accepted":
            check({"tests", "clippy", "formatting"} <= roles, "accepted broad checks missing")
            check(type(self.d.get("accepted_commit")) is str and
                  re.fullmatch(r"[0-9a-f]{40}", self.d["accepted_commit"]), "accepted commit")
        else:
            check("accepted_commit" not in self.d, "rejected candidate cannot have accepted commit")
        check({g.get("profile") for g in self.d["exact_gates"]} == set(PROFILE_BITS) and
              len(self.d["exact_gates"]) == 3, "exact three final profiles")
        for gate in self.d["exact_gates"]:
            self.gate(gate)
        check(self.d["pilots"], "final frozen pilot required")
        if self.d["id"] == "10":
            equal({p.get("folder") for p in self.d["pilots"]}, {"pilot-first-load-heavy", "pilot-final"},
                  "both attempt10 pilot runs retained")

    def historic_failures(self):
        failed = [s for s in self.d["stages"] if s["revision"] != "final" and s["expected_returncode"] != 0]
        qid = self.d["id"]
        if qid in ("08", "09", "10", "12", "16", "19"):
            check(bool(failed), "historic expected failure may not be omitted")
            check(all(s["expected_returncode"] == 101 for s in failed), "historic Cargo failure status")
        if qid == "08":
            check(len(failed) >= 2 and any("duplicate-unfixed-tests" in s["folder"] for s in failed), "both unfixed08 runs retained")
            check(all(any("E0689" in v for v in s["required_log_fragments"].get("stderr", [])) for s in failed), "08 compilation signature")
        if qid == "10":
            check(any(any("E0277" in v for v in s["required_log_fragments"].get("stderr", [])) for s in failed), "10 collector trait signature")
        if qid == "16":
            check(any(s["revision"] == "pre-capfixture-fix" and
                      s["folder"] == "pre-capfixture-fix/tests-targeted" for s in failed),
                  "16 original capped-geometry fixture failure retained")
        if qid == "19":
            check(any(s["revision"] == "pre-initial-seek-fix" and
                      s["folder"] == "pre-initial-seek-fix/tests-targeted" for s in failed),
                  "19 original forward-seek fixture failure retained")
            check(any("assertion failed: target >= self.doc()" in
                      s["required_log_fragments"].get("stderr", []) for s in failed),
                  "19 original forward-seek assertion signature retained")
        for attempt, passed, count in (("09", 17, 3), ("12", 21, 2), ("16", 28, 1), ("19", 30, 2)):
            if qid == attempt:
                check(any(type(s["tests"]) is dict and s["tests"].get("passed") == passed and
                          s["tests"].get("failed") == count for s in failed), "retained historic failed test totals")

    def check_parent(self):
        p = shape(self.parent, ("kind", "binary_sha256", "source_commit", "descriptor",
                               "compiled_snapshot", "exact_receipt"), ("attempt_id",), "parent proof")
        sha_value(p["binary_sha256"])
        check(p["kind"] in ("baseline", "accepted"), "parent kind")
        check(type(p["source_commit"]) is str and re.fullmatch(r"[0-9a-f]{40}", p["source_commit"]),
              "parent commit")
        if p["kind"] == "baseline":
            check("attempt_id" not in p and p["descriptor"] is None, "baseline descriptor")
            equal(p["binary_sha256"], self.audit.common["baseline"]["binary_sha256"], "baseline binary")
            equal(p["source_commit"], self.audit.common["baseline"]["source_commit"], "baseline commit")
            for field in ("compiled_snapshot", "exact_receipt"):
                equal(self.packet.hash(p[field]), self.packet.hash(self.audit.common["baseline"][field]),
                      "baseline relocated proof identity")
        else:
            check(p.get("attempt_id") in self.audit.descriptors, "unknown accepted parent")
            previous = self.audit.descriptors[p["attempt_id"]]
            check(previous["outcome"] == "accepted", "parent is not accepted")
            equal(previous["accepted_commit"], p["source_commit"], "parent chain commit")
            equal(previous["candidate"]["binary_sha256"], p["binary_sha256"], "parent chain binary")
            original = self.packet.obj(p["descriptor"])
            equal(original["binary_sha256"], p["binary_sha256"], "parent descriptor binary")
            equal(original["source_commit"], p["source_commit"], "parent descriptor commit")
            equal(original["compiled_sources_sha256"], self.packet.hash(p["compiled_snapshot"]),
                  "relocated parent compiled proof")
            equal(original["exact_gate_sha256"], self.packet.hash(p["exact_receipt"]),
                  "relocated parent gate proof")
            equal(self.packet.hash(p["compiled_snapshot"]),
                  self.packet.hash(previous["prefix"] + "/" + previous["candidate"]["snapshot"]),
                  "parent compiled proof is the accepted attempt proof")
            default_gate = next(g for g in previous["exact_gates"] if g["profile"] == "default")
            equal(self.packet.hash(p["exact_receipt"]),
                  self.packet.hash(previous["prefix"] + "/" + default_gate["folder"] + "/receipt.json"),
                  "parent gate proof is the accepted attempt gate")
        receipt = self.packet.obj(p["exact_receipt"])
        check(receipt.get("pass") is True and receipt.get("queries") == 1227 and
              receipt.get("failures") == 0, "parent exact receipt")
        self.packet.obj(p["compiled_snapshot"])

    def source_guard(self, guard):
        check(type(guard) is dict and type(guard.get("source")) is dict, "guard/source")
        source = guard["source"]
        equal(source, self.compiled, "guard full compiled source snapshot")
        key = self.candidate["snapshot_key"]
        check(key in source and source[key]["files"] == self.source_files, "guard final compiled files")
        self.audit.proof_fields(guard, self.audit.common["guard_proofs"])
        self.audit.proof_fields(guard, self.d["guard_proofs"])
        check(["controller"] in [v["selector"] for v in self.d["guard_proofs"]], "retained balanced controller hash binding required")
        check(guard.get("queries") == self.packet.hash(self.audit.common["queries"]), "guard queries")
        equal(guard.get("harness"), self.audit.harness_hashes, "guard harness")

    def guard(self, folder, profile="default", actual_port=False):
        before = self.obj(folder + "/before.json")
        after = self.obj(folder + "/after.json")
        equal(after, before, "guard before/after " + folder)
        self.source_guard(before)
        binary = (self.candidate["binary_sha256"] if profile == "default"
                  else self.candidate["configured_binary_sha256"])
        expected_port = self.audit.common["fixed_port"]["binary_sha256"] if actual_port else self.parent["binary_sha256"]
        equal(before.get("binaries"), {"tantivy": binary, "port": expected_port}, "guard binaries")
        maps = {"tantivy": self.audit.index_maps["tantivy"],
                "port": self.audit.index_maps["actual_port" if actual_port else "tantivy"]}
        equal(before.get("indices"), maps, "guard lane index semantics")
        if not actual_port and self.parent["kind"] == "accepted":
            equal(before.get("comparison_controller_sha256"), self.packet.hash(self.audit.common["comparison_controller"]),
                  "parent-selection controller SHA")
            selected = before.get("selected_parent")
            check(type(selected) is dict, "selected parent guard missing")
            equal(selected.get("descriptor_sha256"), self.packet.hash(self.parent["descriptor"]), "selected parent SHA")
            equal(selected.get("source_commit"), self.parent["source_commit"], "selected parent commit")
            equal(selected.get("compiled_sources_sha256"), self.packet.hash(self.parent["compiled_snapshot"]),
                  "selected compiled snapshot SHA")
            equal(selected.get("exact_gate_sha256"), self.packet.hash(self.parent["exact_receipt"]), "selected exact SHA")
            # Absolute original paths are strings of provenance, never resolved.
            check(type(selected.get("descriptor")) is str, "original descriptor path provenance")
        return before

    def stage(self, stage):
        folder = stage["folder"]
        before = self.obj(folder + "/before.json")
        after = self.obj(folder + "/after.json")
        equal(before, after, "stage unchanged " + folder)
        candidate = self.candidate if stage["revision"] == "final" else self.d["revisions"][stage["revision"]]
        check(candidate is not None, "failed revision candidate bindings required")
        files = self.bind_source(before, candidate)
        if stage["revision"] == "final":
            equal(files, self.source_files, "stage final compiled source map")
        equal(before, self.obj(candidate["snapshot"]), "stage exact revision snapshot")
        receipt = self.obj(folder + "/receipt.json")
        check(type(stage["expected_returncode"]) is int, "expected status integer")
        equal(receipt.get("returncode"), stage["expected_returncode"], "stage expected status")
        check(receipt.get("source_unchanged") is True and receipt.get("audited_index_unchanged") is True,
              "stage source/index receipts")
        check(type(stage["expected_receipt"]) is dict and
              {"compiler", "rustflags", "jobs", "profile", "command"} <= set(stage["expected_receipt"]),
              "explicit stage environment/command required")
        for field, wanted in stage["expected_receipt"].items():
            equal(receipt.get(field), wanted, "stage receipt/" + field)
        for field, value in {"compiler": "1.99.0", "rustflags": "-C target-cpu=native", "jobs": 2}.items():
            equal(receipt.get(field), value, "frozen preparation environment/" + field)
        equal(self.artifact(folder + "/compiler.txt").decode("utf-8"), COMPILER_TEXT,
              "full frozen compiler identity/LLVM")
        equal(self.obj(folder + "/command.json"), receipt.get("command"), "retained stage command")
        self.audit.duration(receipt.get("wall_seconds"), "stage wall seconds")
        logs = {}
        for stream in ("stdout", "stderr"):
            logs[stream] = self.artifact(folder + "/" + stream + ".log")
            equal(digest(logs[stream]), receipt.get(stream + "_sha256"), "stage log SHA")
        fragments = stage["required_log_fragments"]
        check(type(fragments) is dict and set(fragments) <= {"stdout", "stderr"}, "log fragments")
        for stream, values in fragments.items():
            check(type(values) is list and all(type(v) is str and v for v in values), "log fragment array")
            for fragment in values:
                check(fragment.encode() in logs[stream], "required retained failure/log signature")
        totals = None
        if stage["tests"] is not None:
            shape(stage["tests"], ("passed", "failed", "ignored", "measured", "filtered_out"), context="test totals")
            matches = TEST_RE.findall(logs["stdout"].decode("utf-8"))
            check(bool(matches), "test totals not present")
            totals = dict(zip(("passed", "failed", "ignored", "measured", "filtered_out"),
                              map(sum, zip(*(tuple(map(int, row)) for row in matches)))))
            equal(totals, stage["tests"], "retained test totals")
        if stage["role"] == "tests" and stage["revision"] == "final":
            check(totals is not None and totals["failed"] == 0, "broad test totals required")
            known = {"01": 1434, "03": 1438, "06": 1440, "11": 1442, "13": 1457, "14": 1445,
                     "16": 1463, "18": 1443, "21": 1442, "22": 1446}
            if self.d["id"] in known:
                equal(totals["passed"], known[self.d["id"]], "recorded broad test count")
                equal(totals["ignored"], 7, "recorded ignored tests")
        if stage["expected_returncode"] != 0:
            check(bool(fragments) and any(values for values in fragments.values()), "failed stage must retain a failure signature")
        return {"role": stage["role"], "revision": stage["revision"],
                "expected_returncode": stage["expected_returncode"], "test_totals": totals}

    def gate(self, gate):
        shape(gate, ("profile", "folder", "protocol"), context="exact gate descriptor")
        profile, folder = gate["profile"], gate["folder"]
        self.guard(folder, profile)
        receipt = self.obj(folder + "/receipt.json")
        for key, value in {"pass": True, "queries": 1227, "failures": 0, "unchanged": True}.items():
            equal(receipt.get(key), value, "exact gate receipt")
        equal(self.obj(folder + "/failures.json"), [], "exact gate failures")
        rows = self.obj(folder + "/dumps.json")
        check(type(rows) is list and len(rows) == 1227, "exact dump count")
        protocol = Protocol(self.packet, self.name(gate["protocol"]), profile)
        if profile != "default":
            equal(receipt.get("ready"), protocol.ready, "configured READY receipt")
        expected_rows = iter(self.audit.cases)
        verified = {}
        for row in rows:
            case = next(expected_rows)
            expected_keys = ("case", "tantivy", "port") if profile == "default" else ("case", "tantivy")
            shape(row, expected_keys, context="exact dump row")
            equal(row["case"], case, "exact full case/order")
            actual = row["tantivy"]
            value = validate_dump(case, actual)
            check("oracle" in actual, "exhaustive oracle missing")
            equal(value, validate_dump(case, actual["oracle"], True), "optimized/exhaustive raw bits")
            response = protocol.exchange({"op": "dump", "id": case["id"]})
            equal(response, actual, "exact dump protocol bytes")
            if profile == "default":
                equal(value, self.audit.frozen[case["id"]]["tantivy"], "frozen initial exact reference")
                equal(value, self.audit.frozen[case["id"]]["port"], "frozen actual port exact reference")
                equal(validate_dump(case, row["port"]), value, "parent-labelled retained reference")
            verified[case["id"]] = value
        protocol.end()
        check(set(verified) == set(self.audit.queries), "exact unique IDs")
        self.exact[profile] = verified

    @staticmethod
    def unrun(lane, pending=False):
        shape(lane, ("status", "reason"), context="unrun lane")
        check(lane["status"] == ("pending" if pending else "not_run") and
              type(lane["reason"]) is str and bool(lane["reason"]), "explicit unrun lane state")

    def lane(self, lane, actual_port=False):
        if lane.get("status") != "retained":
            self.unrun(lane)
            check(self.d["outcome"] != "accepted", "accepted attempt requires both complete lanes")
            return {"status": "not_run", "reason": lane["reason"]}
        shape(lane, ("status", "folder", "csv", "summary", "protocols", "summary_hash_bindings",
                     "metrics", "metrics_format", "metrics_metadata", "metric_hash_bindings", "port_group_audit",
                     "port_group_controller"), context="timing lane")
        folder = lane["folder"]
        self.guard(folder, actual_port=actual_port)
        schedule = self.obj(folder + "/schedule.json")
        pairs = [["tantivy", "port"], ["port", "port"]] if actual_port else [
            ["tantivy", "port"], ["tantivy", "tantivy"], ["port", "port"]]
        modes = ["top10"] if actual_port else ["count", "top10"]
        wanted = {"pairs": pairs, "rounds": 3, "samples_per_order": 16, "queries": self.audit.timed,
                  "cpu": 4, "warmup": 10}
        for field, value in wanted.items():
            equal(schedule.get(field), value, "frozen schedule/" + field)
        if actual_port:
            equal(schedule.get("modes"), modes, "actual-port modes")
            equal(schedule.get("seed"), 20261006, "actual-port seed")
            equal(schedule.get("labels"), {"tantivy": "candidate Tantivy", "port": "fixed lucene-rs"},
                  "actual-port labels")
        else:
            equal(schedule.get("query_order_seed"), 20261006, "parent-lane seed")
            check("port engine label means preserved baseline Tantivy" in schedule.get("primary", ""),
                  "explicit parent 'port' semantic label")
        receipt = self.obj(folder + "/receipt.json")
        check(receipt.get("pass") is True, "timing receipt pass")
        unchanged_field = "unchanged" if actual_port else "source_index_binary_unchanged"
        check(receipt.get(unchanged_field) is True, "timing unchanged receipt")
        raw_name = self.name(folder + "/samples.jsonl")
        if not actual_port:
            equal(receipt.get("samples_sha256"), self.packet.hash(raw_name), "timing receipt raw hash")
        protocols = lane["protocols"]
        check(type(protocols) is dict and set(protocols) == {str(i) for i in range(len(pairs))},
              "exact protocol pair descriptors")
        raw = iter(self.packet.lines(raw_name))
        cells = collections.defaultdict(list)
        round_cells = {}
        seen = set()
        for pair, engines in enumerate(pairs):
            shape(protocols[str(pair)], ("A", "B"), context="pair protocol mapping")
            readers = {}
            for side, engine in zip("AB", engines):
                readers[side] = Protocol(self.packet, self.name(protocols[str(pair)][side]),
                                         actual_port=actual_port and engine == "port")
                exact = self.audit.frozen_port if actual_port and engine == "port" else self.exact["default"]
                for qid in self.audit.timed:
                    for mode in modes:
                        response = readers[side].exchange({"op": "run", "id": qid, "mode": mode, "iterations": 10})
                        validate_run(response, qid, mode, 10, exact[qid])
            for rnd in range(3):
                shuffled = self.audit.timed.copy()
                random.Random(20261006 + rnd).shuffle(shuffled)
                for order in ORDERS:
                    for qid in shuffled:
                        for mode in modes:
                            for side in order:
                                row = next(raw, None)
                                shape(row, ("pair", "round", "order", "id", "mode", "side", "engine", "checksum", "ns"),
                                      context="timing raw row")
                                engine = engines[0 if side == "A" else 1]
                                key = (pair, rnd, order, qid, mode, side)
                                equal([row["pair"], row["round"], row["order"], row["id"], row["mode"], row["side"]],
                                      list(key), "raw complete schedule/seed/row order")
                                check(row["engine"] == engine and key not in seen, "timing engine/duplicate cell")
                                seen.add(key)
                                exact = self.audit.frozen_port if actual_port and engine == "port" else self.exact["default"]
                                ns = validate_run(row, qid, mode, 16, exact[qid])
                                response = readers[side].exchange({"op": "run", "id": qid, "mode": mode, "iterations": 16})
                                equal(response, {k: row[k] for k in ("checksum", "id", "mode", "ns")},
                                      "raw/protocol timing consistency")
                                cells[pair, order, qid, mode, side].extend(ns)
                                round_cells[key] = ns
            for reader in readers.values():
                reader.end()
        check(next(raw, None) is None, "extra timing rows")
        expected_count = 29304 if actual_port else 87912
        check(len(seen) == expected_count, "complete timing cell count")
        # Computed medians retain every query, mode, order and round.
        med = {key: statistics.median(values) for key, values in cells.items()}
        rndmed = {key: statistics.median(values) for key, values in round_cells.items()}
        if actual_port:
            rows, summaries, groups = self.port_stats(med)
        else:
            rows, summaries, groups = self.parent_stats(med, rndmed)
        csv_check(self.packet, self.name(lane["csv"]), rows)
        saved_summary = self.obj(lane["summary"])
        equal(saved_summary.get("summaries"), summaries, "all timing summaries")
        summary_expected = self.summary_envelope(lane, receipt, saved_summary, expected_count, actual_port)
        equal(saved_summary, {**summary_expected, "summaries": summaries}, "full summary envelope")
        losses = [row for row in rows if (row.get("P_wins_beyond_controls") is True if not actual_port else
                  all(math.log(row["candidate_over_port_" + o]) >
                      max(abs(math.log(row["port_control_" + c])) for c in ORDERS) for o in ORDERS))]
        if actual_port:
            check(lane["metrics"] is None and lane["metrics_format"] is None and
                  lane["metrics_metadata"] == {} and lane["metric_hash_bindings"] == [],
                  "actual-port lane has no parent metrics")
            target = all(groups[k]["candidate_win_both_beyond_port_self_controls"] for k in ("OR", "OR_3PLUS"))
            if lane["port_group_audit"] is not None:
                expected = {"groups": groups, "actual_port_target_met": target,
                            "controls_scope": "Contemporaneous fixed-port self comparisons only. Candidate self comparisons are retained in the separate parent-comparison lane.",
                            "samples_sha256": self.packet.hash(raw_name),
                            "controller_sha256": self.packet.hash(lane["port_group_controller"])}
                equal(self.obj(lane["port_group_audit"]), expected, "arithmetic actual-port audit")
            else:
                check(lane["port_group_controller"] is None, "absent port audit/controller pairing")
        else:
            check(lane["port_group_audit"] is None and lane["port_group_controller"] is None,
                  "parent lane cannot carry actual-port audit")
            self.metrics(lane, groups, losses)
            target = None
        return {"status": "retained", "semantics": "actual-lucene-rs" if actual_port else "preserved-parent-tantivy",
                "cells": expected_count, "samples": expected_count * 16, "groups": groups,
                "per_query": rows, "summaries": summaries,
                "per_query_losses_beyond_observed_controls": losses, "actual_port_target_met": target,
                "controls_scope": "fixed-port self only" if actual_port else "candidate and parent self"}

    def summary_envelope(self, lane, receipt, summary, count, actual_port):
        folder = lane["folder"]
        rss = self.obj(folder + "/rss.json")
        telemetry = self.obj(folder + "/telemetry.json")
        check(type(rss) is list and len(rss) == (2 if actual_port else 3), "RSS pair inventory")
        check(type(telemetry) is list and len(telemetry) == (14 if actual_port else 21), "telemetry schedule inventory")
        self.audit.telemetry(telemetry, 14 if actual_port else 21)
        for pair, entry in enumerate(rss):
            shape(entry, ("pair", "A", "B"), context="retained RSS pair")
            equal(entry["pair"], pair, "RSS pair order")
            for side in "AB":
                shape(entry[side], ("VmRSS", "VmHWM", "VmSize"), context="retained worker RSS")
                check(all(type(v) is str and re.fullmatch(r"[0-9]+ kB", v) for v in entry[side].values()), "RSS units/value")
        if actual_port:
            check(lane["summary_hash_bindings"] == {}, "actual-port summary hash fields fixed")
            return {"pass": True, "cells": count, "samples": count * 16,
                    "ratio": "candidate Tantivy / fixed lucene-rs; below1 candidate faster", "rss": rss,
                    "samples_sha256": self.packet.hash(self.name(folder + "/samples.jsonl")),
                    "per_query_sha256": self.packet.hash(self.name(lane["csv"]))}
        hashes = lane["summary_hash_bindings"]
        check(type(hashes) is dict and len(hashes) == 5 and set(hashes.values()) == {
            self.name(folder + "/samples.jsonl"), self.name(lane["csv"]), self.name(folder + "/schedule.json"),
            self.name(folder + "/telemetry.json"), self.audit.common["queries"]}, "all main summary hash bindings")
        return {"protocol": "or-improvement-native-api-v1",
                "ratio": "candidate / preserved baseline Tantivy; below1 means candidate faster. port/P labels are baseline Tantivy for script compatibility",
                "complete_cells": count, "raw_samples": count * 16, "queries_timed": 1221,
                "correctness_queries": 1227, "full_postings_and_positions_identity": True,
                "no_trim_no_cherry_pick": True, "rss": rss, "wall_seconds": receipt["wall_seconds"],
                "source_binary_index_guard": True,
                "files_sha256": {key: self.packet.hash(artifact) for key, artifact in hashes.items()}}

    def parent_stats(self, med, rndmed):
        rows = []
        for qid in self.audit.timed:
            q = self.audit.queries[qid]
            for mode in ("count", "top10"):
                row = {"id": qid, "kind": q["kind"], "terms": " ".join(q["terms"]),
                       "set": "wiki20" if "wiki20" in q["tags"] else "port-style", "mode": mode,
                       "exact_count": self.exact["default"][qid]["count"]}
                for order in ORDERS:
                    for side, engine in (("A", "tantivy"), ("B", "port")):
                        row[engine + "_" + order + "_ns"] = med[0, order, qid, mode, side]
                    row["T_over_P_" + order] = med[0, order, qid, mode, "A"] / med[0, order, qid, mode, "B"]
                    for pair, label in ((1, "TT"), (2, "PP")):
                        row[label + "_" + order] = med[pair, order, qid, mode, "A"] / med[pair, order, qid, mode, "B"]
                noise = max(abs(math.log(row[label + "_" + order])) for label in ("TT", "PP") for order in ORDERS)
                row["control_log_spread"] = noise
                row["raw_T_wins_both"] = all(row["T_over_P_" + o] < 1 for o in ORDERS)
                row["raw_P_wins_both"] = all(row["T_over_P_" + o] > 1 for o in ORDERS)
                row["T_wins_beyond_controls"] = all(math.log(row["T_over_P_" + o]) < -noise for o in ORDERS)
                row["P_wins_beyond_controls"] = all(math.log(row["T_over_P_" + o]) > noise for o in ORDERS)
                rows.append(row)
        summaries = []
        for suite in ("wiki20", "port-style"):
            for mode in ("count", "top10"):
                for kind in KINDS:
                    rs = [r for r in rows if r["mode"] == mode and group_match(self.audit.queries[r["id"]], suite, kind)]
                    if not rs:
                        continue
                    s = {"set": suite, "mode": mode, "kind": kind, "queries": len(rs),
                         "nonzero_queries": sum(r["exact_count"] > 0 for r in rs),
                         "T_wins_both_orders": sum(r["raw_T_wins_both"] for r in rs),
                         "P_wins_both_orders": sum(r["raw_P_wins_both"] for r in rs),
                         "T_wins_beyond_observed_controls": sum(r["T_wins_beyond_controls"] for r in rs),
                         "P_wins_beyond_observed_controls": sum(r["P_wins_beyond_controls"] for r in rs)}
                    for o in ORDERS:
                        t = statistics.mean(r["tantivy_" + o + "_ns"] for r in rs)
                        p = statistics.mean(r["port_" + o + "_ns"] for r in rs)
                        s[o] = {"geomean_T_over_P": geo([r["T_over_P_" + o] for r in rs]),
                                "mean_query_median_T_ns": t, "mean_query_median_P_ns": p,
                                "ratio_of_mean_query_medians": t / p,
                                "controls": {label: geo([r[label + "_" + o] for r in rs]) for label in ("TT", "PP")}}
                    if kind == "ALL":
                        s["per_round"] = []
                        for rnd in range(3):
                            for o in ORDERS:
                                def ratios(pair):
                                    return [rndmed[pair, rnd, o, r["id"], mode, "A"] /
                                            rndmed[pair, rnd, o, r["id"], mode, "B"] for r in rs]
                                s["per_round"].append({"round": rnd, "order": o, "geomean_T_over_P": geo(ratios(0)),
                                                       "controls": {label: geo(ratios(pair)) for pair, label in ((1, "TT"), (2, "PP"))}})
                    summaries.append(s)
        groups = {}
        for kind in ("ALL", "OR", "OR_2", "OR_3PLUS", "AND", "TERM", "PHRASE"):
            ids = [qid for qid in self.audit.timed if group_match(self.audit.queries[qid], "port-style", kind)]
            def ratio(pair, o, rnd=None):
                if rnd is None:
                    value = lambda qid, side: med[pair, o, qid, "top10", side]
                else:
                    value = lambda qid, side: rndmed[pair, rnd, o, qid, "top10", side]
                return sum(value(qid, "A") for qid in ids) / sum(value(qid, "B") for qid in ids)
            primary = {o: ratio(0, o) for o in ORDERS}
            controls = {label: {o: ratio(pair, o) for o in ORDERS} for pair, label in ((1, "TT"), (2, "PP"))}
            noise = max(abs(math.log(v)) for values in controls.values() for v in values.values())
            groups[kind] = {"queries": len(ids), "primary": primary, "controls": controls,
                            "control_log_spread": noise,
                            "candidate_win_both_beyond_group_controls": all(math.log(v) < -noise for v in primary.values()),
                            "parent_win_both_beyond_group_controls": all(math.log(v) > noise for v in primary.values()),
                            "rounds": [{"round": rnd, "order": o, "primary": ratio(0, o, rnd),
                                        "TT": ratio(1, o, rnd), "PP": ratio(2, o, rnd)} for rnd in range(3) for o in ORDERS]}
        return rows, summaries, groups

    def port_stats(self, med):
        rows = []
        for qid in self.audit.timed:
            q = self.audit.queries[qid]
            r = {"id": qid, "set": "wiki20" if "wiki20" in q["tags"] else "port-style",
                 "kind": q["kind"], "terms": " ".join(q["terms"])}
            for o in ORDERS:
                r["candidate_" + o + "_ns"] = med[0, o, qid, "top10", "A"]
                r["port_" + o + "_ns"] = med[0, o, qid, "top10", "B"]
                r["candidate_over_port_" + o] = r["candidate_" + o + "_ns"] / r["port_" + o + "_ns"]
                r["port_control_" + o] = med[1, o, qid, "top10", "A"] / med[1, o, qid, "top10", "B"]
            rows.append(r)
        summaries, groups = [], {}
        for suite in ("wiki20", "port-style"):
            for kind in KINDS:
                rs = [r for r in rows if group_match(self.audit.queries[r["id"]], suite, kind)]
                if not rs:
                    continue
                s = {"set": suite, "kind": kind, "queries": len(rs)}
                primary, controls = {}, {}
                for o in ORDERS:
                    cm = statistics.mean(r["candidate_" + o + "_ns"] for r in rs)
                    pm = statistics.mean(r["port_" + o + "_ns"] for r in rs)
                    s[o] = {"candidate_mean_query_median_ns": cm, "port_mean_query_median_ns": pm,
                            "ratio_of_mean_query_medians_candidate_over_port": cm / pm,
                            "geomean_candidate_over_port": geo([r["candidate_over_port_" + o] for r in rs]),
                            "port_control_geomean": geo([r["port_control_" + o] for r in rs])}
                    ids = [r["id"] for r in rs]
                    primary[o] = sum(med[0, o, qid, "top10", "A"] for qid in ids) / sum(med[0, o, qid, "top10", "B"] for qid in ids)
                    controls[o] = sum(med[1, o, qid, "top10", "A"] for qid in ids) / sum(med[1, o, qid, "top10", "B"] for qid in ids)
                summaries.append(s)
                if suite == "port-style":
                    noise = max(abs(math.log(v)) for v in controls.values())
                    groups[kind] = {"queries": len(rs), "candidate_over_port": primary,
                                    "arithmetic_port_self_control": controls, "control_log_spread": noise,
                                    "candidate_win_both_beyond_port_self_controls": all(math.log(v) < -noise for v in primary.values())}
        return rows, summaries, groups

    def metrics(self, lane, groups, losses):
        check(lane["metrics"] is not None, "retained parent lane metrics required")
        saved = self.obj(lane["metrics"])
        check(lane["metrics_format"] in ("legacy-arithmetic", "group-controls"), "metrics schema variant")
        saved_groups = groups
        if lane["metrics_format"] == "legacy-arithmetic":
            saved_groups = {kind: {key: value for key, value in group.items()
                                  if key not in ("control_log_spread", "candidate_win_both_beyond_group_controls",
                                                 "parent_win_both_beyond_group_controls")}
                            for kind, group in groups.items()}
        metadata = lane["metrics_metadata"]
        check(type(metadata) is dict and set(metadata) <= {"efficiency", "profile_overwrite_guard"}, "metrics metadata keys")
        expected = {**metadata, "groups": saved_groups,
                    "regressions_beyond_per_query_controls": [
                        {key: str(row[key]) for key in ("id", "kind", "terms", "T_over_P_AB", "T_over_P_BA", "control_log_spread")}
                        for row in losses if row["mode"] == "top10"]}
        if self.parent["kind"] == "accepted":
            expected["parent"] = self.packet.obj(self.parent["descriptor"])
        equal(saved, expected, "all parent metric statistics and declared receipt-only metadata")
        self.audit.proof_fields(saved, lane["metric_hash_bindings"])
        if "efficiency" in metadata:
            efficiency = metadata["efficiency"]
            equal(efficiency.get("candidate_binary_sha256"), self.candidate["binary_sha256"], "efficiency candidate SHA")
            equal(efficiency.get("baseline_binary_sha256"), self.parent["binary_sha256"], "efficiency parent SHA")
            check(efficiency.get("index_change") is False, "unchanged index claim")
        if "profile_overwrite_guard" in metadata:
            g = metadata["profile_overwrite_guard"]
            check(g.get("pass") is True and g.get("returncode") != 0, "overwrite refusal result")
            equal(g.get("before_sha256"), g.get("after_sha256"), "overwrite refusal preserved outputs")

    def pilot(self, pilot):
        shape(pilot, ("folder", "protocols", "controller", "schedule", "limitations"), context="pilot")
        folder = pilot["folder"]
        self.guard(folder)
        work = [q for q in self.audit.cases if q["id"] in self.audit.timed and
                (q["kind"] == "OR" or "wiki20" in q["tags"] or q["id"] % 17 == 0)]
        schedule = {"queries": [q["id"] for q in work], "warmup": 4, "samples": 12,
                    "orders": list(ORDERS), "seed": 20261006, "mode": "top10", "cpu": 4}
        equal(pilot["schedule"], schedule, "explicit frozen pilot schedule")
        equal(self.packet.hash(pilot["controller"]), self.audit.common["pilot_controller_sha256"],
              "frozen pilot controller")
        shape(pilot["protocols"], ("A", "B"), context="pilot protocols")
        readers = {side: Protocol(self.packet, self.name(pilot["protocols"][side])) for side in "AB"}
        for side in "AB":
            for q in work:
                response = readers[side].exchange({"op": "run", "id": q["id"], "mode": "top10", "iterations": 4})
                validate_run(response, q["id"], "top10", 4, self.exact["default"][q["id"]])
        rows = self.obj(folder + "/samples.json")
        check(type(rows) is list and len(rows) == 1492, "pilot full cell inventory")
        raw, seen, medians = iter(rows), set(), {}
        for order in ORDERS:
            shuffled = work.copy()
            random.Random(20261006).shuffle(shuffled)
            for q in shuffled:
                for side in order:
                    row = next(raw, None)
                    shape(row, ("order", "side", "engine", "id", "mode", "checksum", "ns"), context="pilot row")
                    key = (order, q["id"], side)
                    equal([row["order"], row["id"], row["side"]], list(key), "pilot full row order")
                    equal(row["engine"], "tantivy" if side == "A" else "port", "pilot engine")
                    check(key not in seen, "duplicate pilot cell")
                    seen.add(key)
                    ns = validate_run(row, q["id"], "top10", 12, self.exact["default"][q["id"]])
                    response = readers[side].exchange({"op": "run", "id": q["id"], "mode": "top10", "iterations": 12})
                    equal(response, {k: row[k] for k in ("id", "mode", "checksum", "ns")}, "pilot/protocol match")
                    medians[key] = statistics.median(ns)
        for reader in readers.values():
            reader.end()
        check(next(raw, None) is None and len(seen) == 1492, "complete pilot cells")
        ratios = [{**q, **{o: medians[o, q["id"], "A"] / medians[o, q["id"], "B"] for o in ORDERS}}
                  for q in work]
        summaries = []
        for label in ("all", "or3plus", "or2"):
            rs = [r for r in ratios if label == "all" or r["kind"] == "OR" and
                  (len(r["terms"]) >= 3 if label == "or3plus" else len(r["terms"]) == 2)]
            summary = {"set": label, "queries": len(rs)}
            for order in ORDERS:
                a = [medians[order, q["id"], "A"] for q in rs]
                b = [medians[order, q["id"], "B"] for q in rs]
                summary[order] = {"candidate_mean_ns": statistics.mean(a), "baseline_mean_ns": statistics.mean(b),
                                  "ratio_mean": sum(a) / sum(b),
                                  "median_ratio": statistics.median(q[order] for q in rs)}
            summaries.append(summary)
        receipt = self.obj(folder + "/receipt.json")
        shape(receipt, ("seconds", "unchanged", "summary", "worst_regressions", "loads"), context="pilot receipt")
        equal(receipt["unchanged"], True, "pilot unchanged")
        self.audit.duration(receipt["seconds"], "pilot wall seconds")
        equal(receipt["summary"], summaries, "all pilot aggregates")
        worst = sorted(ratios, key=lambda q: min(q["AB"], q["BA"]), reverse=True)[:20]
        equal(receipt["worst_regressions"], worst, "pilot worst list, with all rows retained")
        self.audit.telemetry(receipt["loads"], 2)
        check(type(pilot["limitations"]) is list and all(type(v) is str for v in pilot["limitations"]), "pilot limitations")
        return {"folder": folder, "cells": 1492, "samples": 17904, "summary": summaries,
                "per_query": ratios, "worst_regressions": worst, "limitations": pilot["limitations"],
                "controls": "none; pilot alone is not a control-adjusted full-suite result"}

    def diagnostic(self, record):
        check(type(record) is dict, "diagnostic object")
        if record.get("kind") == "receipt-only":
            shape(record, ("kind", "artifact", "bindings", "limitations"), context="receipt-only diagnostic")
            self.audit.proof_fields(self.packet.obj(record["artifact"]), record["bindings"])
            check(type(record["limitations"]) is list and bool(record["limitations"]), "receipt-only limits")
            return record
        if record.get("status") == "overwritten_unavailable":
            shape(record, ("kind", "status", "role", "reason", "correction"), context="unavailable profile")
            check(self.d["id"] == "01" and record["kind"] == "profile" and record["role"] == "original-baseline",
                  "only the recorded original baseline is unavailable")
            check(type(record["reason"]) is str and bool(record["reason"]), "unavailable reason")
            correction = self.obj(record["correction"])
            equal(correction.get("latency_results_affected"), False, "profile correction separate from latency")
            check("overwrote" in correction.get("mistake", "") or "replaced" in correction.get("mistake", ""),
                  "retained overwritten-profile explanation")
            return record
        shape(record, ("kind", "status", "role", "receipt", "receipt_format", "replies", "controller", "cases",
                       "binary_sha256", "trace", "report", "stderr", "limitations"), context="retained profile")
        check(record["kind"] == "profile" and record["status"] in
              ("retained_excluded_own_clippy", "retained_recreated", "retained_diagnostic"), "profile state")
        check(record["role"] in ("baseline", "candidate"), "profile binary role")
        expected_binary = self.parent["binary_sha256"] if record["role"] == "baseline" else self.candidate["binary_sha256"]
        equal(record["binary_sha256"], expected_binary, "profile binary role identity")
        receipt = self.obj(record["receipt"])
        for key, value in {"returncode": 0, "binary_sha256": expected_binary,
                           "correctness_checksums_match": True, "first_preparation_and_warmup_included_in_trace": True}.items():
            equal(receipt.get(key), value, "profile receipt/" + key)
        check(record["receipt_format"] in ("legacy", "guarded"), "profile receipt format")
        if record["receipt_format"] == "guarded":
            equal(receipt.get("binary_unchanged"), True, "guarded profile binary unchanged")
            equal(receipt.get("script_sha256"), self.packet.hash(record["controller"]), "profile script hash")
            equal(receipt.get("cases_sha256"), self.packet.hash(record["cases"]), "profile cases hash")
            if "query_sha256" in receipt or "inputs_unchanged" in receipt:
                equal(receipt.get("query_sha256"), self.packet.hash(self.audit.common["queries"]),
                      "profile frozen query hash")
                equal(receipt.get("inputs_unchanged"), True, "profile query/case inputs unchanged")
                equal(receipt.get("latency_acceptance"), False, "profile is diagnostic only")
        else:
            check(record["status"] == "retained_excluded_own_clippy" and self.d["id"] == "01", "legacy profile scope")
            check(not {"binary_unchanged", "script_sha256", "cases_sha256"} & set(receipt), "legacy missing guards disclosed")
            self.packet.hash(record["controller"]); self.packet.hash(record["cases"])
        cases = self.packet.obj(record["cases"])
        equal(receipt.get("cases"), cases, "profile retained cases")
        check(type(cases) is list and bool(cases), "profile cases")
        seen = set()
        for case in cases:
            check(type(case) is dict, "profile case object")
            qid = integer(case["id"])
            check(qid in self.audit.queries and qid not in seen, "profile unique known IDs")
            seen.add(qid)
            if type(case.get("terms")) is list:
                shape(case, ("id", "kind", "terms", "tags"), context="full profile query case")
                equal(case, self.audit.queries[qid], "profile full frozen query metadata")
            else:
                shape(case, ("id", "terms"), context="joined-term profile case")
                equal(case["terms"], " ".join(self.audit.queries[qid]["terms"]), "profile actual terms")
        replies = self.obj(record["replies"])
        check(type(replies) is list and len(replies) == 1 + len(cases) * 4, "profile reply inventory")
        ready(replies[0], "default")
        index = 1
        for iterations in (10, 100, 100, 100):
            for case in cases:
                validate_run(replies[index], case["id"], "top10", iterations, self.exact["default"][case["id"]])
                index += 1
        self.artifact(record["report"]); self.artifact(record["stderr"])
        trace = record["trace"]
        check(type(trace) is dict, "profile trace descriptor")
        if trace.get("status") == "retained":
            shape(trace, ("status", "artifact"), context="retained profile trace")
            self.packet.hash(trace["artifact"])
        else:
            shape(trace, ("status", "reason"), context="omitted profile trace")
            check(trace["status"] == "omitted" and type(trace["reason"]) is str and trace["reason"], "profile omission")
        check(type(record["limitations"]) is list and bool(record["limitations"]), "profile limits")
        if record["status"] == "retained_excluded_own_clippy":
            check(any("clippy" in value for value in record["limitations"]), "own-job overlap disclosed")
        return record

    def decision(self, results):
        descriptor = shape(self.d["decision"], ("artifact", "recorded_verdict", "checks", "result_bindings",
                                               "acceptance_rule"), context="decision descriptor")
        receipt = self.packet.obj(descriptor["artifact"])
        verdict = descriptor["recorded_verdict"]
        check(verdict in ("kept", "kept_partial_win", "reverted", "rejected",
                          "reverted_no_reliable_win", "reverted_material_per_query_regressions",
                          "reverted_pilot_regression_risk"),
              "recorded decision verdict")
        check((verdict.startswith("kept")) == (self.d["outcome"] == "accepted"), "outcome/verdict match")
        equal(receipt.get("verdict"), verdict, "retained decision verdict")
        if self.d["outcome"] == "accepted":
            equal(receipt.get("commit"), self.d["accepted_commit"], "accepted decision commit")
        if self.d["outcome"] == "accepted" or "candidate_binary_sha256" in receipt:
            equal(receipt.get("candidate_binary_sha256"), self.candidate["binary_sha256"], "decision candidate binary")
        if "binary_sha256" in receipt:
            equal(receipt["binary_sha256"], self.candidate["binary_sha256"], "decision legacy candidate binary")
        if "source_sha256" in receipt:
            claim = receipt["source_sha256"]
            # Historic scalar claims name the OR implementation; later maps
            # explicitly name every claimed source. Bind to already checked
            # copied source bytes, without inferring compiler derivation.
            if type(claim) is str:
                claim = {"src/query/boolean_query/or_maxscore.rs": claim}
            check(type(claim) is dict and bool(claim), "decision source claims")
            for path, hash_value in claim.items():
                relative(path); sha_value(hash_value)
                check(path in self.candidate["source_bindings"], "decision source needs copied bytes")
                equal(hash_value, self.source_files.get(path), "decision compiled source SHA")
        self.audit.value_checks(receipt, descriptor["checks"])
        bindings = descriptor["result_bindings"]
        check(type(bindings) is list, "decision result bindings")
        for binding in bindings:
            shape(binding, ("receipt_selector", "result_selector"), context="decision arithmetic binding")
            equal(select(receipt, binding["receipt_selector"], "decision receipt"),
                  select(results, binding["result_selector"], "decision computed result"), "decision evidence equality")
        rule = descriptor["acceptance_rule"]
        shape(rule, ("kind", "groups"), context="recorded acceptance rule")
        check(type(rule["groups"]) is list and len(set(rule["groups"])) == len(rule["groups"]), "acceptance groups")
        if self.d["outcome"] == "accepted":
            check(rule["kind"] == "parent-group-controls" and bool(rule["groups"]), "accepted metric rule")
            groups = results["parent_lane"]["groups"]
            check(all(k in groups and groups[k]["candidate_win_both_beyond_group_controls"] for k in rule["groups"]),
                  "accepted improvement must clear its recorded aggregate group controls")
        else:
            check(rule["kind"] == "not-accepted" and rule["groups"] == [], "rejected rule")
        # This rule is aggregate-only. Mixed rounds and individual losses remain visible.
        return {"verdict": verdict, "acceptance_rule": rule, "artifact": descriptor["artifact"]}

    def run(self):
        self.setup()
        if self.d["outcome"] == "pending":
            return {"id": self.d["id"], "outcome": "pending", "limitations": self.d["limitations"],
                    "parent_lane": self.d["parent_lane"], "actual_port_lane": self.d["actual_port_lane"]}
        result = {"id": self.d["id"], "outcome": self.d["outcome"], "stages": self.stage_results,
                  "exact_profiles": {profile: {"queries": len(rows), "returned_hits": sum(len(r["hits"]) for r in rows.values()),
                                                "raw_score_bits": True} for profile, rows in self.exact.items()},
                  "pilots": {p["folder"]: self.pilot(p) for p in self.d["pilots"]},
                  "parent_lane": self.lane(self.d["parent_lane"]),
                  "actual_port_lane": self.lane(self.d["actual_port_lane"], True),
                  "diagnostics": [self.diagnostic(record) for record in self.d["diagnostics"]],
                  "limitations": self.d["limitations"]}
        check(len(result["pilots"]) == len(self.d["pilots"]), "duplicate pilot folders")
        if self.d["id"] == "01":
            statuses = [r.get("status") for r in self.d["diagnostics"] if r.get("kind") == "profile"]
            check(statuses.count("overwritten_unavailable") == 1 and
                  statuses.count("retained_excluded_own_clippy") == 1 and statuses.count("retained_recreated") == 2,
                  "attempt01 complete profile lifecycle disclosure")
        result["decision"] = self.decision(result)
        return result


class Audit:
    def __init__(self):
        self.packet = Packet()
        d = shape(self.packet.descriptor, ("schema", "common", "attempts", "required_attempt_ids"), context="attempts.json")
        check(d["schema"] == "tantivy-or-hillclimb-attempts-v1", "attempt descriptor schema")
        self.common = shape(d["common"], ("queries", "frozen_gate", "baseline", "fixed_port", "indices",
                                          "harness", "guard_proofs", "payload_audit", "run_contract",
                                          "decision_trail", "pilot_controller_sha256", "comparison_controller"), context="common artifacts")
        check(type(d["attempts"]) is list and bool(d["attempts"]), "attempts array")
        self.descriptors = {}
        for attempt in d["attempts"]:
            check(type(attempt) is dict and type(attempt.get("id")) is str, "attempt ID shape")
            check(attempt["id"] not in self.descriptors, "duplicate attempt ID")
            self.descriptors[attempt["id"]] = attempt
        ids = d["required_attempt_ids"]
        check(type(ids) is list and len(set(ids)) == len(ids), "required attempt ID inventory")
        equal(list(self.descriptors), ids, "all required attempts in chronological order")
        check(all(f"{i:02d}" in ids for i in range(1, 13)), "historic01..12 cannot be omitted")
        self.load_common()

    @staticmethod
    def duration(value, context):
        check(type(value) in (int, float) and math.isfinite(value) and value >= 0, context)

    @staticmethod
    def telemetry(values, count):
        check(type(values) is list and len(values) == count, "complete telemetry inventory")
        for entry in values:
            shape(entry, ("at", "loadavg", "processes"), context="raw telemetry")
            Audit.duration(entry["at"], "telemetry timestamp")
            check(type(entry["loadavg"]) is list and len(entry["loadavg"]) == 3, "loadavg shape")
            for value in entry["loadavg"]:
                Audit.duration(value, "loadavg finite nonnegative")
            check(type(entry["processes"]) is list and bool(entry["processes"]) and
                  all(type(v) is str for v in entry["processes"]), "retained raw comm-only process inventory")

    def proof_fields(self, obj, bindings):
        check(type(bindings) is list, "hash proof bindings")
        selectors = set()
        for binding in bindings:
            shape(binding, ("selector", "artifact"), context="hash proof binding")
            selector = binding["selector"]
            check(type(selector) is list and all(type(v) is str for v in selector), "hash selector")
            check(tuple(selector) not in selectors, "duplicate hash selector")
            selectors.add(tuple(selector))
            equal(select(obj, selector, "hash proof"), self.packet.hash(binding["artifact"]), "relocated raw SHA binding")

    @staticmethod
    def value_checks(obj, checks):
        check(type(checks) is list, "value checks")
        selectors = set()
        for value in checks:
            shape(value, ("selector", "expected"), context="value assertion")
            selector = value["selector"]
            check(type(selector) is list and all(type(v) is str for v in selector), "value selector")
            check(tuple(selector) not in selectors, "duplicate value check")
            selectors.add(tuple(selector))
            equal(select(obj, selector, "value check"), value["expected"], "explicit retained value")

    def load_common(self):
        common, packet = self.common, self.packet
        equal(packet.hash(common["queries"]), "39a54b200392befbde83469841ce6eaefe7ec1850bb5b0e25c74a0eefa709a4e", "frozen full query bytes")
        self.cases = list(packet.lines(common["queries"]))
        check(len(self.cases) == 1227, "full1227 queries")
        for qid, q in enumerate(self.cases):
            shape(q, ("id", "kind", "terms", "tags"), context="frozen query case")
            equal(q["id"], qid, "query contiguous ID/order")
            check(q["kind"] in ("TERM", "AND", "OR", "PHRASE"), "query kind")
            for field in ("terms", "tags"):
                check(type(q[field]) is list and bool(q[field]) and all(type(v) is str and v for v in q[field]), "case terms/tags")
            expected_tag = "wiki20" if qid < 20 else "port-style" if qid <= 1220 else "correctness-only"
            check(expected_tag in q["tags"], "frozen suite ranges")
        self.queries = {q["id"]: q for q in self.cases}
        self.timed = [q["id"] for q in self.cases if "correctness-only" not in q["tags"]]
        equal(self.timed, list(range(1221)), "all1221 timed IDs")
        for kind, size in (("ALL", 1201), ("TERM", 300), ("AND", 300), ("OR", 301),
                           ("OR_2", 198), ("OR_3PLUS", 103), ("PHRASE", 300)):
            equal(sum(group_match(q, "port-style", kind) for q in self.cases), size, "frozen group size")
        check(type(common["harness"]) is dict and len(common["harness"]) == 5, "five full harness source artifacts")
        self.harness_hashes = {relative(path): packet.hash(artifact) for path, artifact in common["harness"].items()}
        equal(set(self.harness_hashes), {"harness/Cargo.toml", "harness/Cargo.lock", "harness/src/lib.rs",
                                       "harness/src/bin/tantivy_worker.rs", "harness/src/bin/configured_worker.rs"}, "harness inventory")
        shape(common["indices"], ("tantivy", "actual_port"), context="two external index maps")
        self.index_maps = {name: packet.ref(reference) for name, reference in common["indices"].items()}
        for name, mapping in self.index_maps.items():
            check(type(mapping) is dict and len(mapping) == 8, "eight-file recorded index inventory " + name)
            for path, hash_value in mapping.items():
                relative(path); sha_value(hash_value)
                check(not path.endswith(".lock"), "recorded index lockfile exclusion")
        baseline = shape(common["baseline"], ("binary_sha256", "source_commit", "compiled_snapshot", "exact_receipt",
                                               "provenance", "provenance_bindings"), context="baseline")
        equal(baseline["binary_sha256"], "2efc80ae6cc57c4163cab798f50bfb35ceb085c039bc4dd4b8fe31d22d387bbf", "frozen baseline binary")
        equal(baseline["source_commit"], "0e9329dfe782187a8feda0024102e4aabf517c28", "frozen baseline commit")
        provenance = packet.obj(baseline["provenance"])
        for key, value in {"pass": True, "baseline_binary_sha256": baseline["binary_sha256"],
                           "head": baseline["source_commit"], "differences": [], "compiled_non_doc_sources_match": 600}.items():
            equal(provenance.get(key), value, "retained baseline provenance/" + key)
        self.proof_fields(provenance, baseline["provenance_bindings"])
        packet.obj(baseline["compiled_snapshot"])
        fixed = shape(common["fixed_port"], ("binary_sha256", "source_commit", "source_snapshot", "source_bindings", "binary_proof", "harness"), context="fixed actualport")
        equal(fixed["binary_sha256"], "b0923daad72d662a056496ab115a57519ec474216fe05b1522e6529ae2154a2a", "fixed port binary")
        equal(fixed["source_commit"], "e5d1f81bff42c87886b12770f3c79648bb1963ed", "fixed port commit")
        source = packet.ref(fixed["source_snapshot"])
        equal(source.get("head"), fixed["source_commit"], "actual-port source snapshot commit")
        check(type(fixed["source_bindings"]) is dict and bool(fixed["source_bindings"]), "fixed port copied source bindings")
        for path, artifact in fixed["source_bindings"].items():
            relative(path)
            equal(source["files"].get(path), packet.hash(artifact), "fixed port compiled source blob")
        equal(packet.ref(fixed["binary_proof"]), fixed["binary_sha256"], "actualport recorded binary proof")
        check(type(fixed["harness"]) is dict and "harness/src/bin/port_worker.rs" in fixed["harness"], "actualport worker source retained")
        self.fixed_harness_hashes = {relative(path): packet.hash(artifact) for path, artifact in fixed["harness"].items()}
        payload = shape(common["payload_audit"], ("stdout", "receipt", "index_after"), context="payload audit")
        check(type(common["guard_proofs"]) is list and
              {"selector": ["payload_audit"], "artifact": payload["stdout"]} in common["guard_proofs"], "payload guard binding required")
        equal(packet.obj(payload["stdout"]), {
            "documents": 1000000, "field_docs": 917578, "integrity": "pass", "op": "audit",
            "payload_sha256": "893a75958e6d15c997d414d5fb5fc47c633aa18c68fcd33bc0f8e13abfa8dbb8",
            "physical_sha256": "6f7082e3ff479c7be7fe6f819857716630fc4b6b64acc2d6005f1107127029c7",
            "positions": 294827020, "postings": 126703012,
            "term_list_sha256": "dc15193a1ec1d93b778bf9cb0caedd1e71129180af0a1ea0e04b0f742c6c4d20", "terms": 1642896}, "full payload retained audit")
        payload_receipt = packet.obj(payload["receipt"])
        equal(payload_receipt.get("returncode"), 0, "payload audit stage status")
        equal(payload_receipt.get("stdout_sha256"), packet.hash(payload["stdout"]), "payload stdout SHA")
        equal(packet.obj(payload["index_after"]), self.index_maps["actual_port"], "audited actualport index file map")
        contract = packet.obj(common["run_contract"])
        equal(contract, {"base": baseline["source_commit"], "baseline_binary_sha256": baseline["binary_sha256"],
                         "primary_target": "candidate/fixed-port mean query median ratio <1 in AB and BA for all OR (301) and 3+ OR (103), beyond group self controls",
                         "minimum_distinct_attempts": 3,
                         "correctness": "1227 exact default and two configured profiles, independent callback fixtures and integrations",
                         "scope": "OR first; whole 1201 suite retained as negative/regression gate",
                         "index_policy": "same index files and format; bound changes must preserve original leaf replay",
                         "publication": "fork/main only, normal fast-forward"}, "unchanged hillclimb ruler")
        sha_value(common["pilot_controller_sha256"])
        equal(common["pilot_controller_sha256"], "28e34a3482ccbe0dbbad943c7bdcc13c93e56ff8b7ed063609cf20d7a8a6446e", "frozen pilot script bytes")
        self.load_frozen_gate()
        self.load_decisions()

    def load_frozen_gate(self):
        gate = shape(self.common["frozen_gate"], ("dumps", "receipt", "failures", "before", "after", "protocols"), context="frozen reference gate")
        receipt = self.packet.obj(gate["receipt"])
        for key, value in {"pass": True, "queries": 1227, "failures": 0, "unchanged": True}.items():
            equal(receipt.get(key), value, "frozen gate receipt")
        equal(self.packet.obj(gate["failures"]), [], "frozen gate failures")
        before = self.packet.obj(gate["before"])
        equal(self.packet.obj(gate["after"]), before, "frozen gate unchanged")
        equal(before.get("queries"), self.packet.hash(self.common["queries"]), "frozen gate query SHA")
        equal(before.get("binaries"), {"tantivy": "49aa819692bde50fe29328e96b331e9825a8ee13942402427dfa670a105e7cd1",
                                       "port": self.common["fixed_port"]["binary_sha256"]}, "frozen reference binary identity")
        equal(before.get("indices"), {"tantivy": self.index_maps["tantivy"], "port": self.index_maps["actual_port"]}, "frozen external index identity")
        equal(before.get("harness"), self.fixed_harness_hashes, "frozen actualport full harness source binding")
        equal(before.get("source", {}).get("balanced_harness"), self.fixed_harness_hashes, "frozen compiled harness binding")
        shape(gate["protocols"], ("tantivy", "port"), context="frozen protocols")
        readers = {engine: Protocol(self.packet, path, actual_port=engine == "port") for engine, path in gate["protocols"].items()}
        rows = self.packet.obj(gate["dumps"])
        check(type(rows) is list and len(rows) == 1227, "frozen reference dump inventory")
        self.frozen = {}
        for case, row in zip(self.cases, rows):
            shape(row, ("case", "tantivy", "port"), context="frozen dump row")
            equal(row["case"], case, "frozen full case metadata/order")
            values = {}
            for engine in ("tantivy", "port"):
                values[engine] = validate_dump(case, row[engine])
                equal(readers[engine].exchange({"op": "dump", "id": case["id"]}), row[engine], "frozen full protocol dump")
            equal(values["tantivy"], values["port"], "both frozen exact count/document/rawbit references")
            equal(values["tantivy"], validate_dump(case, row["tantivy"]["oracle"], True), "frozen Tantivy exhaustive oracle")
            self.frozen[case["id"]] = values
        for reader in readers.values():
            reader.end()
        self.frozen_port = {qid: row["port"] for qid, row in self.frozen.items()}
        equal(sum(len(row["tantivy"]["hits"]) for row in self.frozen.values()), 11509, "default returned rawbit values")

    def load_decisions(self):
        reader = csv.DictReader(io.StringIO(self.packet.raw(self.common["decision_trail"]).decode("utf-8")), delimiter="\t")
        check(reader.fieldnames is not None and "id" in reader.fieldnames and "verdict" in reader.fieldnames, "decision trail header")
        rows = list(reader)
        check(all(None not in row and all(v is not None for v in row.values()) for row in rows), "decision TSV shape")
        ids = [row["id"] for row in rows]
        check(len(set(ids)) == len(ids), "decision trail duplicate attempt")
        completed = [d for d in self.descriptors.values() if d["outcome"] != "pending"]
        equal(ids, [d["id"] for d in completed], "all completed decisions retained")
        for row, descriptor in zip(rows, completed):
            check(row["verdict"] in ("kept", "reverted", "rejected"), "decision trail verdict")
            check((row["verdict"] == "kept") == (descriptor["outcome"] == "accepted"), "trail/outcome match")

    def run(self):
        results = []
        seen = set()
        for descriptor in self.descriptors.values():
            parent = descriptor["parent"]
            if parent is not None and parent.get("kind") == "accepted":
                check(parent.get("attempt_id") in seen, "parents must precede children; no cycles")
            result = Attempt(self, descriptor).run()
            results.append(result)
            seen.add(descriptor["id"])
        completed = [r for r in results if r["outcome"] != "pending"]
        accepted = [r for r in completed if r["outcome"] == "accepted"]
        check(len(completed) >= 3 and bool(accepted), "distinct attempt floor")
        latest = accepted[-1]
        target = latest["actual_port_lane"]["actual_port_target_met"]
        return {"integrity_pass": True, "attempts": results, "completed_attempts": len(completed),
                "accepted_attempts": len(accepted), "rejected_attempts": len(completed) - len(accepted),
                "latest_accepted_attempt": latest["id"], "actual_port_target_met": target,
                "limitations": [
                    "Hashes bind retained bytes; they are not signatures or source-to-binary authenticity proof.",
                    "The baseline600-source match is a retained provenance receipt, not a fresh compiler/source authenticity audit.",
                    "Engine binaries, corpus and index payload remain external; index hash maps cannot re-audit absent bytes.",
                    "Timed checksums can collide; full exact gates independently compare ordered documents and raw score bits.",
                    "Self-control spread is an observed threshold, not a confidence interval or guarantee for unseen workloads.",
                    "Configured profiles have own-oracle correctness gates, not measured configured Java/port performance.",
                    "Tantivy omits top10 totals; actual port tracks lower-bound hit totals with threshold1000.",
                    "Mixed rounds and all individual losses remain in the output; no universal performance/API parity claim.",
                    "Latency was recorded under user background load; retained telemetry does not remove contention.",
                    "Profiles are diagnostic and cannot recover overwritten original attempt01 raw evidence."]}


def main():
    try:
        check(len(sys.argv) == 1, "usage: python3 recompute.py (writes JSON to stdout)")
        result = Audit().run()
    except (AuditError, OSError, ValueError, KeyError, TypeError, StopIteration, OverflowError, zlib.error) as error:
        print(json.dumps({"integrity_pass": False, "error": str(error)}, sort_keys=True), file=sys.stderr)
        return 1
    print(json.dumps(result, indent=2, allow_nan=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
