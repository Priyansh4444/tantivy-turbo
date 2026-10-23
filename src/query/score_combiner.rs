use crate::query::Scorer;
use crate::Score;

/// The `ScoreCombiner` trait defines how to compute
/// an overall score given a list of scores.
pub trait ScoreCombiner: Default + Clone + Send + Copy + 'static {
    /// Whether Block-WAND may drive this combiner.
    ///
    /// Block-WAND prunes blocks by comparing the *sum* of the per-term block
    /// maxima against the threshold, and scores the surviving documents the
    /// same way. That is only the right answer for a combiner that sums. A
    /// combiner aggregating any other way (dis_max takes the maximum) has to
    /// see every matching term, so it must not be pruned this way.
    const SUPPORTS_BLOCK_WAND: bool = false;

    /// Whether this combiner sums child f32 scores in f64 and rounds once.
    /// Optimized summing scorers require this convention. Other combiners
    /// retain their own arithmetic through the generic scorer.
    const SUMS_IN_F64: bool = false;

    /// Whether `update` reads per-document scores. Bulk bitset fill for
    /// COUNT is valid when this is false.
    const NEEDS_PER_DOC_SCORES: bool = true;

    /// Aggregates the score combiner with the given scorer.
    ///
    /// The `ScoreCombiner` may decide to call `.scorer.score()`
    /// or not.
    fn update<TScorer: Scorer>(&mut self, scorer: &mut TScorer);

    /// Clears the score combiner state back to its initial state.
    fn clear(&mut self);

    /// Returns the aggregate score.
    fn score(&self) -> Score;
}

/// Just ignores scores. The `DoNothingCombiner` does not
/// even call the scorers `.score()` function.
///
/// It is useful to optimize the case when scoring is disabled.
#[derive(Default, Clone, Copy)] //< these should not be too much work :)
pub struct DoNothingCombiner;

impl ScoreCombiner for DoNothingCombiner {
    const NEEDS_PER_DOC_SCORES: bool = false;

    fn update<TScorer: Scorer>(&mut self, _scorer: &mut TScorer) {}

    fn clear(&mut self) {}

    #[inline]
    fn score(&self) -> Score {
        1.0
    }
}

/// Sums the score of different scorers.
///
/// Child scores accumulate in f64 and convert to f32 once when read. This
/// avoids intermediate f32 rounding changing ordinary ties with visit order.
#[derive(Default, Clone, Copy)]
pub struct SumCombiner {
    score: f64,
}

impl ScoreCombiner for SumCombiner {
    const SUPPORTS_BLOCK_WAND: bool = true;
    const SUMS_IN_F64: bool = true;

    fn update<TScorer: Scorer>(&mut self, scorer: &mut TScorer) {
        self.score += f64::from(scorer.score());
    }

    fn clear(&mut self) {
        self.score = 0.0;
    }

    #[inline]
    fn score(&self) -> Score {
        self.score as Score
    }
}

/// Bounds differently ordered f64 sums of nonnegative score contributions.
/// Document scores never use this error allowance.
pub(crate) struct ScoreSumUpperBound {
    factor: f64,
}

impl ScoreSumUpperBound {
    pub(crate) fn new(num_terms: usize) -> Self {
        if num_terms <= 2 {
            // Two-operand addition is commutative, so there is no order error.
            return Self { factor: 1.0 };
        }
        // For k additions, Higham's gamma_k bounds recursive summation's
        // relative error by k*u/(1-k*u), with unit roundoff u=2^-53.
        // If one order yields S, another is at most S*(1+gamma)/(1-gamma).
        // Round every bound-construction operation outward. Use the number
        // of original leaves even when prefixes/suffixes are grouped.
        let k = ((num_terms - 1) as f64).next_up();
        let rho = (k * (f64::EPSILON / 2.0)).next_up();
        let gamma = (rho / (1.0 - rho).next_down()).next_up();
        let factor = if rho >= 1.0 || gamma >= 1.0 {
            f64::INFINITY
        } else {
            ((1.0 + gamma).next_up() / (1.0 - gamma).next_down()).next_up()
        };
        Self { factor }
    }

    /// Multiplier applied to a sum before rounding (1.0 for up to two terms).
    pub(crate) fn factor(&self) -> f64 {
        self.factor
    }

