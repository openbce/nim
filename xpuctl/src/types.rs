use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BMC {
    pub name: String,
    pub vendor: String,
    pub address: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Context {
    pub username: String,
    pub password: String,

    pub bmc: Vec<BMC>,
}

impl From<&BMC> for libonm::xpu::BMC {
    fn from(bmc: &BMC) -> Self {
        libonm::xpu::BMC {
            username: bmc.username.clone().unwrap(),
            address: bmc.address.clone(),
            password: bmc.password.clone().unwrap(),
        }
    }
}
