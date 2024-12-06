use libonm::sm::{self, UFMConfig, UFMError};

pub async fn run(conf: UFMConfig, pkey: &str) -> Result<(), UFMError> {
    let ufm = sm::connect(conf)?;
    ufm.delete_partition(pkey).await?;

    Ok(())
}
