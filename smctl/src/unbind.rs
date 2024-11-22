use libufm::{PartitionKey, UFMConfig, UFMError};

pub async fn run(conf: UFMConfig, pkey: &str, guids: &[String]) -> Result<(), UFMError> {
    let ufm = libufm::connect(conf)?;

    let p = PartitionKey::try_from(pkey.to_owned())?;

    ufm.unbind_ports(p, guids.to_owned()).await?;

    Ok(())
}
