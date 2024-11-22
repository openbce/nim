use libufm::{UFMConfig, UFMError};

pub async fn run(conf: UFMConfig) -> Result<(), UFMError> {
    let ufm = libufm::connect(conf)?;
    let ps = ufm.list_partition().await?;

    println!(
        "{:<15}{:<10}{:<10}{:<10}{:<10}{:<10}",
        "Name", "Pkey", "IPoIB", "MTU", "Rate", "Level"
    );

    for p in ps {
        let qos = p
            .qos
            .ok_or(UFMError::InvalidConfig("no partition qos".to_string()))?;

        println!(
            "{:<15}{:<10}{:<10}{:<10}{:<10}{:<10}",
            p.name,
            p.pkey.to_string(),
            p.ipoib,
            qos.mtu_limit,
            qos.rate_limit,
            qos.service_level
        )
    }

    Ok(())
}
