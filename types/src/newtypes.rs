use uuid::Uuid;

macro_rules! f64_newtype {
    ($x:ident) => {
        #[derive(Clone, Copy, Debug, Display, Default, PartialEq, PartialOrd, From, FromStr, Into, Serialize, Deserialize, DieselTypes)]
        pub struct $x(pub f64);
    };
}
macro_rules! i32_newtype {
    ($x:ident) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            Display,
            Default,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            From,
            FromStr,
            Into,
            Hash,
            Serialize,
            Deserialize,
            DieselTypes,
        )]
        pub struct $x(pub i32);
    };
}
macro_rules! string_newtype {
    ($x:ident) => {
        #[derive(
            Clone, Debug, Display, Default, PartialEq, Eq, PartialOrd, Ord, From, FromStr, Into, Hash, Serialize, Deserialize, DieselTypes,
        )]
        pub struct $x(pub String);

        impl std::convert::AsRef<str> for $x {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}
macro_rules! uuid_newtype {
    ($x:ident) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            Default,
            Display,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            From,
            FromStr,
            Into,
            Hash,
            Serialize,
            Deserialize,
            DieselTypes,
        )]
        pub struct $x(pub Uuid);

        impl $x {
            pub fn new() -> Self {
                $x(Uuid::new_v4())
            }
        }
    };
}

i32_newtype!(UserId);
i32_newtype!(SessionId);
i32_newtype!(ProductId);
i32_newtype!(BaseProductId);
i32_newtype!(Quantity);
i32_newtype!(StoreId);
i32_newtype!(OrderSlug);
i32_newtype!(CompanyPackageId);
i32_newtype!(CompanyId);
i32_newtype!(PackageId);
i32_newtype!(CustomAttributeId);
i32_newtype!(AttributeId);
i32_newtype!(CategoryId);
i32_newtype!(CouponId);
i32_newtype!(PickupId);
i32_newtype!(ShippingId);
i32_newtype!(ShippingRatesId);
i32_newtype!(ProdAttrId);
i32_newtype!(AttributeValueId);
i32_newtype!(EmarsysId);
i32_newtype!(StoreBillingTypeId);
i32_newtype!(InternationalBillingId);
i32_newtype!(RussiaBillingId);
i32_newtype!(ProxyCompanyBillingInfoId);
i32_newtype!(StoreSubscriptionId);
i32_newtype!(SubscriptionId);
i32_newtype!(SubscriptionPaymentId);

string_newtype!(WarehouseSlug);
string_newtype!(CountryLabel);
string_newtype!(PageSlug);
string_newtype!(Alpha2);
string_newtype!(Alpha3);
string_newtype!(AttributeValueCode);
string_newtype!(CouponCode);
string_newtype!(BaseProductSlug);
string_newtype!(StoreSlug);
string_newtype!(CategorySlug);
string_newtype!(SwiftId);

pub mod stripe {
    string_newtype!(SourceId);
    string_newtype!(PaymentIntentId);
    string_newtype!(ChargeId);
}

uuid_newtype!(RoleEntryId);
uuid_newtype!(RoleId);
uuid_newtype!(StockId);
uuid_newtype!(InvoiceId);
uuid_newtype!(SagaId);
uuid_newtype!(MerchantId);
uuid_newtype!(CartItemId);
uuid_newtype!(OrderId);
uuid_newtype!(OrderDiffId);
uuid_newtype!(OrderInfoId);
uuid_newtype!(CallbackId);
uuid_newtype!(ConversionId);
uuid_newtype!(WarehouseId);
uuid_newtype!(CurrencyExchangeId);
uuid_newtype!(PageId);
uuid_newtype!(PayoutId);

f64_newtype!(ProductPrice);
f64_newtype!(ExchangeRate);
f64_newtype!(CashbackPercent);
