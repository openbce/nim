use libufm::{Partition, PartitionKey, PortConfig, PortMembership, UFMConfig, UFMError};

pub struct CreateOptions {
    pub pkey: String,
    pub ipoib: bool,
    pub index0: bool,
    pub membership: String,
    pub guids: Vec<String>,
}

pub async fn run(conf: UFMConfig, opt: &CreateOptions) -> Result<(), UFMError> {
    let ufm = libufm::connect(conf)?;

    let mut pbs = vec![];
    for g in &opt.guids {
        pbs.push(PortConfig {
            guid: g.to_string(),
            index0: opt.index0,
            membership: PortMembership::try_from(opt.membership.clone())?,
        })
    }

    let p = Partition {
        name: "".to_string(),
        pkey: PartitionKey::try_from(opt.pkey.clone())?,
        ipoib: opt.ipoib,
        qos: None,
    };

    ufm.bind_ports(p, pbs).await?;

    Ok(())
}
