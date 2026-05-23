use ark_ff::{BigInt, BigInteger};

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
    pub fn add(&self, other: &Self) -> Self {
        let mut result = Self::new([0; 4]);
        let mut carry: u64 = 0;

        for i in 0..4 {
            let sum_with_carry = self.limbs[i] as u128
                + other.limbs[i] as u128
                + carry as u128;

            result.limbs[i] = sum_with_carry as u64;
            carry = (sum_with_carry >> 64) as u64;
        }

        result
    }

    pub fn to_ark_bigint(&self) -> BigInt<4> {
        BigInt(self.limbs)
    }

    pub fn from_ark_bigint(value: BigInt<4>) -> Self {
        Self::new(value.0)
    }

    /// Same integer addition ark-ff uses internally on `BigInt` limbs.
    pub fn ark_bigint_add(&self, other: &Self) -> Self {
        let mut sum = self.to_ark_bigint();
        sum.add_with_carry(&other.to_ark_bigint());
        Self::from_ark_bigint(sum)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_matches_ark_bigint_prime_plus_one() {
        let prime = BigNumber::PRIME;
        let one = BigNumber::new([1, 0, 0, 0]);

        let ours = prime.add(&one);
        let ark = prime.ark_bigint_add(&one);

        assert_eq!(ours, ark, "limb add should match ark-ff BigInt::add_with_carry");
    }

    #[test]
    fn add_matches_ark_bigint_with_carry() {
        let max_limb = BigNumber::new([u64::MAX, 0, 0, 0]);
        let one = BigNumber::new([1, 0, 0, 0]);

        let ours = max_limb.add(&one);
        let ark = max_limb.ark_bigint_add(&one);

        assert_eq!(ours.limbs, [0, 1, 0, 0]);
        assert_eq!(ours, ark);
    }

}
