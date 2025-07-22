use anyhow::Result;
use clap::{Arg, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Command::new("barad-dur")
        .version(env!("CARGO_PKG_VERSION"))
        .args(&[Arg::new("config")
            .help("path of config file")
            .short('c')
            .long("config")
            .default_value("./config.yaml")])
        .get_matches();

    barad_dur::run(opts).await?;
    Ok(())
}
