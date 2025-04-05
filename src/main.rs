use postcard_idl::Pidl;

// A small harness
const INPUT: &str = include_str!("../input/input-001.kdl");

fn main() {
    println!("{INPUT}");
    let parse = Pidl::parse_from_str(INPUT).unwrap();
    println!("{:#?}", parse.types);
}
