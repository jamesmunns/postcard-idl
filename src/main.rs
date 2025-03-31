use pidl2::Pidl;

// A small harness
const INPUT: &str = include_str!("../input/input-001.pidl");

fn main() {
    println!("{INPUT}");
    let _parse = Pidl::parse_from_str(INPUT).unwrap();
}
