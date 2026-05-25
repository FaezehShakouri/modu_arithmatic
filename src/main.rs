use modular_arithmatic::BigNumber;

fn main() {
    let a = BigNumber::new([42, 0, 0, 0]);
    let b = BigNumber::new([100, 0, 0, 0]);

    println!("raw_add: {:?}", a.raw_add(&b));
    println!("add_mod: {:?}", a.add_mod(&b));
    println!("raw_sub: {:?}", a.raw_sub(&b));
    println!("mod_sub: {:?}", a.mod_sub(&b));
}
