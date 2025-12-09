use reda_lef::LefTechnology;

fn main() {
    if let Err(e) = main_result() {
        eprint!("Error: >>{}<<", e);
    }
}

fn main_result() -> Result<(), Box<dyn std::error::Error>> {
    let _ = LefTechnology::load_file("./bench/NangateOpenCellLibrary.tech.lef")?;
    Ok(())
}