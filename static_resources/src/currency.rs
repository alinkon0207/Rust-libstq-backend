use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::str;
use std::str::FromStr;

use diesel::expression::bound::Bound;
use diesel::expression::AsExpression;
use diesel::pg::Pg;
use diesel::row::Row;
use diesel::serialize::Output;
use diesel::sql_types::*;
use diesel::types::{FromSqlRow, IsNull, ToSql};
use diesel::Queryable;
use juniper::FieldError;

use super::CurrencyType;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIterator, GraphQLEnum)]
pub enum Currency {
    RUB,
    EUR,
    USD,
    BTC,
    ETH,
    STQ,
}

impl Currency {
    pub fn code(&self) -> &'static str {
        match self {
            Currency::RUB => "RUB",
            Currency::EUR => "EUR",
            Currency::USD => "USD",
            Currency::BTC => "BTC",
            Currency::ETH => "ETH",
            Currency::STQ => "STQ",
        }
    }

    pub fn from_code(s: &str) -> Option<Self> {
        Some(match s.to_ascii_uppercase().as_str() {
            "RUB" => Currency::RUB,
            "EUR" => Currency::EUR,
            "USD" | "USDT" => Currency::USD, // USDT - for EUR/USD exchange pair
            "BTC" => Currency::BTC,
            "ETH" => Currency::ETH,
            "STQ" => Currency::STQ,
            _ => {
                return None;
            }
        })
    }

    pub fn currency_type(&self) -> CurrencyType {
        match self {
            Currency::RUB | Currency::EUR | Currency::USD => CurrencyType::Fiat,
            Currency::BTC | Currency::ETH | Currency::STQ => CurrencyType::Crypto,
        }
    }
}

impl Display for Currency {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", self.code())
    }
}

impl FromStr for Currency {
    type Err = FieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_code(s).ok_or_else(|| {
            FieldError::new(
                "Unknown Currency",
                graphql_value!({ "code": 300, "details": {
                format!("Can not resolve Currency name. Unknown Currency: '{}'", s)
                }}),
            )
        })
    }
}

impl NotNull for Currency {}
impl SingleValue for Currency {}
impl Queryable<VarChar, Pg> for Currency {
    type Row = Currency;
    fn build(row: Self::Row) -> Self {
        row
    }
}
impl AsExpression<VarChar> for Currency {
    type Expression = Bound<VarChar, Currency>;
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}
impl<'a> AsExpression<VarChar> for &'a Currency {
    type Expression = Bound<VarChar, &'a Currency>;
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}
impl ToSql<VarChar, Pg> for Currency {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> Result<IsNull, Box<Error + Send + Sync>> {
        out.write_all(self.code().as_bytes())?;
        Ok(IsNull::No)
    }
}
impl FromSqlRow<VarChar, Pg> for Currency {
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>> {
        match row.take() {
            Some(v) => {
                let s = str::from_utf8(v).unwrap_or("unreadable value");
                Self::from_code(s).ok_or_else(|| format!("Unrecognized enum variant: {:?}", s).into())
            }
            None => Err("Unexpected null for non-null column".into()),
        }
    }
}
