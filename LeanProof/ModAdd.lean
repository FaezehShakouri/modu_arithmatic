namespace LeanProof.ModAdd

/-!
This file models the final carry returned by `BigNumber::raw_add`.

For a 4-limb `u64` integer, the final carry is the high bit beyond the
256-bit value range. Mathematically, that carry is:

  `(a + b) / 2^256`

when `a` and `b` are interpreted as natural numbers below `2^256`.

In `add_mod`, both inputs are intended to be valid BN254 field elements:

  `a < PRIME` and `b < PRIME`

The theorem below proves that under that precondition the carry returned by
`raw_add` is always zero.
-/

def WordBits : Nat := 64
def NumLimbs : Nat := 4
def Base : Nat := 2 ^ WordBits
def TwoPow256 : Nat := Base ^ NumLimbs

def Prime : Nat :=
  21888242871839275222246405745257275088548364400416034343698204186575808495617

def IsFieldElement (x : Nat) : Prop :=
  x < Prime

/- The final carry of unsigned 256-bit addition. -/
def rawAddCarry (a b : Nat) : Nat :=
  (a + b) / TwoPow256

/- BN254's scalar modulus is small enough that adding two reduced elements
   still fits inside 256 bits. -/
theorem two_prime_lt_two_pow_256 : Prime + Prime < TwoPow256 := by
  native_decide

theorem raw_add_carry_zero_for_field_elements
    {a b : Nat}
    (ha : IsFieldElement a)
    (hb : IsFieldElement b) :
    rawAddCarry a b = 0 := by
  unfold rawAddCarry IsFieldElement at *
  apply Nat.div_eq_of_lt
  exact Nat.lt_trans (Nat.add_lt_add ha hb) two_prime_lt_two_pow_256

/- Same theorem, named after the Rust function where this fact is used. -/
theorem carry_returned_from_raw_add_in_add_mod_is_zero
    {a b : Nat}
    (ha : IsFieldElement a)
    (hb : IsFieldElement b) :
    rawAddCarry a b = 0 :=
  raw_add_carry_zero_for_field_elements ha hb

end LeanProof.ModAdd
