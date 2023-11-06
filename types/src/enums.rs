use super::*;

use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes)]
pub enum StoresRole {
    Superuser,
    User,
    Moderator,
    PlatformAdmin,
}

impl FromStr for StoresRole {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "superuser" => Ok(StoresRole::Superuser),
            "user" => Ok(StoresRole::User),
            "moderator" => Ok(StoresRole::Moderator),
            "platform_admin" => Ok(StoresRole::PlatformAdmin),
            _ => Err(()),
        }
    }
}

impl fmt::Display for StoresRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StoresRole::Superuser => write!(f, "superuser"),
            StoresRole::User => write!(f, "user"),
            StoresRole::Moderator => write!(f, "moderator"),
            StoresRole::PlatformAdmin => write!(f, "platform_admin"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes)]
pub enum UsersRole {
    Superuser,
    User,
    Moderator,
}

impl FromStr for UsersRole {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "superuser" => Ok(UsersRole::Superuser),
            "user" => Ok(UsersRole::User),
            "moderator" => Ok(UsersRole::Moderator),
            _ => Err(()),
        }
    }
}

impl fmt::Display for UsersRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UsersRole::Superuser => write!(f, "superuser"),
            UsersRole::User => write!(f, "user"),
            UsersRole::Moderator => write!(f, "moderator"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes)]
pub enum BillingRole {
    Superuser,
    User,
    StoreManager,
    FinancialManager,
}

impl FromStr for BillingRole {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "superuser" => Ok(BillingRole::Superuser),
            "user" => Ok(BillingRole::User),
            "store_manager" => Ok(BillingRole::StoreManager),
            "financial_manager" => Ok(BillingRole::FinancialManager),
            _ => Err(()),
        }
    }
}

impl fmt::Display for BillingRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BillingRole::Superuser => write!(f, "superuser"),
            BillingRole::User => write!(f, "user"),
            BillingRole::StoreManager => write!(f, "store_manager"),
            BillingRole::FinancialManager => write!(f, "financial_manager"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes)]
pub enum DeliveryRole {
    Superuser,
    User,
    StoreManager,
}

impl FromStr for DeliveryRole {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "superuser" => Ok(DeliveryRole::Superuser),
            "user" => Ok(DeliveryRole::User),
            "store_manager" => Ok(DeliveryRole::StoreManager),
            _ => Err(()),
        }
    }
}

impl fmt::Display for DeliveryRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeliveryRole::Superuser => write!(f, "superuser"),
            DeliveryRole::User => write!(f, "user"),
            DeliveryRole::StoreManager => write!(f, "store_manager"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes)]
pub enum OrderRole {
    Superuser,
    User,
    StoreManager,
}

impl FromStr for OrderRole {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "superuser" => Ok(OrderRole::Superuser),
            "user" => Ok(OrderRole::User),
            "store_manager" => Ok(OrderRole::StoreManager),
            _ => Err(()),
        }
    }
}

impl fmt::Display for OrderRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OrderRole::Superuser => write!(f, "superuser"),
            OrderRole::User => write!(f, "user"),
            OrderRole::StoreManager => write!(f, "store_manager"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes)]
pub enum WarehouseRole {
    Superuser,
    User,
    StoreManager,
}

impl FromStr for WarehouseRole {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "superuser" => Ok(WarehouseRole::Superuser),
            "user" => Ok(WarehouseRole::User),
            "store_manager" => Ok(WarehouseRole::StoreManager),
            _ => Err(()),
        }
    }
}

impl fmt::Display for WarehouseRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WarehouseRole::Superuser => write!(f, "superuser"),
            WarehouseRole::User => write!(f, "user"),
            WarehouseRole::StoreManager => write!(f, "store_manager"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes)]
pub enum MerchantType {
    Store,
    User,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, DieselTypes)]
pub enum BillingType {
    International,
    Russia,
}

#[derive(Clone, Debug, PartialEq, Eq, From, Hash)]
pub enum WarehouseIdentifier {
    Id(WarehouseId),
    Slug(WarehouseSlug),
}

/// Anything that can uniquely identify an Order
#[derive(Clone, Copy, Debug, Eq, From, PartialEq, Hash)]
pub enum OrderIdentifier {
    Id(OrderId),
    Slug(OrderSlug),
}

/// Anything that can uniquely identify a page
#[derive(Clone, Debug, Eq, From, PartialEq, Hash)]
pub enum PageIdentifier {
    Id(PageId),
    Slug(PageSlug),
}

/// Anything that can uniquely identify a store
#[derive(Clone, Debug, Eq, From, PartialEq, Hash)]
pub enum StoreIdentifier {
    Id(StoreId),
    Slug(StoreSlug),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, From, Hash, Serialize, Deserialize)]
pub enum CartCustomer {
    User(UserId),
    Anonymous(SessionId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, From, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryMethodId {
    Package { id: CompanyPackageId }, // deprecated
    Pickup { id: PickupId },
    ShippingPackage { id: ShippingId },
}

impl fmt::Display for CartCustomer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CartCustomer::*;

        write!(
            f,
            "{}",
            match self {
                User(id) => format!("user / {}", id),
                Anonymous(id) => format!("session / {}", id),
            }
        )
    }
}
