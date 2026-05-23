use modular_arithmatic::BigNumber;

fn main() {
    let prime = BigNumber::PRIME;
    let one = BigNumber::new([1, 0, 0, 0]);

    let (ours, carry) = prime.add(&one);
    let ark = prime.ark_bigint_add(&one);

    println!("our add:  {:?}", ours);
    println!("carry: {}", carry);
    println!("ark bigint add: {:?}", ark);
}
