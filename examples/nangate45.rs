use reda_lef::{LefLayer, LefTechnology};

fn main() {
    if let Err(e) = main_result() {
        eprint!("Error: >>{}<<", e);
    }
}

fn main_result() -> Result<(), Box<dyn std::error::Error>> {
    let lef = LefTechnology::load("./bench/NangateOpenCellLibrary.tech.lef")?;
    println!("{:?}", lef.version);
    println!("{:?}", lef.busbitchars);
    println!("{:?}", lef.dividerchar);
    for layer in lef.layers {
        match layer {
            LefLayer::Routing(layer) => {
                println!("{:#?}", layer);
            }
            _ => {}
        }
    }
    Ok(())
}