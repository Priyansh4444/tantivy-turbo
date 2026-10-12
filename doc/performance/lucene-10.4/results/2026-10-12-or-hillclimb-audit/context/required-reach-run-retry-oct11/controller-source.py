#!/usr/bin/env python3
"""Source-only authored controller; root executes once, outside latency jobs.

Build a throwaway observer, then dump every frozen long OR exactly once. No
latency samples, performance claims, frozen-controller imports, or user-process
changes. A fresh required-reach-run folder is mandatory, including on failure.
"""
import difflib
import hashlib
import json
import math
import os
from pathlib import Path
import select
import shutil
import signal
import struct
import subprocess
import sys
import time

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[1]
RUN = OUT / "required-reach-run-retry-oct11"
REPO = ROOT / "tantivy-or-required-diagnostic-oct11"
PORT_REPO = ROOT / "lucene-rs-idf-fix-oct06"
INDEX = ROOT / "bench/wiki-1m-native-aligned-v11-oct05.idx"
QUERIES = OUT.parent / "lucene-rs-balanced-oct06/queries.jsonl"
PARENT = OUT / "parent-11.json"
REFERENCE = OUT / "attempt-11/verify/dumps.json"
RULER = ROOT / "bench/native-profile-aligned-oct05/frozen-ruler.json"
OR_PATH = "src/query/boolean_query/or_maxscore.rs"
PARENT_COMMIT = "43f06833a325d2a18c03bd7e4fa65f2f27944292"
SOURCE_SHA = "1991289b0e36bea08849549cc8ff6ab9e6b156063dc5d1ff0260be2482d9b6cf"
QUERY_SHA = "39a54b200392befbde83469841ce6eaefe7ec1850bb5b0e25c74a0eefa709a4e"
REFERENCE_SHA = "87f4a01b34c244ebabc1352001d77c0570d3010b19cc8376a323a41331d742c4"
HARNESSES = {
    "Cargo.toml": "f91587119b02f3f3b5105c8130b1c196b068adf4d704157fe6c857971bf7df7d",
    "Cargo.lock": "21dfbb56abf6714579bdde51f28e1d708a0c82a3f61b3acd3853ec4835116e0c",
    "src/lib.rs": "63aee1908ba57563748428a1bb42c2cbd079a3284907043f59744277a08359b0",
    "src/bin/tantivy_worker.rs": "b502c978356e02f02171b68f5143ea1d05fb76beba5354c26e008ef16d45e562",
    "src/bin/configured_worker.rs": "d63a488d294a42a238284eb4bf3936bb835d1fc8bc9a228997545c9561e91664",
}
COMPILER = """rustc 1.99.0 (b940084d7 2026-09-28)
binary: rustc
commit-hash: b940084d7eb6a299eb4bfeb8e34901bc051e7ac4
commit-date: 2026-09-28
host: x86_64-unknown-linux-gnu
release: 1.99.0
LLVM version: 23.1.1
"""
SCHEMA = "tantivy-required-optional-reach-v1"
PROFILE = {"opt_level": 3, "lto": "fat", "codegen_units": 1,
           "overflow_checks": False, "panic": "abort", "debug": False}


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def write(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def git(repo, *args):
    return subprocess.check_output(["git", *args], cwd=repo, timeout=60)


def files(directory, *, index=False):
    result = {}
    for path in sorted(directory.rglob("*")):
        require(not path.is_symlink(), f"symlink in guarded directory: {path}")
        if path.is_file() and not (index and path.name.endswith(".lock")):
            result[str(path.relative_to(directory))] = sha(path)
    return result


def source(repo):
    tracked = set(git(repo, "ls-files", "--cached", "-z").split(b"\0"))
    untracked = set(git(repo, "ls-files", "--others", "--exclude-standard", "-z").split(b"\0"))
    generated = lambda name: os.fsdecode(name).startswith("tests/compat_tests_data/index_v11/")
    names = sorted(tracked | {name for name in untracked if not generated(name)})
    hashes = {}
    for name in names:
        if name:
            path = repo / os.fsdecode(name)
            require(path.is_file() and not path.is_symlink(), f"invalid guarded source: {path}")
            hashes[os.fsdecode(name)] = sha(path)
    status = git(repo, "status", "--porcelain").decode().splitlines()
    return {"head": git(repo, "rev-parse", "HEAD").decode().strip(),
            "status": [line for line in status if line != "?? tests/compat_tests_data/index_v11/"],
            "files": hashes}


def identity():
    descriptor = json.loads(PARENT.read_text())
    parent_files = {"descriptor": sha(PARENT), "reference": sha(REFERENCE), "ruler": sha(RULER)}
    for key in ("binary", "compiled_sources", "exact_gate"):
        parent_files[key] = sha(Path(descriptor[key]))
    return {"diagnostic_sources": source(REPO), "port_sources": source(PORT_REPO),
            "frozen_harness": files(OUT / "harness"),
            "staged_harness": {name: sha(RUN / name) for name in HARNESSES},
            "pristine_harness": {name: sha(RUN / "original-harness" / name) for name in HARNESSES},
            "index": files(INDEX, index=True), "queries": sha(QUERIES),
            "parent": parent_files, "controller": sha(Path(__file__)),
            "diagnostic_snapshot": sha(OUT / "required-reach-source-or_maxscore.rs"),
            "diagnostic_patch": sha(OUT / "required-reach-source.patch"),
            "static_receipt": sha(OUT / "required-reach-static-receipt.json")}


def after_identity():
    # Failed guard readbacks are retained as failed after-snapshots instead of
    # preventing the command/log receipt from being written.
    try:
        return identity()
    except BaseException as exc:
        return {"guard_error": {"type": type(exc).__name__, "message": str(exc)}}


def environment():
    env = dict(os.environ)
    for name in list(env):
        if name.startswith("CARGO_PROFILE_") or name in (
                "CARGO_ENCODED_RUSTFLAGS", "RUSTFLAGS", "RUSTUP_TOOLCHAIN", "CARGO_TARGET_DIR"):
            del env[name]
    env.update(RUSTUP_TOOLCHAIN="1.99.0", RUSTFLAGS="-C target-cpu=native",
               CARGO_BUILD_JOBS="2", CARGO_TARGET_DIR=str(RUN / "target"))
    return env


def prepare():
    require(files(OUT / "harness") == HARNESSES, "frozen five-file harness changed")
    require(git(REPO, "rev-parse", "HEAD").decode().strip() == PARENT_COMMIT, "diagnostic parent changed")
    require(git(REPO, "diff", "HEAD", "--name-only").decode().splitlines() == [OR_PATH], "diagnostic source scope changed")
    require(git(REPO, "diff", "--cached", "--name-only").decode().strip() == "", "diagnostic has unexpected staged changes")
    unexpected_untracked = [os.fsdecode(name) for name in git(REPO, "ls-files", "--others", "--exclude-standard", "-z").split(b"\0")
                            if name and not os.fsdecode(name).startswith("tests/compat_tests_data/index_v11/")]
    require(not unexpected_untracked, "diagnostic has unexpected untracked source files")
    require(sha(REPO / OR_PATH) == SOURCE_SHA, "diagnostic source changed")
    require(sha(OUT / "required-reach-source-or_maxscore.rs") == SOURCE_SHA, "snapshot changed")
    require(sha(OUT / "required-reach-source.patch") == "6b669b8be6f6a87d51b1df34b20632ea382aa09920c4442f83857400a2b881f8", "diagnostic patch changed")
    require(git(PORT_REPO, "rev-parse", "HEAD").decode().strip() == "e5d1f81bff42c87886b12770f3c79648bb1963ed", "fixed port source parent changed")
    require(sha(QUERIES) == QUERY_SHA, "frozen queries changed")
    require(sha(REFERENCE) == REFERENCE_SHA, "accepted-11 dump reference changed")
    descriptor = json.loads(PARENT.read_text())
    require(descriptor["source_commit"] == PARENT_COMMIT, "parent descriptor source changed")
    expected = {"binary": "e11c4939984690d2c2d781cb6ace40d5e818e3a18a3f9f558d6cbcfd62546e5d",
                "compiled_sources": "9972db388f51e23da6de3f586e9a0f16073ce4fa675c0b354d3cf1b052a45e5f",
                "exact_gate": "35c1fe105532a1d3fe8a699c1d088437b1873793f33b01510f952441ad98a3ed"}
    for key, digest in expected.items():
        require(descriptor[key + "_sha256"] == digest and sha(Path(descriptor[key])) == digest,
                f"parent descriptor/artifact changed: {key}")
    require(json.loads(Path(descriptor["exact_gate"]).read_text())["pass"] is True,
            "parent exact gate not passed")
    ruler = json.loads(RULER.read_text())
    index_expected = {str(Path(path).relative_to(INDEX)): value["sha256"]
                      for path, value in ruler["artifacts"].items()
                      if str(path).startswith(str(INDEX) + "/") and not str(path).endswith(".lock")}
    require(index_expected and files(INDEX, index=True) == index_expected, "frozen full index changed")
    for name in HARNESSES:
        for destination in (RUN / name, RUN / "original-harness" / name):
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(OUT / "harness" / name, destination)
    cargo = (RUN / "Cargo.toml").read_text()
    old = 'tantivy = { path = "../../../tantivy-or-hillclimb-oct07" }'
    new = 'tantivy = { path = "' + str(REPO) + '" }'
    require(cargo.count(old) == 1, "unexpected Tantivy dependency declaration")
    adjusted = cargo.replace(old, new)
    (RUN / "Cargo.toml").write_text(adjusted)
    (RUN / "harness-dependency.patch").write_text("".join(difflib.unified_diff(
        cargo.splitlines(keepends=True), adjusted.splitlines(keepends=True),
        fromfile="original-harness/Cargo.toml", tofile="Cargo.toml")))
    write(RUN / "harness-copy.json", {"original": HARNESSES,
          "staged": {name: sha(RUN / name) for name in HARNESSES},
          "only_change": {"file": "Cargo.toml", "old": old, "new": new},
          "cargo_project": str(RUN), "port_dependency_resolves_to": str(PORT_REPO)})
    cases = [json.loads(line) for line in QUERIES.read_text().splitlines()]
    require(len(cases) == 1227 and [case["id"] for case in cases] == list(range(1227)), "query identities changed")
    selected = [case for case in cases if 20 <= case["id"] <= 1220
                and case["kind"] == "OR" and len(case["terms"]) >= 3]
    require(len(selected) == 103 and all(3 <= len(case["terms"]) <= 32 for case in selected), "long OR scope changed")
    write(RUN / "cases.json", selected)
    references = {row["case"]["id"]: row for row in json.loads(REFERENCE.read_text())}
    require(len(references) == 1227, "duplicate/missing reference cases")
    for case in selected:
        require(references[case["id"]]["case"] == case, "reference case differs")
    return selected, references


def build(env):
    folder = RUN / "build"
    folder.mkdir()
    before = identity()
    write(folder / "before.json", before)
    command = ["cargo", "build", "--locked", "--release", "--bin", "tantivy_worker"]
    write(folder / "command.json", {"argv": command, "cwd": str(RUN), "timeout_seconds": 1800,
          "compiler_argv": ["rustc", "-Vv"], "compiler_timeout_seconds": 30,
          "rustflags": env["RUSTFLAGS"], "jobs": 2, "target_dir": env["CARGO_TARGET_DIR"], "profile": PROFILE})
    started = time.monotonic()
    returncode, error, compiler, compiler_returncode = None, None, None, None
    try:
        compiler_command = subprocess.run(["rustc", "-Vv"], env=env, text=True, capture_output=True, timeout=30)
        compiler = compiler_command.stdout
        compiler_returncode = compiler_command.returncode
        (folder / "compiler.txt").write_text(compiler)
        (folder / "compiler.stderr").write_text(compiler_command.stderr)
        require(compiler_command.returncode == 0, "compiler identity command failed")
        require(compiler == COMPILER, "compiler/LLVM differs from frozen build")
        with (folder / "stdout.log").open("wb") as stdout, (folder / "stderr.log").open("wb") as stderr:
            process = subprocess.Popen(command, cwd=RUN, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
            try:
                returncode = process.wait(timeout=1800)
            except BaseException:
                # Only this controller's freshly created Cargo process group.
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=10)
                raise
    except BaseException as exc:
        error = {"type": type(exc).__name__, "message": str(exc)}
        if isinstance(exc, subprocess.TimeoutExpired):
            returncode = 124
            if compiler is None:
                for name, content in (("compiler.txt", exc.stdout), ("compiler.stderr", exc.stderr)):
                    (folder / name).write_bytes(content if isinstance(content, bytes) else (content or "").encode())
    after = after_identity()
    write(folder / "after.json", after)
    receipt = {"returncode": returncode, "error": error, "source_index_parent_unchanged": before == after,
               "compiler_matches": compiler == COMPILER, "compiler_returncode": compiler_returncode,
               "wall_seconds": time.monotonic() - started,
               "profile": PROFILE, "rustflags": env["RUSTFLAGS"], "jobs": 2,
               "logs": {name: sha(folder / name) for name in ("stdout.log", "stderr.log", "compiler.txt", "compiler.stderr", "command.json")
                        if (folder / name).is_file()}}
    binary = RUN / "target/release/tantivy_worker"
    if binary.is_file():
        receipt["binary"] = str(binary)
        receipt["binary_sha256"] = sha(binary)
    write(folder / "receipt.json", receipt)
    require(returncode == 0 and error is None, "diagnostic build failed; retained build logs/receipt")
    require(before == after, "build changed guarded input; retained receipts")
    require(binary.is_file(), "build did not produce worker")
    return binary, receipt["binary_sha256"]


def integer(value):
    return type(value) is int and value >= 0


def validate_dump(case, value, reference):
    require(value["id"] == case["id"] and value["reported"] is None, "dump identity/report changed")
    count, hits = value["count"], value["hits"]
    require(integer(count) and count <= 1000000 and len(hits) == min(10, count), "invalid hit count")
    previous, seen = None, set()
    for doc, bits in hits:
        require(integer(doc) and doc < 1000000 and integer(bits) and bits < 2 ** 32 and doc not in seen, "invalid/duplicate hit")
        score = struct.unpack("<f", struct.pack("<I", bits))[0]
        order = (-score, doc)
        require(math.isfinite(score) and (previous is None or previous <= order), "unordered/nonfinite hit")
        previous = order
        seen.add(doc)
    require(value["oracle"] == {"count": count, "hits": hits}, "optimized vs independent exhaustive mismatch")
    for key in ("count", "hits", "oracle"):
        require(value[key] == reference[key], f"accepted-11 dump mismatch: {key}")


def validate_counters(case, counters):
    c = counters
    require(c["schema"] == SCHEMA and c["admitted"] is True and c["completed"] is True, "incomplete/fallback diagnostic")
    require(c["original_num_terms"] == len(case["terms"]) and c["max_doc"] == 1000000, "resolved term/domain mismatch")
    histogram_names = ("gap_candidates_histogram", "driver_batch_length_histogram")
    for name, value in c.items():
        if name not in ("schema", "admitted", "completed", *histogram_names):
            require(integer(value), f"invalid counter: {name}")
    for name in histogram_names:
        require(len(c[name]) == 129 and all(integer(value) for value in c[name]), f"invalid histogram: {name}")
    require(c["regions"] == sum(c[name] for name in ("globally_certified_terminal_regions", "all_optional_regions",
            "single_essential_regions", "multiple_essential_regions")), "region partition identity")
    require(c["first_optional_probes"] == c["first_optional_matches"] + c["first_optional_misses"], "probe identity")
    require(c["single_essential_candidate_visits"] == c["first_optional_probes"] + c["first_optional_preprobe_rejects"], "candidate/probe identity")
    batches, gaps = c[histogram_names[1]], c[histogram_names[0]]
    require(c["nonempty_driver_batches"] == sum(batches[1:]) and c["empty_driver_batches"] == batches[0], "batch histogram identity")
    require(c["driver_batch_entries_already_scored"] == sum(i * value for i, value in enumerate(batches))
            == c["single_essential_candidate_visits"], "owned batch/candidate identity")
    require(c["full_driver_batches"] == batches[128] and c["partial_driver_batches"] == sum(batches[1:128]), "batch length classes")
    require(c["selected_finite_driver_batches"] + c["selected_tail_driver_batches"] == c["nonempty_driver_batches"], "selected block classes")
    require(gaps[0] == 0 and c["deduplicated_gaps"] == sum(gaps), "gap histogram identity")
    require(c["unique_skippable_candidates_including_trigger"] == sum(i * value for i, value in enumerate(gaps)), "gap candidate identity")
    require(c["unique_additional_candidates_after_trigger"] == c["unique_skippable_candidates_including_trigger"] - c["deduplicated_gaps"]
            == c["shadow_covered_candidate_visits"], "shadow gap deduplication identity")
    require(c["gap_candidates_max"] == max((i for i, value in enumerate(gaps) if value), default=0), "gap max/histogram identity")
    require(c["shadow_covered_callbacks"] == 0 and c["lower_optional_probes_after_certified_miss"] == 0, "certificate would suppress callbacks/probes")
    require(c["certificate_pass_misses"] == c["next_lower_prefix_rejects_after_certified_miss"], "certified miss not cheap next rejection")
    require(c["certificate_regions_after_initial_failure"] <= c["certificate_regions"] <= c["certificate_builds"] <= c["single_essential_regions"], "certificate region bounds")
    require(c["certificate_equal_threshold_misses"] <= c["certificate_pass_misses"] <= c["first_optional_misses"], "certificate miss bounds")
    require(c["deduplicated_gaps"] <= c["certificate_pass_misses"] and c["gaps_to_region_end"] <= c["gaps_exhausting_pending_batch"] <= c["deduplicated_gaps"], "gap bounds")
    require(sum(c[name] for name in ("next_optional_exactly_region_end", "next_optional_beyond_region_end", "next_optional_terminated"))
            == c["gaps_to_region_end"], "optional endpoint partition")
    require(sum(c[name] for name in ("driver_first_unread_exactly_region_end", "driver_first_unread_beyond_region_end", "driver_first_unread_terminated"))
            == c["nonempty_driver_batches"], "driver first-unread ownership")
    require(c["regions_with_multiple_nonempty_driver_batches"] == 0, "parent one-block ownership disproved")
    require(c["next_lower_prefix_rejects_after_certified_miss"] <= c["next_lower_prefix_rejects_after_miss"] <= c["first_optional_misses"], "next rejection subset")
    require(c["single_essential_candidate_visits"] <= c["candidate_visits"] and c["callbacks"] <= c["candidate_visits"], "candidate bounds")


def summarize(rows):
    totals = {}
    for row in rows:
        for name, value in row["counters"].items():
            if name in ("schema", "admitted", "completed", "original_num_terms", "max_doc"):
                continue
            if isinstance(value, list):
                totals.setdefault(name, [0] * len(value))
                totals[name] = [left + right for left, right in zip(totals[name], value)]
            elif name.endswith("_max"):
                totals[name] = max(totals.get(name, 0), value)
            else:
                totals[name] = totals.get(name, 0) + value
    gaps = totals.get("deduplicated_gaps", 0)
    builds = totals.get("certificate_builds", 0)
    additional = totals.get("unique_additional_candidates_after_trigger", 0)
    return {"status": "reach_only_no_latency_claim", "cases": len(rows), "all_case_ids": [r["case"]["id"] for r in rows],
            "zero_certificate_queries": [r["case"]["id"] for r in rows if r["counters"]["certificate_pass_misses"] == 0],
            "zero_gap_queries": [r["case"]["id"] for r in rows if r["counters"]["deduplicated_gaps"] == 0],
            "totals": totals, "additional_candidates_per_gap": additional / gaps if gaps else None,
            "additional_candidates_per_certificate_build": additional / builds if builds else None,
            "excluded_bound_terms_added": sum(r["counters"]["certificate_builds"] * (r["counters"]["original_num_terms"] - 1) for r in rows),
            "driver_decoding_or_bm25_savings_claimed": False,
            "screen": "Root must assess whether future candidate-work reach warrants an isolated production trial; these counts prove no speedup."}


def diagnostic(binary, binary_sha, cases, references, env):
    folder = RUN / "diagnostic"
    folder.mkdir()
    before = identity()
    write(folder / "before.json", before)
    command = [str(binary), str(INDEX), str(QUERIES)]
    write(folder / "command.json", {"argv": command, "cpu_affinity": [4], "timeout_seconds": 3600,
          "per_response_timeout_seconds": 120, "requests": "one dump per all 103 frozen long OR cases", "binary_sha256": binary_sha})
    started, deadline = time.monotonic(), time.monotonic() + 3600
    rows, failure, process, returncode = [], None, None, None
    stderr_offset, stdout_buffer = 0, bytearray()
    with (folder / "worker.stderr").open("wb", buffering=0) as stderr, (folder / "worker.stdout").open("wb") as stdout, \
            (folder / "protocol.jsonl").open("w") as protocol, (folder / "counters.jsonl").open("w") as counters_log:
        def read_response():
            response_deadline = min(deadline, time.monotonic() + 120)
            while b"\n" not in stdout_buffer:
                remaining = response_deadline - time.monotonic()
                require(remaining > 0, "diagnostic deadline")
                require(select.select([process.stdout], [], [], remaining)[0], "worker response timeout")
                block = os.read(process.stdout.fileno(), 65536)
                require(bool(block), f"worker EOF returncode={process.poll()}")
                stdout.write(block)
                stdout.flush()
                stdout_buffer.extend(block)
                require(len(stdout_buffer) <= 1 << 20, "oversized worker line")
            line, _, rest = stdout_buffer.partition(b"\n")
            stdout_buffer[:] = rest
            value = json.loads(line)
            protocol.write(json.dumps({"response": value}, separators=(",", ":"), allow_nan=False) + "\n")
            protocol.flush()
            return value

        def read_stderr_records():
            nonlocal stderr_offset
            with (folder / "worker.stderr").open("rb") as reader:
                reader.seek(stderr_offset)
                raw = reader.read()
            require(not raw or raw.endswith(b"\n"), "incomplete stderr record after response")
            begin = stderr_offset
            stderr_offset += len(raw)
            records = []
            for line in raw.splitlines():
                value = json.loads(line)
                require(isinstance(value, dict) and value.get("schema") == SCHEMA, "unexpected worker stderr line; raw retained")
                records.append(value)
            return records, {"begin": begin, "end": stderr_offset}

        try:
            require(sha(binary) == binary_sha, "diagnostic binary changed before launch")
            process = subprocess.Popen(command, cwd=RUN, env=env, stdin=subprocess.PIPE,
                                       stdout=subprocess.PIPE, stderr=stderr,
                                       preexec_fn=lambda: os.sched_setaffinity(0, {4}))
            ready = read_response()
            write(folder / "ready.json", ready)
            expected = {"op": "ready", "engine": "tantivy", "queries": 1227, "max_doc": 1000000,
                        "num_docs": 1000000, "doc_count": 917578, "sum_total_term_freq": 294827020,
                        "k1_bits": "3f99999a", "b_bits": "3f400000", "collector": "top10-no-totals", "query_cache": "disabled"}
            require(ready == expected, "worker ready differs from frozen workload")
            require(read_stderr_records()[0] == [], "unexpected pruning at startup")
            for case in cases:
                request = {"op": "dump", "id": case["id"]}
                protocol.write(json.dumps({"request": request}, separators=(",", ":")) + "\n")
                protocol.flush()
                process.stdin.write((json.dumps(request, separators=(",", ":")) + "\n").encode())
                process.stdin.flush()
                response = read_response()
                records, offsets = read_stderr_records()
                row = {"case": case, "request": request, "response": response, "stderr_offsets": offsets,
                       "stderr_records": records}
                # Keep even a failed association/response before validating it.
                counters_log.write(json.dumps(row, separators=(",", ":"), allow_nan=False) + "\n")
                counters_log.flush()
                require(len(records) == 1, f"expected exactly one observer for query {case['id']}, got {len(records)}")
                row["counters"] = records[0]
                validate_dump(case, response, references[case["id"]]["tantivy"])
                validate_counters(case, records[0])
                rows.append(row)
            process.stdin.close()
            returncode = process.wait(timeout=15)
            require(returncode == 0, "worker failed on exit")
            require(read_stderr_records()[0] == [], "late/unassociated diagnostic records")
            extra = process.stdout.read()
            stdout.write(extra)
            stdout.flush()
            require(not stdout_buffer and not extra, "extra worker stdout")
        except BaseException as exc:
            failure = {"type": type(exc).__name__, "message": str(exc)}
        finally:
            if process is not None and process.poll() is None:
                if process.stdin and not process.stdin.closed:
                    process.stdin.close()
                try:
                    returncode = process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.terminate()
                    try:
                        returncode = process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        returncode = process.wait(timeout=10)
            elif process is not None:
                returncode = process.returncode
            if process is not None and process.stdout:
                # Process has been reaped; retain any residual raw output even
                # when response validation or stderr association failed.
                stdout.write(process.stdout.read())
                stdout.flush()
                process.stdout.close()
    write(folder / "dumps-and-counters.json", rows)
    write(folder / "summary.json", summarize(rows))
    after = after_identity()
    write(folder / "after.json", after)
    binary_after_sha = sha(binary) if binary.is_file() else None
    receipt = {"returncode": returncode, "error": failure, "requested_cases": 103,
               "verified_cases": len(rows), "source_index_parent_unchanged": before == after,
               "binary_unchanged": binary_after_sha == binary_sha,
               "binary_before_sha256": binary_sha, "binary_after_sha256": binary_after_sha,
               "wall_seconds": time.monotonic() - started, "latency_claim": False,
               "logs": {name: sha(folder / name) for name in ("worker.stderr", "worker.stdout", "protocol.jsonl", "counters.jsonl", "command.json")},
               "pass": failure is None and returncode == 0 and len(rows) == 103 and before == after and binary_after_sha == binary_sha}
    write(folder / "receipt.json", receipt)
    require(receipt["pass"], "reach diagnostic failed; all existing logs/receipts retained")
    return receipt


def main():
    require(len(sys.argv) == 1, "usage: python3 -B root-run-required-reach.py")
    RUN.mkdir()  # Fail even if an earlier failed run left this directory.
    error, result = None, None
    try:
        shutil.copyfile(Path(__file__), RUN / "controller-source.py")
        for name in ("required-reach-source-or_maxscore.rs", "required-reach-source.patch", "required-reach-static-receipt.json", "required-reach-report.md"):
            destination = RUN / "source-evidence" / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(OUT / name, destination)
        cases, references = prepare()
        env = environment()
        binary, digest = build(env)
        result = diagnostic(binary, digest, cases, references, env)
    except BaseException as exc:
        error = {"type": type(exc).__name__, "message": str(exc)}
        write(RUN / "error.json", error)
    finally:
        # Retain hashes of stage receipts, commands, original/staged source,
        # compiler and raw output even on failure; omit the build cache/ELFs.
        inventory = {}
        for path in sorted(RUN.rglob("*")):
            if path.is_file() and "target" not in path.relative_to(RUN).parts:
                inventory[str(path.relative_to(RUN))] = sha(path)
        write(RUN / "artifact-hashes.json", {"files": inventory,
              "excluded": ["target/** build cache and binary; binary SHA recorded in build/diagnostic receipts"]})
        final = {"pass": error is None and result is not None and result["pass"],
                 "error": error, "diagnostic": result, "latency_claim": False}
        write(RUN / "receipt.json", final)
        print(json.dumps(final, separators=(",", ":"), allow_nan=False), flush=True)
    return 0 if final["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
