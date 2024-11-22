use libufm::{UFMConfig, UFMError};

pub async fn run(conf: UFMConfig) -> Result<(), UFMError> {
    let ufm = libufm::connect(conf)?;
    let v = ufm.version().await?;

    println!("{}", v);

    Ok(())
}
