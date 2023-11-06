use std::fmt;
use std::str::FromStr;

#[derive(GraphQLEnum, Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes, EnumIterator)]
#[graphql(name = "Device", description = "Device type")]
pub enum Device {
    IOS,
    WEB,
    Android,
}

impl FromStr for Device {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ios" => Ok(Device::IOS),
            "web" => Ok(Device::WEB),
            "android" => Ok(Device::Android),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Device::IOS => write!(f, "ios"),
            Device::WEB => write!(f, "web"),
            Device::Android => write!(f, "android"),
        }
    }
}

impl Device {
    pub fn as_vec() -> Vec<Device> {
        Device::enum_iter().collect()
    }
}
