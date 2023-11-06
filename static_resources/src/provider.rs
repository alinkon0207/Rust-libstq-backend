use std::fmt;
use std::str::FromStr;

#[derive(GraphQLEnum, Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes, EnumIterator)]
pub enum Provider {
    #[graphql(description = "Email")]
    Email,
    #[graphql(description = "Facebook")]
    Facebook,
    #[graphql(description = "Google")]
    Google,
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Provider::Facebook => write!(f, "facebook"),
            Provider::Google => write!(f, "google"),
            Provider::Email => write!(f, "email"),
        }
    }
}

impl FromStr for Provider {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "facebook" => Ok(Provider::Facebook),
            "google" => Ok(Provider::Google),
            "email" => Ok(Provider::Email),
            _ => Err(()),
        }
    }
}
