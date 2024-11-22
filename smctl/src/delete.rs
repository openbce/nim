use libufm::{UFMConfig, UFMError};

pub async fn run(conf: UFMConfig, pkey: &str) -> Result<(), UFMError> {
    let ufm = libufm::connect(conf)?;
    ufm.delete_partition(pkey).await?;

    Ok(())
}
