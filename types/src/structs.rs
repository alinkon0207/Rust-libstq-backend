use super::*;

use std::collections::HashSet;
use std::fmt::{self, Debug, Display};
use std::str::FromStr;

use stq_static_resources::{Currency, CurrencyType};
use uuid::{self, Uuid};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProductSellerPrice {
    pub price: ProductPrice,
    pub currency: Currency,
    pub discount: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CartItem {
    pub id: CartItemId,
    pub customer: CartCustomer,
    pub product_id: ProductId,
    pub quantity: Quantity,
    pub selected: bool,
    pub comment: String,
    pub store_id: StoreId,
    pub pre_order: bool,
    pub pre_order_days: i32,
    pub coupon_id: Option<CouponId>,
    pub delivery_method_id: Option<DeliveryMethodId>,
    pub currency_type: CurrencyType,
    pub user_country_code: Option<Alpha3>,
}

pub type Cart = HashSet<CartItem>;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionId(Uuid);

impl TransactionId {
    pub fn new(id: Uuid) -> Self {
        TransactionId(id)
    }

    pub fn inner(&self) -> &Uuid {
        &self.0
    }

    pub fn generate() -> Self {
        TransactionId(Uuid::new_v4())
    }

    pub fn next(&self) -> Self {
        let mut bytes = self.0.as_bytes().to_vec();
        let last = bytes.len() - 1;
        bytes[last] = bytes[last].wrapping_add(1);
        let uuid = Uuid::from_bytes(&bytes).unwrap();
        TransactionId(uuid)
    }

    pub fn prev(&self) -> Self {
        let mut bytes = self.0.as_bytes().to_vec();
        let last = bytes.len() - 1;
        bytes[last] = bytes[last].wrapping_sub(1);
        let uuid = Uuid::from_bytes(&bytes).unwrap();
        TransactionId(uuid)
    }

    pub fn last_byte(&self) -> u8 {
        let bytes = self.0.as_bytes().to_vec();
        let last = bytes.len() - 1;
        bytes[last]
    }
}

impl FromStr for TransactionId {
    type Err = uuid::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = Uuid::parse_str(s)?;
        Ok(TransactionId::new(id))
    }
}

impl Into<Uuid> for TransactionId {
    fn into(self) -> Uuid {
        self.0
    }
}

impl Debug for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Display::fmt(&self.0, f)
    }
}

impl Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("{}", self.0.hyphenated()))
    }
}
