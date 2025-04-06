use postcard_idl::{generate::rust_std::generate_rust_std, Pidl};

// A small harness
const INPUT: &str = include_str!("../input/input-001.kdl");

fn main() {
    println!("{INPUT}");
    let parse = Pidl::parse_from_str(INPUT).unwrap();
    println!("{:#?}", parse.types);
    let gen = generate_rust_std(&parse);
    println!("/* GENERATED */");
    println!();
    println!("{}", gen.aliases);
    println!("{}", gen.types);
}
