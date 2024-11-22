use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum PortType {
    Physical,
    Virtual,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Port {
    pub guid: String,
    pub name: Option<String>,
    pub system_id: String,
    pub lid: i32,
    pub system_name: String,
    pub logical_state: String,
    pub parent_guid: Option<String>,
    pub port_type: Option<PortType>,
}

impl Default for Port {
    fn default() -> Self {
        Self {
            guid: "".to_string(),
            name: None,
            system_id: "".to_string(),
            lid: 65535,
            system_name: "".to_string(),
            logical_state: "Unknown".to_string(),
            parent_guid: None,
            port_type: None,
        }
    }
}

impl From<PhysicalPort> for Port {
    fn from(physicalport: PhysicalPort) -> Self {
        Port {
            guid: physicalport.guid.clone(),
            name: Some(physicalport.name),
            system_id: physicalport.system_id,
            lid: physicalport.lid,
            system_name: physicalport.system_name,
            logical_state: physicalport.logical_state,
            parent_guid: None,
            port_type: Some(PortType::Physical),
        }
    }
}

impl From<VirtualPort> for Port {
    fn from(virtualport: VirtualPort) -> Self {
        Port {
            guid: virtualport.virtual_port_guid.clone(),
            name: None,
            system_id: virtualport.system_guid,
            lid: virtualport.virtual_port_lid,
            system_name: virtualport.system_name,
            logical_state: virtualport.virtual_port_state,
            parent_guid: Some(virtualport.port_guid),
            port_type: Some(PortType::Virtual),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PhysicalPort {
    pub guid: String,
    pub name: String,
    #[serde(rename = "systemID")]
    pub system_id: String,
    pub lid: i32,
    pub system_name: String,
    pub logical_state: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VirtualPort {
    pub virtual_port_guid: String,
    pub system_guid: String,
    pub virtual_port_lid: i32,
    pub system_name: String,
    pub virtual_port_state: String,
    pub port_guid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub subnet_prefix: String,
    pub m_key: String,
    pub m_key_per_port: bool,
    pub sm_key: String,
    pub sa_key: String,
    pub log_file: String,
    pub qos: i32,
}
