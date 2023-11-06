use postgres;
use postgres::types::{FromSql, IsNull, ToSql, Type};
use postgres_protocol::types::{text_from_sql, text_to_sql};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(GraphQLEnum, Deserialize, Serialize, Debug, Clone, Copy, PartialEq, DieselTypes, EnumIterator)]
#[graphql(name = "CommitterRole", description = "Order committer role")]
pub enum CommitterRole {
    #[graphql(description = "System role")]
    #[serde(rename = "system")]
    System,

    #[graphql(description = "Customer")]
    #[serde(rename = "customer")]
    Customer,

    #[graphql(description = "Seller")]
    #[serde(rename = "seller")]
    Seller,
}

impl FromStr for CommitterRole {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "system" => CommitterRole::System,
            "customer" => CommitterRole::Customer,
            "seller" => CommitterRole::Seller,
            other => {
                return Err(format!("Unrecognized enum variant: {}", other).to_string().into());
            }
        })
    }
}

impl Display for CommitterRole {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::CommitterRole::*;

        write!(
            f,
            "{}",
            match self {
                System => "system",
                Customer => "customer",
                Seller => "seller",
            }
        )
    }
}

impl ToSql for CommitterRole {
    to_sql_checked!();

    fn to_sql(&self, _ty: &Type, out: &mut Vec<u8>) -> Result<IsNull, Box<Error + Sync + Send>> {
        use self::CommitterRole::*;

        text_to_sql(
            match self {
                System => "system",
                Customer => "customer",
                Seller => "seller",
            },
            out,
        );
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        <&str as ToSql>::accepts(ty)
    }
}

impl<'a> FromSql<'a> for CommitterRole {
    fn from_sql(_: &Type, raw: &'a [u8]) -> Result<Self, Box<Error + Sync + Send>> {
        use self::CommitterRole::*;

        text_from_sql(raw).and_then(|buf| {
            Ok(match buf {
                "system" => System,
                "customer" => Customer,
                "seller" => Seller,
                other => {
                    return Err(Box::new(postgres::error::conversion(
                        format!("Unknown CommitterRole variant: {}", other).into(),
                    )));
                }
            })
        })
    }

    fn accepts(ty: &Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }
}
