use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes, EnumIterator)]
pub enum TokenType {
    EmailVerify,
    PasswordReset,
    Undefined,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TokenType::EmailVerify => write!(f, "email_verify"),
            TokenType::PasswordReset => write!(f, "password_reset"),
            TokenType::Undefined => write!(f, "undefined"),
        }
    }
}

impl FromStr for TokenType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "email_verify" => Ok(TokenType::EmailVerify),
            "password_reset" => Ok(TokenType::PasswordReset),
            _ => Ok(TokenType::Undefined),
        }
    }
}
