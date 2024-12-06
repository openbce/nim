use libonm::sm::{self, UFMConfig, UFMError};

pub async fn run(conf: UFMConfig) -> Result<(), UFMError> {
    let ufm = sm::connect(conf)?;
    let v = ufm.version().await?;

    println!("{}", v);

    Ok(())
}