    /// Precomputed form of `self.score(sum) <= threshold` for finite nonnegative
    /// sums; recompute it whenever the threshold changes.
    pub(crate) fn cutoff(&self, threshold: Score) -> ScoreSumThresholdCutoff {
        if self.factor == 1.0 {
            // `score` is then the plain f32 rounding of the sum (no successor).
            ScoreSumThresholdCutoff::for_rounding(threshold)
        } else {
            ScoreSumThresholdCutoff::new(threshold)
        }
    }

    /// Exactly `self.score(sum) <= threshold` for the threshold of `cutoff`.
    #[inline]
    pub(crate) fn is_at_most(
        &self,
        sum: f64,
        threshold: Score,
        cutoff: &ScoreSumThresholdCutoff,
    ) -> bool {
        if !(sum.is_finite() && sum >= 0.0) || self.factor.is_infinite() {
            return self.score(sum) <= threshold;
        }
        // sum == 0 scores 0 in both modes; 0 * factor = 0 compares the same way.
        sum * self.factor <= cutoff.0
    }

    #[inline]
    pub(crate) fn score(&self, sum: f64) -> Score {
        if !sum.is_finite() || sum < 0.0 {
            return Score::INFINITY;
        }
        if sum == 0.0 || self.factor == 1.0 {
            return sum as Score;
        }
        // Float conversion is monotone: bound the double sum first, then
        // compare the rounded bound to the rounded document threshold.
        (sum * self.factor).next_up() as Score
    }
}

/// The same rounded bound for an admitted 3..=32-term nonnegative score sum.
/// Keep the generic helper for signed, exceptional, and unrestricted callers.
#[derive(Clone, Copy)]
pub(crate) struct SmallScoreSumUpperBound {
    factor: f64,
}

/// An exact cutoff for an admitted small bound's post-multiply product.
/// The private field prevents confusing this with a widened f32 threshold.
pub(crate) struct ScoreSumThresholdCutoff(f64);

impl ScoreSumThresholdCutoff {
    /// Exact post-multiply cutoff for `score(sum) <= threshold`. The midpoint
    /// belongs to the lower f32 only when its significand is even. Account for
    /// that tie and the f64 bit successor already used by `score`.
    pub(crate) fn new(threshold: Score) -> Self {
        if threshold < 0.0 {
            return Self(f64::NEG_INFINITY);
        }
        if threshold == Score::INFINITY {
            return Self(f64::INFINITY);
        }
        if threshold.is_nan() {
            return Self(f64::NAN);
        }
        // Normalize -0; its ordered comparisons agree with +0. MAX's next
        // finite rounding endpoint is 2^128, not f32 infinity.
        let bits = threshold.to_bits() & 0x7fff_ffff;
        let upper = if bits == Score::MAX.to_bits() {
            f64::from_bits(0x47f0_0000_0000_0000)
        } else {
            f64::from(Score::from_bits(bits + 1))
        };
        let midpoint = (f64::from(Score::from_bits(bits)) + upper) * 0.5;
        Self(f64::from_bits(midpoint.to_bits() - 1 - u64::from(bits & 1)))
    }

    /// Exact cutoff for `(x as Score) <= threshold` with nonnegative `x`: the
    /// same midpoint tie rule as `new`, without its f64 successor step.
    pub(crate) fn for_rounding(threshold: Score) -> Self {
        if threshold < 0.0 {
            return Self(f64::NEG_INFINITY);
        }
        if threshold == Score::INFINITY {
            return Self(f64::INFINITY);
        }
        if threshold.is_nan() {
            return Self(f64::NAN);
        }
        let bits = threshold.to_bits() & 0x7fff_ffff;
        let upper = if bits == Score::MAX.to_bits() {
            f64::from_bits(0x47f0_0000_0000_0000)
        } else {
            f64::from(Score::from_bits(bits + 1))
        };
        let midpoint = (f64::from(Score::from_bits(bits)) + upper) * 0.5;
        // At the midpoint, round-to-nearest-even keeps `threshold` when its
        // significand is even.
        Self(f64::from_bits(midpoint.to_bits() - u64::from(bits & 1)))
    }
}

impl SmallScoreSumUpperBound {
    pub(crate) fn new(num_terms: usize) -> Option<Self> {
        if !(3..=32).contains(&num_terms) {
            return None;
        }
        let factor = ScoreSumUpperBound::new(num_terms).factor;
        debug_assert!(factor.is_finite() && factor > 1.0 && factor < 2.0);
        Some(Self { factor })
    }

