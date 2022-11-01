use anyhow::Result;

mod configuration;
mod ini;

fn main() -> Result<()> {
    let config_dir = configuration::get_configuration_directory()?;
    let mule_configuration = configuration::read_mule_configuration(&config_dir)?;
    println!("app_version={:?}", mule_configuration.app_version());
    println!("nickname={:?}", mule_configuration.nickname());
    println!("confirm_exit={:?}", mule_configuration.confirm_exit());
    Ok(())
}

