use std::fmt;
use std::str::FromStr;

#[derive(GraphQLEnum, Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes, EnumIterator)]
#[graphql(name = "Project", description = "Project type")]
pub enum Project {
    MarketPlace,
    Wallet,
}

impl FromStr for Project {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "marketplace" => Ok(Project::MarketPlace),
            "wallet" => Ok(Project::Wallet),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Project::MarketPlace => write!(f, "marketplace"),
            Project::Wallet => write!(f, "wallet"),
        }
    }
}

impl Default for Project {
    fn default() -> Project {
        Project::MarketPlace
    }
}

impl Project {
    pub fn as_vec() -> Vec<Project> {
        Project::enum_iter().collect()
    }
}
