use std::error::Error;
use std::fmt;
use std::str::FromStr;

use postgres;
use postgres::types::{FromSql, IsNull, ToSql, Type};
use postgres_protocol::types::{text_from_sql, text_to_sql};

#[derive(GraphQLEnum, Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes, EnumIterator)]
pub enum CurrencyType {
    #[graphql(description = "Crypto")]
    Crypto,
    #[graphql(description = "Fiat")]
    Fiat,
}

impl fmt::Display for CurrencyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CurrencyType::Crypto => write!(f, "crypto"),
            CurrencyType::Fiat => write!(f, "fiat"),
        }
    }
}

impl FromStr for CurrencyType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "crypto" => Ok(CurrencyType::Crypto),
            "fiat" => Ok(CurrencyType::Fiat),
            _ => Err(()),
        }
    }
}

impl ToSql for CurrencyType {
    to_sql_checked!();

    fn to_sql(&self, _ty: &Type, out: &mut Vec<u8>) -> Result<IsNull, Box<Error + Sync + Send>> {
        use self::CurrencyType::*;

        text_to_sql(
            match self {
                Crypto => "crypto",
                Fiat => "fiat",
            },
            out,
        );
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        <&str as ToSql>::accepts(ty)
    }
}

impl<'a> FromSql<'a> for CurrencyType {
    fn from_sql(_: &Type, raw: &'a [u8]) -> Result<Self, Box<Error + Sync + Send>> {
        use self::CurrencyType::*;

        text_from_sql(raw).and_then(|buf| {
            Ok(match buf {
                "crypto" => Crypto,
                "fiat" => Fiat,
                other => {
                    return Err(Box::new(postgres::error::conversion(
                        format!("Unknown CurrencyType variant: {}", other).into(),
                    )));
                }
            })
        })
    }

    fn accepts(ty: &Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }
}
