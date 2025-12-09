use reda_lef::LefTechnology;

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
    // for layer in lef.layers {
    //     match layer {
    //         LefLayer::Routing(layer) => {
    //             // println!("{:#?}", layer);
    //         }
    //         _ => {}
    //     }
    // }
    for (via_name, via) in lef.via_rules {
        println!("{}", via_name);
        println!("{:#?}", via);
    }
    // for (site_name, site) in lef.sites {
    //     println!("{}", site_name);
    //     println!("{:#?}", site);
    // }
    Ok(())
}