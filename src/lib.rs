use ark_ff::{BigInt, BigInteger, PrimeField};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BigNumber {
    pub limbs: [u64; 4],
}

impl BigNumber {
    pub const PRIME: Self = Self::new([
        0x43e1f593f0000001,
        0x2833e84879b97091,
        0xb85045b68181585d,
        0x30644e72e131a029,
    ]);

    pub const fn new(limbs: [u64; 4]) -> Self {
        Self { limbs }
    }

    /// Limb-wise integer addition (no modular reduction).
    pub fn raw_add(&self, other: &Self) -> (Self, u64) {
        let mut result = Self::new([0; 4]);
        let mut carry: u64 = 0;

        for i in 0..4 {
            let (sum0, overflow0) = self.limbs[i].overflowing_add(other.limbs[i]);
            let (sum1, overflow1) = sum0.overflowing_add(carry);

            result.limbs[i] = sum1;
            carry = if overflow0 || overflow1 { 1 } else { 0 };
        }

        (result, carry)
    }

    /// `(a + b) mod p` for values already reduced below `p`.
    pub fn add_mod(&self, other: &Self) -> Self {
        let (mut sum, _) = self.raw_add(other);

        if sum.gte(&Self::PRIME) {
            let (reduced, _) = sum.raw_sub(&Self::PRIME);
            sum = reduced;
        }

        sum
    }

    pub fn raw_sub(&self, other: &Self) -> (Self, u64) {
        let mut result = Self::new([0; 4]);
        let mut borrow: u64 = 0;

        for i in 0..4 {
            let (sub0, overflow0) = self.limbs[i].overflowing_sub(other.limbs[i]);
            let (sub1, overflow1) = sub0.overflowing_sub(borrow);

            result.limbs[i] = sub1;
            borrow = if overflow0 || overflow1 { 1 } else { 0 };
        }

        (result, borrow)
    }

    /// `(a - b) mod p`.
    pub fn mod_sub(&self, other: &Self) -> Self {
        let (diff, borrow) = self.raw_sub(other);

        if borrow == 0 {
            diff
        } else {
            let (wrapped, _) = diff.raw_add(&Self::PRIME);
            wrapped
        }
    }

    /// Returns true when `self >= other` (limb order: 3 down to 0).
    pub fn gte(&self, other: &Self) -> bool {
        for i in (0..4).rev() {
            if self.limbs[i] > other.limbs[i] {
                return true;
            }
            if self.limbs[i] < other.limbs[i] {
                return false;
            }
        }
        true
    }

    pub fn modulo_reduction(&self) -> Self {
        let mut result = *self;

        while result.gte(&Self::PRIME) {
            let (reduced, _) = result.raw_sub(&Self::PRIME);
            result = reduced;
        }

        result
    }

    pub fn to_ark_bigint(&self) -> BigInt<4> {
        BigInt(self.limbs)
    }

    pub fn from_ark_bigint(value: BigInt<4>) -> Self {
        Self::new(value.0)
    }

    pub fn ark_bigint_add(&self, other: &Self) -> (Self, u64) {
        let mut sum = self.to_ark_bigint();
        let carry = sum.add_with_carry(&other.to_ark_bigint());
        (Self::from_ark_bigint(sum), carry as u64)
    }

    pub fn ark_bigint_sub(&self, other: &Self) -> (Self, u64) {
        let mut diff = self.to_ark_bigint();
        let borrow = diff.sub_with_borrow(&other.to_ark_bigint());
        (Self::from_ark_bigint(diff), borrow as u64)
    }

    /// Reference modular addition via `ark_bn254::Fr`.
    pub fn ark_fr_add(&self, other: &Self) -> Option<ark_bn254::Fr> {
        let a = ark_bn254::Fr::from_bigint(self.to_ark_bigint())?;
        let b = ark_bn254::Fr::from_bigint(other.to_ark_bigint())?;
        Some(a + b)
    }

