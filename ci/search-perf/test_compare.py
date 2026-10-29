import contextlib
import io
import unittest
from unittest.mock import patch

import compare


def pair(ratio=1.0, checksum=5):
    base = {name: (100, 5) for name in compare.CASES}
    candidate = {name: (100 * ratio, checksum) for name in compare.CASES}
    return base, candidate


class GateTest(unittest.TestCase):
    def evaluate(self, pairs):
        with contextlib.redirect_stdout(io.StringIO()):
            return compare.evaluate(pairs)

    def test_correct_fast_candidate_passes(self):
        self.assertFalse(self.evaluate([pair(0.9)] * 7))

    def test_sustained_slowdown_fails(self):
        self.assertTrue(self.evaluate([pair(1.3)] * 7))

    def test_result_change_fails(self):
        self.assertTrue(self.evaluate([pair(checksum=6)] * 7))

    def test_unstable_comparison_fails(self):
        ratios = [0.7, 0.75, 0.8, 1.0, 1.3, 1.4, 1.5]
        self.assertTrue(self.evaluate([pair(r) for r in ratios]))

    @patch('compare.subprocess.run')
    def test_rejects_malformed_results(self, run):
        for output in ('', 'term_top10 0 1 3\n', 'term_top10 5 0 3\n',
                       'term_top10 4 1 3\nterm_top10 4 1 3\n', 'bad data'):
            run.return_value.stdout = output
            with self.subTest(output=output), self.assertRaises(ValueError):
                compare.measure('binary', 'index')


if __name__ == '__main__':
    unittest.main()
