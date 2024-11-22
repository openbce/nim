use libufm::{UFMConfig, UFMError};

pub async fn run(conf: UFMConfig) -> Result<(), UFMError> {
    let ufm = libufm::connect(conf)?;
    let config = ufm.get_configuration().await?;

    println!("subnet prefix  : {}", config.subnet_prefix);
    println!("m_key          : {}", config.m_key);
    println!("m_key_per_port : {}", config.m_key_per_port);
    println!("sm_key         : {}", config.sm_key);
    println!("sa_key         : {}", config.sa_key);
    println!("qos            : {}", config.qos);
    println!("log_file       : {}", config.log_file);

    Ok(())
}
