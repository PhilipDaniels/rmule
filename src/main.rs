use anyhow::Result;

mod configuration;

fn main() -> Result<()> {
    let config_dir = configuration::get_configuration_directory()?;
    let mule_configuration = configuration::read_mule_configuration(&config_dir)?;
    println!("{0:#?}", mule_configuration);
    Ok(())
}

