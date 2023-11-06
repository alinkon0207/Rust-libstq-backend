use postgres;
use postgres::types::{FromSql, IsNull, ToSql, Type};
use postgres_protocol::types::{text_from_sql, text_to_sql};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(GraphQLEnum, Deserialize, Serialize, Debug, Clone, Copy, PartialEq, DieselTypes, EnumIterator)]
#[graphql(name = "OrderState", description = "Current order status")]
pub enum OrderState {
    #[graphql(description = "State set on order creation.")]
    #[serde(rename = "new")]
    New,

    #[graphql(description = "State set on order wallet creation.")]
    #[serde(rename = "payment_awaited")]
    PaymentAwaited,

    #[graphql(description = "State set on user's transaction appeared in blockchain, but is not included.")]
    #[serde(rename = "transaction_pending")]
    TransactionPending,

    #[graphql(description = "Set after price timeout has passed. Amount recalculation needed.")]
    #[serde(rename = "amount_expired")]
    AmountExpired,

    #[graphql(description = "Set after payment is accepted by blockchain by request of billing")]
    #[serde(rename = "paid")]
    Paid,

    #[graphql(description = "Order is being processed by store management")]
    #[serde(rename = "in_processing")]
    InProcessing,

    #[graphql(description = "Can be cancelled by any party before order being sent.")]
    #[serde(rename = "cancelled")]
    Cancelled,

    #[graphql(description = "Wares are on their way to the customer. Tracking ID must be set.")]
    #[serde(rename = "sent")]
    Sent,

    #[graphql(description = "Wares are delivered to the customer.")]
    #[serde(rename = "delivered")]
    Delivered,

    #[graphql(description = "Wares are received by the customer.")]
    #[serde(rename = "received")]
    Received,

    #[graphql(description = "The customer opened a dispute")]
    #[serde(rename = "dispute")]
    Dispute,

    #[graphql(description = "Order is complete.")]
    #[serde(rename = "complete")]
    Complete,
}

impl FromStr for OrderState {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "New" => OrderState::New,
            "Payment Awaited" => OrderState::PaymentAwaited,
            "Transaction pending" => OrderState::TransactionPending,
            "Amount expired" => OrderState::AmountExpired,
            "Paid" => OrderState::Paid,
            "In processing" => OrderState::InProcessing,
            "Cancelled" => OrderState::Cancelled,
            "Sent" => OrderState::Sent,
            "Delivered" => OrderState::Delivered,
            "Received" => OrderState::Received,
            "Dispute" => OrderState::Dispute,
            "Complete" => OrderState::Complete,
            other => {
                return Err(format!("Unrecognized enum variant: {}", other).to_string().into());
            }
        })
    }
}

impl Display for OrderState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::OrderState::*;

        write!(
            f,
            "{}",
            match self {
                New => "New",
                PaymentAwaited => "Payment Awaited",
                TransactionPending => "Transaction pending",
                AmountExpired => "Amount expired",
                Paid => "Paid",
                InProcessing => "In processing",
                Cancelled => "Cancelled",
                Sent => "Sent",
                Delivered => "Delivered",
                Received => "Received",
                Dispute => "Dispute",
                Complete => "Complete",
            }
        )
    }
}

impl ToSql for OrderState {
    to_sql_checked!();

    fn to_sql(&self, _ty: &Type, out: &mut Vec<u8>) -> Result<IsNull, Box<Error + Sync + Send>> {
        use self::OrderState::*;

        text_to_sql(
            match self {
                New => "new",
                PaymentAwaited => "payment_awaited",
                TransactionPending => "transaction_pending",
                AmountExpired => "amount_expired",
                Paid => "paid",
                InProcessing => "in_processing",
                Cancelled => "cancelled",
                Sent => "sent",
                Delivered => "delivered",
                Received => "received",
                Dispute => "dispute",
                Complete => "complete",
            },
            out,
        );
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        <&str as ToSql>::accepts(ty)
    }
}

impl<'a> FromSql<'a> for OrderState {
    fn from_sql(_: &Type, raw: &'a [u8]) -> Result<Self, Box<Error + Sync + Send>> {
        use self::OrderState::*;

        text_from_sql(raw).and_then(|buf| {
            Ok(match buf {
                "new" => New,
                "payment_awaited" => PaymentAwaited,
                "transaction_pending" => TransactionPending,
                "amount_expired" => AmountExpired,
                "paid" => Paid,
                "in_processing" => InProcessing,
                "cancelled" => Cancelled,
                "sent" => Sent,
                "delivered" => Delivered,
                "received" => Received,
                "dispute" => Dispute,
                "complete" => Complete,
                other => {
                    return Err(Box::new(postgres::error::conversion(
                        format!("Unknown OrderState variant: {}", other).into(),
                    )));
                }
            })
        })
    }

    fn accepts(ty: &Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }
}