    /// `sum` must be finite, nonnegative, and at most 32 * f32::MAX. The admitted
    /// term caller establishes this domain before any cursor movement.
    #[inline]
    pub(crate) fn score(&self, sum: f64) -> Score {
        debug_assert!(sum.is_finite() && sum >= 0.0 && sum <= 32.0 * f64::from(Score::MAX));
        let inflated = sum * self.factor;
        // Positive finite products use exactly the next_up bit successor.
        // For either signed zero, this instead produces the corresponding
        // signed least subnormal f64; its f32 cast preserves the zero bits.
        f64::from_bits(inflated.to_bits() + 1) as Score
    }

    /// The same admitted domain and multiplication as `score`; only the
    /// successor, f32 cast, and threshold comparison have moved to the cutoff.
    #[inline]
    pub(crate) fn score_is_at_most(&self, sum: f64, cutoff: &ScoreSumThresholdCutoff) -> bool {
        debug_assert!(sum.is_finite() && sum >= 0.0 && sum <= 32.0 * f64::from(Score::MAX));
        (sum * self.factor) <= cutoff.0
    }

    /// The largest `x >= 0` with `score_is_at_most(x + rest, cutoff)`, or -1
    /// when there is none: by monotonicity, a nonnegative sum `known`
    /// satisfies `score_is_at_most(known + rest, cutoff)` exactly when
    /// `known <= max_rejected_sum(rest, cutoff)`. The same domain as
    /// `score_is_at_most` applies to `x + rest` up to `32 * f32::MAX`.
    pub(crate) fn max_rejected_sum(&self, rest: f64, cutoff: &ScoreSumThresholdCutoff) -> f64 {
        let limit = 32.0 * f64::from(Score::MAX);
        let rejected = |bits: u64| self.score_is_at_most(f64::from_bits(bits) + rest, cutoff);
        let max_bits = limit.to_bits();
        if !rejected(0) {
            return -1.0;
        }
        if rejected(max_bits) {
            return limit;
        }
        let estimate = cutoff.0 / self.factor - rest;
        let guess = if estimate.is_finite() && estimate > 0.0 {
            estimate.to_bits().min(max_bits - 1)
        } else {
            0
        };
        let (mut low, mut high);
        if rejected(guess) {
            low = guess;
            let mut step = 1u64;
            loop {
                let probe = low.saturating_add(step).min(max_bits);
                if !rejected(probe) {
                    high = probe;
                    break;
                }
                low = probe;
                step = step.saturating_mul(2);
            }
        } else {
            high = guess;
            let mut step = 1u64;
            loop {
                let probe = high.saturating_sub(step);
                if rejected(probe) {
                    low = probe;
                    break;
                }
                high = probe;
                step = step.saturating_mul(2);
            }
        }
        while high - low > 1 {
            let middle = low + (high - low) / 2;
            if rejected(middle) {
                low = middle;
            } else {
                high = middle;
            }
        }
        f64::from_bits(low)
    }

    /// The largest finite leaf `x >= 0` with
    /// `score_is_at_most(f64::from(x) * copies + rest, cutoff)`, or -1 when no
    /// leaf qualifies. The f64 conversion and the small integer multiple are
    /// exact, and rounded addition and multiplication are monotone, so the
    /// predicate holds exactly for the leaves at most the result; nonnegative
    /// f32 bits order like values. `rest` must be a finite nonnegative sum of
    /// admitted f32 values, at most `num_terms - copies` of them.
    pub(crate) fn max_rejected_leaf(
        &self,
        copies: f64,
        rest: f64,
        cutoff: &ScoreSumThresholdCutoff,
    ) -> Score {
        let rejected = |bits: u32| {
            self.score_is_at_most(f64::from(Score::from_bits(bits)) * copies + rest, cutoff)
        };
        let max_bits = Score::MAX.to_bits();
        if !rejected(0) {
            return -1.0;
        }
        if rejected(max_bits) {
            return Score::MAX;
        }
        // Invariant: rejected(low) and !rejected(high). Start from the
        // unrounded solution and widen geometrically; it is usually within
        // a few bits of the answer.
        let estimate = ((cutoff.0 / self.factor - rest) / copies) as Score;
        let guess = if estimate.is_finite() && estimate > 0.0 {
            estimate.to_bits().min(max_bits - 1)
        } else {
            0
        };
        let (mut low, mut high);
        if rejected(guess) {
            low = guess;
            let mut step = 1u32;
            loop {
                let probe = low.saturating_add(step).min(max_bits);
                if !rejected(probe) {
                    high = probe;
                    break;
                }
                low = probe;
                step = step.saturating_mul(2);
            }
        } else {
            high = guess;
            let mut step = 1u32;
            loop {
                let probe = high.saturating_sub(step);
                if rejected(probe) {
                    low = probe;
                    break;
                }
                high = probe;
                step = step.saturating_mul(2);
            }
        }
        while high - low > 1 {
            let middle = low + (high - low) / 2;
            if rejected(middle) {
                low = middle;
            } else {
                high = middle;
            }
        }
        Score::from_bits(low)
    }
}

