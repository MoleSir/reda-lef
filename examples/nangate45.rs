use reda_lef::Lef;

fn main() {
    let lef = Lef::load("./bench/NangateOpenCellLibrary.tech.lef").unwrap();
    println!("{:?}", lef.version);
}