    /// Reference modular subtraction via `ark_bn254::Fr`.
    pub fn ark_fr_sub(&self, other: &Self) -> Option<ark_bn254::Fr> {
        let a = ark_bn254::Fr::from_bigint(self.to_ark_bigint())?;
        let b = ark_bn254::Fr::from_bigint(other.to_ark_bigint())?;
        Some(a - b)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_ff::One;

    fn fr_to_bignum(value: Fr) -> BigNumber {
        BigNumber::from_ark_bigint(value.into_bigint())
    }

    fn assert_valid_field_element(n: BigNumber) {
        assert!(
            Fr::from_bigint(n.to_ark_bigint()).is_some(),
            "test value must be < field modulus, got {:?}",
            n
        );
    }

    fn assert_eq_mod(a: BigNumber, b: BigNumber) {
        assert_eq!(a, b, "expected {:?}, got {:?}", b, a);
    }

    #[test]
    fn raw_add_matches_ark_bigint() {
        let cases = [
            (BigNumber::PRIME, BigNumber::new([1, 0, 0, 0])),
            (BigNumber::new([u64::MAX, 0, 0, 0]), BigNumber::new([1, 0, 0, 0])),
            (BigNumber::new([42, 0, 0, 0]), BigNumber::new([100, 0, 0, 0])),
        ];

        for (a, b) in cases {
            let (ours, our_carry) = a.raw_add(&b);
            let (ark_sum, ark_carry) = a.ark_bigint_add(&b);

            assert_eq!(ours, ark_sum, "raw_add limbs mismatch for {a:?} + {b:?}");
            assert_eq!(our_carry, ark_carry, "raw_add carry mismatch for {a:?} + {b:?}");
        }
    }

    #[test]
    fn raw_sub_matches_ark_bigint() {
        let cases = [
            (BigNumber::new([100, 0, 0, 0]), BigNumber::new([42, 0, 0, 0])),
            (BigNumber::new([42, 0, 0, 0]), BigNumber::new([100, 0, 0, 0])),
            (
                BigNumber::new([0, 1, 0, 0]),
                BigNumber::new([1, 0, 0, 0]),
            ),
        ];

        for (a, b) in cases {
            let (ours, our_borrow) = a.raw_sub(&b);
            let (ark_diff, ark_borrow) = a.ark_bigint_sub(&b);

            assert_eq!(ours, ark_diff, "raw_sub limbs mismatch for {a:?} - {b:?}");
            assert_eq!(our_borrow, ark_borrow, "raw_sub borrow mismatch for {a:?} - {b:?}");
        }
    }

    #[test]
    fn add_mod_matches_ark_fr() {
        let cases = [
            (
                BigNumber::new([42, 0, 0, 0]),
                BigNumber::new([100, 0, 0, 0]),
            ),
            (
                BigNumber::new([
                    0x43e1f593f0000000,
                    0x2833e84879b97091,
                    0xb85045b68181585d,
                    0x30644e72e131a029,
                ]),
                BigNumber::new([2, 0, 0, 0]),
            ),
            (BigNumber::new([1, 0, 0, 0]), BigNumber::new([1, 0, 0, 0])),
            (
                BigNumber::new([0, 0, 0, 0]),
                BigNumber::new([1, 0, 0, 0]),
            ),
        ];

        for (a, b) in cases {
            assert_valid_field_element(a);
            assert_valid_field_element(b);

            let ours = a.add_mod(&b);
            let expected = fr_to_bignum(a.ark_fr_add(&b).expect("field elements"));

            assert_eq_mod(ours, expected);
        }
    }

    #[test]
    fn mod_sub_matches_ark_fr() {
        let cases = [
            (
                BigNumber::new([100, 0, 0, 0]),
                BigNumber::new([42, 0, 0, 0]),
            ),
            (
                BigNumber::new([42, 0, 0, 0]),
                BigNumber::new([100, 0, 0, 0]),
            ),
            (
                BigNumber::new([1, 0, 0, 0]),
                BigNumber::new([
                    0x43e1f593f0000000,
                    0x2833e84879b97091,
                    0xb85045b68181585d,
                    0x30644e72e131a029,
                ]),
            ),
            (
                BigNumber::new([0, 0, 0, 0]),
                BigNumber::new([1, 0, 0, 0]),
            ),
        ];

        for (a, b) in cases {
            assert_valid_field_element(a);
            assert_valid_field_element(b);

            let ours = a.mod_sub(&b);
            let expected = fr_to_bignum(a.ark_fr_sub(&b).expect("field elements"));

            assert_eq_mod(ours, expected);
        }
    }

    #[test]
    fn add_mod_wraps_at_modulus() {
        let p_minus_one = BigNumber::new([
            0x43e1f593f0000000,
            0x2833e84879b97091,
            0xb85045b68181585d,
            0x30644e72e131a029,
        ]);
        let two = BigNumber::new([2, 0, 0, 0]);

        assert_eq_mod(p_minus_one.add_mod(&two), fr_to_bignum(Fr::one()));
    }

    #[test]
    fn mod_sub_borrows_via_prime() {
        let small = BigNumber::new([3, 0, 0, 0]);
        let large = BigNumber::new([10, 0, 0, 0]);

        assert_eq_mod(small.mod_sub(&large), fr_to_bignum(small.ark_fr_sub(&large).unwrap()));
    }
}
