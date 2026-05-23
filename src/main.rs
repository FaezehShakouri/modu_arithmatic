use modular_arithmatic::BigNumber;

fn main() {
    let prime = BigNumber::PRIME;
    let one = BigNumber::new([1, 0, 0, 0]);

    let ours = prime.add(&one);
    let ark = prime.ark_bigint_add(&one);

    println!("our add:  {:?}", ours);
    println!("ark bigint add: {:?}", ark);
    println!("match: {}", ours == ark);
}
