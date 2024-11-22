use libufm::{Partition, PartitionKey, PortConfig, PortMembership, UFMConfig, UFMError};

pub async fn run(conf: UFMConfig, pkey: &str, guids: &Vec<String>) -> Result<(), UFMError> {
    let ufm = libufm::connect(conf)?;

    let mut pbs = vec![];
    for g in guids {
        pbs.push(PortConfig {
            guid: g.to_string(),
            index0: true,
            membership: PortMembership::Full,
        })
    }

    let p = Partition {
        name: "".to_string(),
        pkey: PartitionKey::try_from(pkey.to_owned())?,
        ipoib: false,
        qos: None,
    };

    ufm.bind_ports(p, pbs).await?;

    Ok(())
}
