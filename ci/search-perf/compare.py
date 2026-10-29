#!/usr/bin/env python3
"""Build a common harness against two revisions and compare paired samples.

Only the selected synthetic, warm search cases are gated. This does not measure
cold starts, indexing, memory, or every query. Shared runners can be noisy;
failed or inconclusive runs need investigation and may be rerun.
"""
import argparse
import json
import pathlib
import statistics
import subprocess
import tempfile

CASES = ("term_top10", "and_top10", "or_top10", "phrase_top10", "and_count", "or_count")
ROUNDS = 7


def build(root, work, source):
    work.mkdir()
    (work / "src").mkdir()
    (work / "src" / "main.rs").write_text(source.read_text())
    # JSON string quoting is also valid TOML string quoting for these paths.
    (work / "Cargo.toml").write_text(
        '[package]\nname = "turbo-perf-gate"\nversion = "0.0.0"\nedition = "2021"\n'
        '[dependencies]\ntantivy = { package = "tantivy-turbo", path = '
        + json.dumps(str(root.resolve())) + ' }\n'
    )
    subprocess.run(["cargo", "+stable", "build", "--release", "--manifest-path", str(work / "Cargo.toml")], check=True)
    return work / "target" / "release" / "turbo-perf-gate"


def measure(binary, index):
    output = subprocess.run([str(binary), "measure", str(index)], check=True, text=True, capture_output=True).stdout
    result = {}
    for line in output.splitlines():
        name, elapsed, iterations, checksum = line.split()
        if name not in CASES or name in result or int(iterations) <= 0 or int(elapsed) <= 0:
            raise ValueError(f"invalid benchmark output: {line}")
        result[name] = (int(elapsed) / int(iterations), int(checksum))
    if set(result) != set(CASES):
        raise ValueError(f"missing benchmark cases: {output}")
    return result


def evaluate(pairs):
    failed = False
    for name in CASES:
        ratios = []
        for base, candidate in pairs:
            if base[name][1] != candidate[name][1]:
                print(f"FAIL {name}: result fingerprint differs")
                failed = True
                break
            ratios.append(candidate[name][0] / base[name][0])
        else:
            median = statistics.median(ratios)
            print(f"{name}: median candidate/base {median:.3f}x; paired ratios " + ", ".join(f"{x:.3f}" for x in ratios))
            # Fail noisy comparisons rather than silently passing an uncertain
            # result. Shared-host timing cannot establish small regressions.
            mad = statistics.median(abs(x - median) for x in ratios) / median
            if mad > 0.15:
                print(f"FAIL {name}: inconclusive (relative median deviation {mad:.1%}); rerun")
                failed = True
            elif median > 1.25:
                print(f"FAIL {name}: median slowdown exceeds 25%")
                failed = True
    return failed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base", type=pathlib.Path)
    parser.add_argument("candidate", type=pathlib.Path)
    args = parser.parse_args()
    source = pathlib.Path(__file__).resolve().with_name("harness.rs")
    with tempfile.TemporaryDirectory(prefix="turbo-perf-") as tmp:
        work = pathlib.Path(tmp)
        base = build(args.base, work / "base", source)
        candidate = build(args.candidate, work / "candidate", source)
        index = work / "index"
        subprocess.run([str(base), "build", str(index)], check=True)
        pairs = []
        for i in range(ROUNDS):
            # Alternate order to reduce systematic drift and thermal effects.
            if i % 2 == 0:
                b, c = measure(base, index), measure(candidate, index)
            else:
                c, b = measure(candidate, index), measure(base, index)
            pairs.append((b, c))
        if evaluate(pairs):
            raise SystemExit(1)


if __name__ == "__main__":
    main()
