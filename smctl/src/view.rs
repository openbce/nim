use libufm::{PortType, UFMConfig, UFMError};

pub async fn run(conf: UFMConfig, pkey: &str) -> Result<(), UFMError> {
    let ufm = libufm::connect(conf)?;
    let p = ufm.get_partition(pkey).await?;
    let ps = ufm.list_port(p.pkey).await?;

    let qos = p
        .qos
        .ok_or(UFMError::InvalidConfig("no partition qos".to_string()))?;

    println!("{:15}: {}", "Name", p.name);
    println!("{:15}: {}", "Pkey", p.pkey.to_string());
    println!("{:15}: {}", "IPoIB", p.ipoib);
    println!("{:15}: {}", "MTU", qos.mtu_limit);
    println!("{:15}: {}", "Rate Limit", qos.rate_limit);
    println!("{:15}: {}", "Service Level", qos.service_level);
    println!("{:15}: ", "Ports");

    println!(
        "    {:<20}{:<20}{:<10}{:<20}{:<10}{:<10}{:<20}{:<15}",
        "GUID", "ParentGUID", "PortType", "SystemID", "LID", "LogState", "Name", "SystemName",
    );
    for port in ps {
        let name = match port.name.clone() {
            Some(n) => n,
            None => "-".to_string(),
        };
        let parent_guid = match port.parent_guid.clone() {
            Some(p) => p,
            None => "-".to_string(),
        };
        let port_type = match port.port_type {
            Some(PortType::Physical) => "pf".to_string(),
            Some(PortType::Virtual) => "vf".to_string(),
            None => "-".to_string(),
        };
        println!(
            "    {:<20}{:<20}{:<10}{:<20}{:<10}{:<10}{:<20}{:<15}",
            port.guid,
            parent_guid,
            port_type,
            port.system_id,
            port.lid,
            port.logical_state,
            name,
            port.system_name,
        )
    }

    Ok(())
}
