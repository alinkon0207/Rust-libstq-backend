use std::fmt;
use std::str::FromStr;

#[derive(GraphQLEnum, Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes, EnumIterator)]
pub enum Gender {
    #[graphql(description = "Male")]
    Male,
    #[graphql(description = "Female")]
    Female,
    #[graphql(description = "Undefined")]
    Undefined,
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Gender::Male => write!(f, "male"),
            Gender::Female => write!(f, "female"),
            Gender::Undefined => write!(f, "undefined"),
        }
    }
}

impl FromStr for Gender {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "male" => Ok(Gender::Male),
            "female" => Ok(Gender::Female),
            "undefined" => Ok(Gender::Undefined),
            _ => Err(()),
        }
    }
}