#[cfg(test)]
mod cutoff_tests {
    use super::*;

    #[test]
    fn max_rejected_sum_splits_the_sum_predicate() {
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for terms in [3usize, 6, 32] {
            let bound = SmallScoreSumUpperBound::new(terms).unwrap();
            for round in 0..4000 {
                let threshold = match round {
                    0 => -1.0,
                    1 => 0.0,
                    2 => f32::MAX,
                    _ => f32::from_bits(next() as u32 % 0x4400_0000),
                };
                let rest = if round % 5 == 0 {
                    0.0
                } else {
                    f64::from(f32::from_bits(next() as u32 % 0x4300_0000)) * (next() % 4) as f64
                };
                let cutoff = ScoreSumThresholdCutoff::new(threshold);
                let rejected = |x: f64| bound.score_is_at_most(x + rest, &cutoff);
                let floor = bound.max_rejected_sum(rest, &cutoff);
                if floor < 0.0 {
                    assert!(!rejected(0.0));
                    continue;
                }
                assert!(rejected(floor));
                assert!(!rejected(f64::from_bits(floor.to_bits() + 1)));
                for _ in 0..8 {
                    let near =
                        f64::from_bits(floor.to_bits().wrapping_add(next() % 9).wrapping_sub(4));
                    let far = f64::from(f32::from_bits(next() as u32 % 0x4400_0000));
                    for known in [near, far] {
                        if known >= 0.0 && known < 1e39 {
                            assert_eq!(rejected(known), known <= floor);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn max_rejected_leaf_splits_the_leaf_predicate() {
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let thresholds = [0.0f32, -1.0, 1e-40, f32::MIN_POSITIVE, 1.0, 7.25, f32::MAX];
        let rests = [0.0f64, 1e-40, 0.5, 3.0, 1e30];
        for terms in [3usize, 4, 17, 32] {
            let bound = SmallScoreSumUpperBound::new(terms).unwrap();
            for round in 0..4000 {
                let threshold = if round < thresholds.len() {
                    thresholds[round]
                } else {
                    f32::from_bits(next() as u32 % 0x4400_0000)
                };
                let rest = if round < rests.len() {
                    rests[round]
                } else {
                    let count = (next() % terms as u64) as usize;
                    (0..count)
                        .map(|_| f64::from(f32::from_bits(next() as u32 % 0x4200_0000)))
                        .sum::<f64>()
                };
                let cutoff = ScoreSumThresholdCutoff::new(threshold);
                let copies = f64::from(1 + (round % 3) as u32);
                let rejected =
                    |x: f32| bound.score_is_at_most(f64::from(x) * copies + rest, &cutoff);
                let floor = bound.max_rejected_leaf(copies, rest, &cutoff);
                if floor < 0.0 {
                    assert!(!rejected(0.0));
                } else {
                    assert!(rejected(floor));
                    if floor < f32::MAX {
                        assert!(!rejected(f32::from_bits(floor.to_bits() + 1)));
                    }
                }
                for _ in 0..8 {
                    let leaf = f32::from_bits(next() as u32 % 0x4400_0000);
                    assert_eq!(
                        rejected(leaf),
                        leaf <= floor,
                        "{terms} {threshold} {rest} {leaf}"
                    );
                }
            }
        }
    }

    #[test]
    fn is_at_most_matches_score_comparison() {
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let special_t = [
            0.0f32,
            -0.0,
            -1.0,
            f32::MIN_POSITIVE,
            1e-40,
            1.0,
            3.5,
            f32::MAX,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
        ];
        for terms in [1usize, 2, 3, 5, 32] {
            let bound = ScoreSumUpperBound::new(terms);
            for round in 0..3000 {
                let threshold = if round < special_t.len() {
                    special_t[round]
                } else {
                    f32::from_bits(next() as u32 & 0x7fff_ffff)
                };
                if threshold.is_nan() && round >= special_t.len() {
                    continue;
                }
                let cutoff = bound.cutoff(threshold);
                let t = f64::from(threshold);
                let mut sums = vec![
                    0.0f64,
                    -0.0,
                    -1.0,
                    f64::INFINITY,
                    f64::NAN,
                    f64::from(f32::MAX) * 2.0,
                ];
                if t.is_finite() {
                    for c in [t, t / bound.factor()] {
                        let mut x = c;
                        for _ in 0..6 {
                            sums.push(x);
                            x = x.next_down();
                        }
                        let mut x = c;
                        for _ in 0..6 {
                            sums.push(x);
                            x = x.next_up();
                        }
                        // Midpoints to the neighbouring f32 values, where ties round.
                        let up = f64::from(threshold.next_up());
                        let mid = (t + up) * 0.5;
                        sums.extend([mid, mid.next_down(), mid.next_up()]);
                    }
                }
                sums.push(f64::from_bits(next() >> 2));
                for sum in sums {
                    assert_eq!(
                        bound.is_at_most(sum, threshold, &cutoff),
                        bound.score(sum) <= threshold,
                        "terms={terms} threshold={threshold:e} sum={sum:e}"
                    );
                }
            }
        }
    }
}

/// Take max score of different scorers
/// and optionally sum it with other matches multiplied by `tie_breaker`
#[derive(Default, Clone, Copy)]
pub struct DisjunctionMaxCombiner {
    max: Score,
    sum: Score,
    tie_breaker: Score,
}

impl DisjunctionMaxCombiner {
    /// Creates `DisjunctionMaxCombiner` with tie breaker
    pub fn with_tie_breaker(tie_breaker: Score) -> DisjunctionMaxCombiner {
        DisjunctionMaxCombiner {
            max: 0.0,
            sum: 0.0,
            tie_breaker,
        }
    }
}

impl ScoreCombiner for DisjunctionMaxCombiner {
    fn update<TScorer: Scorer>(&mut self, scorer: &mut TScorer) {
        let score = scorer.score();
        self.max = Score::max(score, self.max);
        self.sum += score;
    }

    fn clear(&mut self) {
        self.max = 0.0;
        self.sum = 0.0;
    }

    #[inline]
    fn score(&self) -> Score {
        self.max + (self.sum - self.max) * self.tie_breaker
    }
}

#[cfg(test)]
mod tests {
    use super::{ScoreSumThresholdCutoff, ScoreSumUpperBound, SmallScoreSumUpperBound};

    fn cutoff_threshold_bits() -> Vec<u32> {
        let mut bits = vec![
            0,
            1,
            2,
            3,
            0x007f_fffe,
            0x007f_ffff,
            0x0080_0000,
            0x0080_0001,
            0x3f7f_ffff,
            0x3f80_0000,
            0x3f80_0001,
            0x7f7f_fffd,
            0x7f7f_fffe,
            0x7f7f_ffff,
        ];
        for exponent in 1..255u32 {
            let boundary = exponent << 23;
            bits.extend([boundary - 1, boundary, boundary + 1]);
        }
        bits.sort_unstable();
        bits.dedup();
        bits
    }

    fn check_threshold_cutoff(small: &SmallScoreSumUpperBound, sum: f64, threshold: f32) {
        let cutoff = ScoreSumThresholdCutoff::new(threshold);
        assert_eq!(
            small.score_is_at_most(sum, &cutoff),
            small.score(sum) <= threshold,
            "sum_bits={:#018x} threshold_bits={:#010x} factor_bits={:#018x}",
            sum.to_bits(),
            threshold.to_bits(),
            small.factor.to_bits()
        );
    }

    #[test]
    fn small_bound_cutoff_preserves_all_threshold_classes_and_signed_zeros() {
        let exceptional_bits = [
            0x8000_0000,
            0x8000_0001,
            0xbf80_0000,
            0xff7f_ffff,
            0xff80_0000,
            0x7f80_0000,
            0x7f80_0001,
            0x7fbf_ffff,
            0x7fc0_0000,
            0x7fff_ffff,
            0xff80_0001,
            0xffbf_ffff,
            0xffc0_0000,
            0xffff_ffff,
        ];
        let thresholds: Vec<_> = cutoff_threshold_bits()
            .into_iter()
            .chain(exceptional_bits)
            .map(f32::from_bits)
            .collect();
        for count in 3..=32 {
            let small = SmallScoreSumUpperBound::new(count).unwrap();
            assert_eq!(
                small.factor.to_bits(),
                ScoreSumUpperBound::new(count).factor.to_bits()
            );
            for &threshold in &thresholds {
                for sum in [
                    0.0,
                    -0.0,
                    f64::from_bits(1),
                    f64::from_bits(2),
                    f64::MIN_POSITIVE.next_down(),
                    f64::MIN_POSITIVE,
                    f64::from(f32::from_bits(1)),
                    1.0,
                    f64::from(f32::MAX),
                    32.0 * f64::from(f32::MAX),
                ] {
                    check_threshold_cutoff(&small, sum, threshold);
                }
                let cutoff = ScoreSumThresholdCutoff::new(threshold).0;
                if threshold.is_nan() {
                    assert!(cutoff.is_nan());
                } else if threshold < 0.0 {
                    assert_eq!(cutoff, f64::NEG_INFINITY);
                } else if threshold == f32::INFINITY {
                    assert_eq!(cutoff, f64::INFINITY);
                } else {
                    assert!(cutoff.is_finite() && cutoff > 0.0);
                }
            }
        }
        assert_eq!(
            ScoreSumThresholdCutoff::new(-0.0).0.to_bits(),
            ScoreSumThresholdCutoff::new(0.0).0.to_bits()
        );
        // The production MAX endpoint literal denotes exactly 2^128.
        assert_eq!(f64::from_bits(0x47f0_0000_0000_0000), 2.0f64.powi(128));
    }

    #[test]
    fn small_bound_cutoff_direct_products_witness_even_odd_midpoints_and_overflow() {
        let mut saw_even = false;
        let mut saw_odd = false;
        for bits in cutoff_threshold_bits() {
            let threshold = f32::from_bits(bits);
            // Independently construct the midpoint from the upward spacing;
            // the old successor-and-cast remains the decision oracle.
            let midpoint = if threshold == f32::MAX {
                f64::from(threshold) + 2.0f64.powi(103)
            } else {
                f64::from(threshold) + (f64::from(threshold.next_up()) - f64::from(threshold)) * 0.5
            };
            let cutoff = ScoreSumThresholdCutoff::new(threshold).0;
            let pred1 = midpoint.next_down();
            let pred2 = pred1.next_down();
            let next1 = midpoint.next_up();
            let next2 = next1.next_up();
            for product in [0.0, -0.0, pred2, pred1, midpoint, next1, next2] {
                let old_score = f64::from_bits(product.to_bits() + 1) as f32;
                assert_eq!(
                    product <= cutoff,
                    old_score <= threshold,
                    "threshold_bits={bits:#010x} product_bits={:#018x}",
                    product.to_bits()
                );
            }
            let even = bits & 1 == 0;
            assert!(pred2 <= cutoff);
            assert_eq!(pred1 <= cutoff, even);
            assert!(midpoint > cutoff);
            assert_eq!(
                cutoff.to_bits(),
                if even {
                    pred1.to_bits()
                } else {
                    pred2.to_bits()
                }
            );
            assert_eq!(
                (midpoint as f32).to_bits(),
                if even {
                    bits
                } else {
                    threshold.next_up().to_bits()
                }
            );
            saw_even |= even;
            saw_odd |= !even;
        }
        assert!(saw_even && saw_odd);
    }

    #[test]
    fn small_bound_cutoff_matches_rounded_products_near_every_exponent_boundary() {
        for count in 3..=32 {
            let small = SmallScoreSumUpperBound::new(count).unwrap();
            for bits in cutoff_threshold_bits() {
                let threshold = f32::from_bits(bits);
                let cutoff = ScoreSumThresholdCutoff::new(threshold);
                // Division only generates test inputs. Production retains the
                // original rounded multiplication before its comparison.
                let center = cutoff.0 / small.factor;
                let mut below = center;
                let mut above = center;
                let mut saw_true = false;
                let mut saw_false = false;
                for _ in 0..=8 {
                    for sum in [below, above] {
                        check_threshold_cutoff(&small, sum, threshold);
                        if small.score(sum) <= threshold {
                            saw_true = true;
                        } else {
                            saw_false = true;
                        }
                    }
                    below = below.next_down();
                    above = above.next_up();
                }
                assert!(
                    saw_true && saw_false,
                    "count={count} threshold_bits={bits:#010x}"
                );
            }
        }
    }

    #[test]
    fn small_bound_cutoff_matches_fixed_seed_valid_raw_bit_sums() {
        let max_sum = 32.0 * f64::from(f32::MAX);
        for count in 3..=32 {
            let small = SmallScoreSumUpperBound::new(count).unwrap();
            let mut state = 0x81ef_74c9_a32d_105bu64;
            for _ in 0..2048 {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let sum = f64::from_bits(state % (max_sum.to_bits() + 1));
                assert!(sum.is_finite() && sum >= 0.0 && sum <= max_sum);
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let threshold = f32::from_bits(u32::try_from(state >> 32).unwrap());
                check_threshold_cutoff(&small, sum, threshold);
                let old_score = small.score(sum);
                for neighboring_threshold in [old_score.next_down(), old_score, old_score.next_up()]
                {
                    check_threshold_cutoff(&small, sum, neighboring_threshold);
                }
            }
        }
    }

    fn check_small_bound(generic: &ScoreSumUpperBound, small: &SmallScoreSumUpperBound, sum: f64) {
        assert_eq!(
            small.score(sum).to_bits(),
            generic.score(sum).to_bits(),
            "sum={sum:?} sum_bits={:#018x}",
            sum.to_bits()
        );
    }

    #[test]
    fn small_score_sum_bound_matches_generic_at_cast_boundaries() {
        for unsupported in [0, 1, 2, 33, usize::MAX] {
            assert!(SmallScoreSumUpperBound::new(unsupported).is_none());
        }
        let leaf_bits = [
            0,
            1,
            2,
            0x007f_fffe,
            0x007f_ffff,
            0x0080_0000,
            0x0080_0001,
            0x3f7f_ffff,
            0x3f80_0000,
            0x3f80_0001,
            0x7f7f_fffe,
            0x7f7f_ffff,
        ];
        for count in 3..=32 {
            let generic = ScoreSumUpperBound::new(count);
            let small = SmallScoreSumUpperBound::new(count).unwrap();
            assert_eq!(small.factor.to_bits(), generic.factor.to_bits());
            // Distinguish both signs: numeric equality alone would miss drift.
            check_small_bound(&generic, &small, 0.0);
            check_small_bound(&generic, &small, -0.0);
            assert_eq!(small.score(0.0).to_bits(), 0.0f32.to_bits());
            assert_eq!(small.score(-0.0).to_bits(), (-0.0f32).to_bits());
            check_small_bound(
                &generic,
                &small,
                f64::from(u32::try_from(count).unwrap()) * f64::from(f32::MAX),
            );
            for bits in leaf_bits {
                let leaf = f32::from_bits(bits);
                check_small_bound(&generic, &small, f64::from(leaf));
                // The final finite-to-infinite f32 cast boundary has the same
                // half-ULP tie as the adjacent finite representable values.
                let midpoint = if leaf == f32::MAX {
                    f64::from(leaf) + 2.0f64.powi(103)
                } else {
                    (f64::from(leaf) + f64::from(leaf.next_up())) * 0.5
                };
                // Reverse the unchanged factor to put products around each
                // f32 midpoint, then probe neighboring representable inputs.
                // The first midpoint also checks f32 cast underflow to zero.
                let input = midpoint / generic.factor;
                let mut below = input;
                let mut above = input;
                for _ in 0..4 {
                    below = below.next_down();
                    above = above.next_up();
                }
                for sum in [below, input.next_down(), input, input.next_up(), above] {
                    check_small_bound(&generic, &small, sum);
                }
                // Also prove both sides of the cast boundary were exercised,
                // including zero/subnormal and maximum/infinity transitions.
                assert_eq!(small.score(below).to_bits(), leaf.to_bits());
                assert_eq!(small.score(above).to_bits(), leaf.next_up().to_bits());
            }
            // Exceptional/signed values keep the generic conservative route.
            for sum in [-1.0, f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
                assert_eq!(generic.score(sum), f32::INFINITY);
            }
        }
        assert_eq!(
            ScoreSumUpperBound::new(2).score(3.0).to_bits(),
            3.0f32.to_bits()
        );
    }

    #[test]
    fn small_score_sum_bound_matches_generic_for_disjoint_leaf_sums() {
        for count in 3..=32 {
            let generic = ScoreSumUpperBound::new(count);
            let small = SmallScoreSumUpperBound::new(count).unwrap();
            for seed in 0..256u64 {
                let mut state = seed + 1;
                let leaves: Vec<_> = (0..count)
                    .map(|_| {
                        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                        let bits = u32::try_from(state >> 32).unwrap() % 0x7f80_0000;
                        f32::from_bits(bits)
                    })
                    .collect();
                let mut prefix = 0.0f64;
                // Every split mirrors disjoint known leaves plus a remaining
                // inclusive prefix, retaining the original count's allowance.
                for split in 0..=count {
                    let mut known = 0.0f64;
                    for &leaf in leaves[split..].iter().rev() {
                        known += f64::from(leaf);
                    }
                    check_small_bound(&generic, &small, prefix);
                    check_small_bound(&generic, &small, known + prefix);
                    if split < count {
                        prefix += f64::from(leaves[split]);
                    }
                }
            }
        }
        // Adversarial incoming orders straddle a normal f32 tie even though
        // every leaf is itself exactly representable in f32.
        let leaves = [1.0f32, 2.0f32.powi(-24), 2.0f32.powi(-53), 2.0f32.powi(-53)];
        let generic = ScoreSumUpperBound::new(leaves.len());
        let small = SmallScoreSumUpperBound::new(leaves.len()).unwrap();
        for order in [[0, 1, 2, 3], [2, 3, 1, 0], [1, 0, 3, 2]] {
            let mut sum = 0.0f64;
            for index in order {
                sum += f64::from(leaves[index]);
            }
            check_small_bound(&generic, &small, sum);
        }
    }

    #[test]
    fn score_sum_bound_covers_double_order_error_at_float_midpoint() {
        let leaves = [1.0f32, 2.0f32.powi(-24), 2.0f32.powi(-53), 2.0f32.powi(-53)];
        let sum = |order: [usize; 4]| {
            let mut result = 0.0f64;
            for i in order {
                result += f64::from(leaves[i]);
            }
            result
        };
        let lower = sum([0, 1, 2, 3]);
        let higher = sum([2, 3, 1, 0]);
        assert_eq!(lower as f32, 1.0);
        assert_eq!(higher as f32, 1.0f32.next_up());
        let bound = ScoreSumUpperBound::new(4);
        assert!(bound.score(lower) >= higher as f32);
        for threshold in [1.0f32.next_down(), 1.0, 1.0f32.next_up()] {
            if higher as f32 > threshold {
                assert!(bound.score(lower) > threshold);
            }
        }
        // Prefix/suffix grouping represents four original leaves, not two.
        let grouped = (f64::from(leaves[0]) + f64::from(leaves[1]))
            + (f64::from(leaves[2]) + f64::from(leaves[3]));
        assert!(bound.score(grouped) >= higher as f32);
    }

    #[test]
    fn score_sum_bound_handles_signed_pointwise_replacements_and_extremes() {
        let bound = ScoreSumUpperBound::new(4);
        for leaves in [
            [f32::MAX, -f32::MAX, 1.0, f32::from_bits(1)],
            [-1.0, 1.0, f32::from_bits(1), f32::from_bits(1)],
            [0.0; 4],
        ] {
            let mut actual = 0.0f64;
            let mut nonnegative = 0.0f64;
            for leaf in leaves {
                actual += f64::from(leaf);
                nonnegative += f64::from(leaf.max(0.0));
            }
            assert!(bound.score(nonnegative) >= actual as f32);
        }
        assert_eq!(bound.score(f64::INFINITY), f32::INFINITY);
        assert_eq!(bound.score(f64::NAN), f32::INFINITY);
        assert_eq!(ScoreSumUpperBound::new(2).score(3.0), 3.0);
    }
}
