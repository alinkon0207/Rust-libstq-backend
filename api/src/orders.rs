use rpc_client::RestApiClient;
use types::*;
use util::*;

use chrono::prelude::*;
use regex::Regex;
use uuid::Uuid;

use std::collections::HashMap;
use stq_roles;
use stq_router::{Builder as RouterBuilder, Router};
use stq_static_resources::{CommitterRole, Currency, CurrencyType, OrderState};
use stq_types::*;
use validator::{Validate, ValidationError};

#[derive(Clone, Debug)]
pub enum Route {
    Cart {
        customer: CartCustomer,
    },
    CartProducts {
        customer: CartCustomer,
    },
    CartIncrementProduct {
        customer: CartCustomer,
        product_id: ProductId,
    },
    AddCartCoupon {
        customer: CartCustomer,
        product_id: ProductId,
        coupon_id: CouponId,
    },
    DeleteCartCoupon {
        customer: CartCustomer,
        coupon_id: CouponId,
    },
    DeleteCartCouponByProduct {
        customer: CartCustomer,
        product_id: ProductId,
    },
    CartProduct {
        customer: CartCustomer,
        product_id: ProductId,
    },
    CartProductDeliveryMethod {
        customer: CartCustomer,
        product_id: ProductId,
    },
    CartProductQuantity {
        customer: CartCustomer,
        product_id: ProductId,
    },
    CartProductSelection {
        customer: CartCustomer,
        product_id: ProductId,
    },
    CartProductComment {
        customer: CartCustomer,
        product_id: ProductId,
    },
    CartClear {
        customer: CartCustomer,
    },
    DeleteProductsFromAllCarts,
    DeleteDeliveryMethodFromAllCarts,
    CartMerge,
    OrderFromCart,
    OrderFromBuyNow,
    OrderFromCartRevert,
    OrderSearch,
    Orders,
    OrdersByUser {
        user: UserId,
    },
    OrdersByStore {
        store_id: StoreId,
    },
    Order {
        order_id: OrderIdentifier,
    },
    OrderDiff {
        order_id: OrderIdentifier,
    },
    OrderStatus {
        order_id: OrderIdentifier,
    },
    OrdersAllowedStatuses,
    Roles(stq_roles::routing::Route),
}

impl From<stq_roles::routing::Route> for Route {
    fn from(v: stq_roles::routing::Route) -> Self {
        Route::Roles(v)
    }
}

fn cart_customer_route(id: &CartCustomer) -> String {
    use self::CartCustomer::*;

    match id {
        User(user_id) => format!("by-user/{}", user_id),
        Anonymous(session_id) => format!("by-session/{}", session_id),
    }
}

fn order_identifier_route(id: &OrderIdentifier) -> String {
    use self::OrderIdentifier::*;

    match id {
        Id(id) => format!("by-id/{}", id),
        Slug(slug) => format!("by-slug/{}", slug),
    }
}

impl RouteBuilder for Route {
    fn route(&self) -> String {
        use self::Route::*;

        match self {
            Cart { customer } => format!("cart/{}", cart_customer_route(customer)),
            CartProducts { customer } => format!("cart/{}/products", cart_customer_route(customer)),
            CartIncrementProduct {
                customer,
                product_id,
            } => format!(
                "cart/{}/products/{}/increment",
                cart_customer_route(customer),
                product_id
            ),
            AddCartCoupon {
                customer,
                product_id,
                coupon_id,
            } => format!(
                "cart/{}/products/{}/coupon/{}",
                cart_customer_route(customer),
                product_id,
                coupon_id,
            ),
            DeleteCartCoupon {
                customer,
                coupon_id,
            } => format!(
                "cart/{}/coupons/{}",
                cart_customer_route(customer),
                coupon_id,
            ),
            DeleteCartCouponByProduct {
                customer,
                product_id,
            } => format!(
                "cart/{}/products/{}/coupons",
                cart_customer_route(customer),
                product_id,
            ),
            CartProduct {
                customer,
                product_id,
            } => format!(
                "cart/{}/products/{}",
                cart_customer_route(customer),
                product_id
            ),
            CartProductDeliveryMethod {
                customer,
                product_id,
            } => format!(
                "cart/{}/products/{}/delivery_method",
                cart_customer_route(customer),
                product_id,
            ),
            CartProductQuantity {
                customer,
                product_id,
            } => format!(
                "cart/{}/products/{}/quantity",
                cart_customer_route(customer),
                product_id
            ),
            CartProductSelection {
                customer,
                product_id,
            } => format!(
                "cart/{}/products/{}/selection",
                cart_customer_route(customer),
                product_id
            ),
            CartProductComment {
                customer,
                product_id,
            } => format!(
                "cart/{}/products/{}/comment",
                cart_customer_route(customer),
                product_id
            ),
            DeleteProductsFromAllCarts => "cart/delete-products-from-all-carts".to_string(),
            DeleteDeliveryMethodFromAllCarts => {
                "cart/delete-delivery-method-from-all-carts".to_string()
            }
            CartClear { customer } => format!("cart/{}/clear", cart_customer_route(customer)),
            CartMerge => "cart/merge".to_string(),
            OrderFromCart => "orders/create_from_cart".to_string(),
            OrderFromBuyNow => "orders/create_buy_now".to_string(),
            OrderFromCartRevert => "orders/create_from_cart/revert".to_string(),
            OrderSearch => "orders/search".to_string(),
            Orders => "orders".to_string(),
            OrdersByUser { user } => format!("orders/by-user/{}", user),
            OrdersByStore { store_id } => format!("orders/by-store/{}", store_id),
            Order { order_id } => format!("orders/{}", order_identifier_route(order_id)),
            OrderDiff { order_id } => format!("order_diffs/{}", order_identifier_route(order_id)),
            OrderStatus { order_id } => {
                format!("orders/{}/status", order_identifier_route(order_id))
            }
            OrdersAllowedStatuses => "orders/allowed_statuses".to_string(),
            Roles(route) => route.route(),
        }
    }
}

impl Route {
    pub fn from_path(s: &str) -> Option<Self> {
        lazy_static! {
            static ref ROUTER: Router<Route> =
                stq_roles::routing::add_routes(RouterBuilder::default())
                    .with_route(r"^/cart/by-user/(\d+)$", |params| params
                        .into_iter()
                        .next()
                        .and_then(|string_id| string_id.parse().ok().map(CartCustomer::User))
                        .map(|customer| Route::Cart { customer }))
                    .with_route(r"^/cart/by-session/([a-zA-Z0-9-]+)$", |params| params
                        .into_iter()
                        .next()
                        .and_then(|string_id| string_id.parse().ok().map(CartCustomer::Anonymous))
                        .map(|customer| Route::Cart { customer }))
                    .with_route(r"^/cart/by-user/(\d+)/products/(\d+)$", |params| {
                        let mut params = params.into_iter();
                        if let Some(customer_id_s) = params.next() {
                            if let Some(product_id_s) = params.next() {
                                if let Ok(customer) = customer_id_s.parse().map(CartCustomer::User)
                                {
                                    if let Ok(product_id) = product_id_s.parse().map(ProductId) {
                                        return Some(Route::CartProduct {
                                            customer,
                                            product_id,
                                        });
                                    }
                                }
                            }
                        }
                        None
                    })
                    .with_route(
                        r"^/cart/by-session/([a-zA-Z0-9-]+)/products/(\d+)$",
                        |params| {
                            let mut params = params.into_iter();
                            if let Some(customer_id_s) = params.next() {
                                if let Some(product_id_s) = params.next() {
                                    if let Ok(customer) =
                                        customer_id_s.parse().map(CartCustomer::Anonymous)
                                    {
                                        if let Ok(product_id) = product_id_s.parse().map(ProductId)
                                        {
                                            return Some(Route::CartProduct {
                                                customer,
                                                product_id,
                                            });
                                        }
                                    }
                                }
                            }
                            None
                        }
                    )
                    .with_route(
                        r"^/cart/by-user/(\d+)/products/(\d+)/increment$",
                        |params| {
                            let mut params = params.into_iter();
                            if let Some(customer_id_s) = params.next() {
                                if let Some(product_id_s) = params.next() {
                                    if let Ok(customer) =
                                        customer_id_s.parse().map(CartCustomer::User)
                                    {
                                        if let Ok(product_id) = product_id_s.parse().map(ProductId)
                                        {
                                            return Some(Route::CartIncrementProduct {
                                                customer,
                                                product_id,
                                            });
                                        }
                                    }
                                }
                            }
                            None
                        }
                    )
                    .with_route(
                        r"^/cart/by-user/(\d+)/products/(\d+)/coupon/(\d+)$",
                        |params| {
                            let mut params = params.into_iter();
                            let customer = params.next()?.parse().ok().map(CartCustomer::User)?;
                            let product_id = params.next()?.parse().ok().map(ProductId)?;
                            let coupon_id = params.next()?.parse().ok().map(CouponId)?;
                            Some(Route::AddCartCoupon {
                                customer,
                                product_id,
                                coupon_id,
                            })
                        }
                    )
                    .with_route(r"^/cart/by-user/(\d+)/coupons/(\d+)$", |params| {
                        let mut params = params.into_iter();
                        let customer = params.next()?.parse().ok().map(CartCustomer::User)?;
                        let coupon_id = params.next()?.parse().ok().map(CouponId)?;
                        Some(Route::DeleteCartCoupon {
                            customer,
                            coupon_id,
                        })
                    })
                    .with_route(r"^/cart/by-user/(\d+)/products/(\d+)/coupons$", |params| {
                        let mut params = params.into_iter();
                        let customer = params.next()?.parse().ok().map(CartCustomer::User)?;
                        let product_id = params.next()?.parse().ok().map(ProductId)?;
                        Some(Route::DeleteCartCouponByProduct {
                            customer,
                            product_id,
                        })
                    })
                    .with_route(
                        r"^/cart/by-session/([a-zA-Z0-9-]+)/products/(\d+)/coupon/(\d+)$",
                        |params| {
                            let mut params = params.into_iter();
                            let customer =
                                params.next()?.parse().ok().map(CartCustomer::Anonymous)?;
                            let product_id = params.next()?.parse().ok().map(ProductId)?;
                            let coupon_id = params.next()?.parse().ok().map(CouponId)?;
                            Some(Route::AddCartCoupon {
                                customer,
                                product_id,
                                coupon_id,
                            })
                        }
                    )
                    .with_route(
                        r"^/cart/by-session/([a-zA-Z0-9-]+)/coupons/(\d+)$",
                        |params| {
                            let mut params = params.into_iter();
                            let customer =
                                params.next()?.parse().ok().map(CartCustomer::Anonymous)?;
                            let coupon_id = params.next()?.parse().ok().map(CouponId)?;
                            Some(Route::DeleteCartCoupon {
                                customer,
                                coupon_id,
                            })
                        }
                    )
                    .with_route(
                        r"^/cart/by-session/([a-zA-Z0-9-]+)/products/(\d+)/coupons$",
                        |params| {
                            let mut params = params.into_iter();
                            let customer =
                                params.next()?.parse().ok().map(CartCustomer::Anonymous)?;
                            let product_id = params.next()?.parse().ok().map(ProductId)?;
                            Some(Route::DeleteCartCouponByProduct {
                                customer,
                                product_id,
                            })
                        }
                    )
                    .with_route(
                        r"^/cart/by-user/(\d+)/products/(\d+)/delivery_method$",
                        |params| {
                            let mut params = params.into_iter();
                            let customer = params.next()?.parse().ok().map(CartCustomer::User)?;
                            let product_id = params.next()?.parse().ok().map(ProductId)?;
                            Some(Route::CartProductDeliveryMethod {
                                customer,
                                product_id,
                            })
                        }
                    )
                    .with_route(
                        r"^/cart/by-session/([a-zA-Z0-9-]+)/products/(\d+)/delivery_method$",
                        |params| {
                            let mut params = params.into_iter();
                            let customer =
                                params.next()?.parse().ok().map(CartCustomer::Anonymous)?;
                            let product_id = params.next()?.parse().ok().map(ProductId)?;
                            Some(Route::CartProductDeliveryMethod {
                                customer,
                                product_id,
                            })
                        }
                    )
                    .with_route(
                        r"^/cart/by-session/([a-zA-Z0-9-]+)/products/(\d+)/increment$",
                        |params| {
                            let mut params = params.into_iter();
                            if let Some(customer_id_s) = params.next() {
                                if let Some(product_id_s) = params.next() {
                                    if let Ok(customer) =
                                        customer_id_s.parse().map(CartCustomer::Anonymous)
                                    {
                                        if let Ok(product_id) = product_id_s.parse().map(ProductId)
                                        {
                                            return Some(Route::CartIncrementProduct {
                                                customer,
                                                product_id,
                                            });
                                        }
                                    }
                                }
                            }
                            None
                        }
                    )
                    .with_route(r"^/cart/by-user/(\d+)/products/(\d+)/quantity$", |params| {
                        let mut params = params.into_iter();
                        if let Some(customer_id_s) = params.next() {
                            if let Some(product_id_s) = params.next() {
                                if let Ok(customer) = customer_id_s.parse().map(CartCustomer::User)
                                {
                                    if let Ok(product_id) = product_id_s.parse().map(ProductId) {
                                        return Some(Route::CartProductQuantity {
                                            customer,
                                            product_id,
                                        });
                                    }
                                }
                            }
                        }
                        None
                    })
                    .with_route(
                        r"^/cart/by-session/([a-zA-Z0-9-]+)/products/(\d+)/quantity$",
                        |params| {
                            let mut params = params.into_iter();
                            if let Some(customer_id_s) = params.next() {
                                if let Some(product_id_s) = params.next() {
                                    if let Ok(customer) =
                                        customer_id_s.parse().map(CartCustomer::Anonymous)
                                    {
                                        if let Ok(product_id) = product_id_s.parse().map(ProductId)
                                        {
                                            return Some(Route::CartProductQuantity {
                                                customer,
                                                product_id,
                                            });
                                        }
                                    }
                                }
                            }
                            None
                        }
                    )
                    .with_route(
                        r"^/cart/by-user/(\d+)/products/(\d+)/selection$",
                        |params| {
                            let mut params = params.into_iter();
                            if let Some(customer_id_s) = params.next() {
                                if let Some(product_id_s) = params.next() {
                                    if let Ok(customer) =
                                        customer_id_s.parse().map(CartCustomer::User)
                                    {
                                        if let Ok(product_id) = product_id_s.parse().map(ProductId)
                                        {
                                            return Some(Route::CartProductSelection {
                                                customer,
                                                product_id,
                                            });
                                        }
                                    }
                                }
                            }
                            None
                        }
                    )
                    .with_route(
                        r"^/cart/by-session/([a-zA-Z0-9-]+)/products/(\d+)/selection$",
                        |params| {
                            let mut params = params.into_iter();
                            if let Some(customer_id_s) = params.next() {
                                if let Some(product_id_s) = params.next() {
                                    if let Ok(customer) =
                                        customer_id_s.parse().map(CartCustomer::Anonymous)
                                    {
                                        if let Ok(product_id) = product_id_s.parse().map(ProductId)
                                        {
                                            return Some(Route::CartProductSelection {
                                                customer,
                                                product_id,
                                            });
                                        }
                                    }
                                }
                            }
                            None
                        }
                    )
                    .with_route(r"^/cart/by-user/(\d+)/products/(\d+)/comment$", |params| {
                        let mut params = params.into_iter();
                        if let Some(customer_id_s) = params.next() {
                            if let Some(product_id_s) = params.next() {
                                if let Ok(customer) = customer_id_s.parse().map(CartCustomer::User)
                                {
                                    if let Ok(product_id) = product_id_s.parse().map(ProductId) {
                                        return Some(Route::CartProductComment {
                                            customer,
                                            product_id,
                                        });
                                    }
                                }
                            }
                        }
                        None
                    })
                    .with_route(
                        r"^/cart/by-session/([a-zA-Z0-9-]+)/products/(\d+)/comment$",
                        |params| {
                            let mut params = params.into_iter();
                            if let Some(customer_id_s) = params.next() {
                                if let Some(product_id_s) = params.next() {
                                    if let Ok(customer) =
                                        customer_id_s.parse().map(CartCustomer::Anonymous)
                                    {
                                        if let Ok(product_id) = product_id_s.parse().map(ProductId)
                                        {
                                            return Some(Route::CartProductComment {
                                                customer,
                                                product_id,
                                            });
                                        }
                                    }
                                }
                            }
                            None
                        }
                    )
                    .with_route(r"^/cart/by-user/(\d+)/products$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok().map(CartCustomer::User))
                        .map(|customer| Route::CartProducts { customer }))
                    .with_route(r"^/cart/delete-products-from-all-carts$", |_| Some(
                        Route::DeleteProductsFromAllCarts
                    ))
                    .with_route(r"^/cart/delete-delivery-method-from-all-carts$", |_| Some(
                        Route::DeleteDeliveryMethodFromAllCarts
                    ))
                    .with_route(r"^/cart/by-session/([a-zA-Z0-9-]+)/products$", |params| {
                        params
                            .get(0)
                            .and_then(|string_id| {
                                string_id.parse().ok().map(CartCustomer::Anonymous)
                            })
                            .map(|customer| Route::CartProducts { customer })
                    })
                    .with_route(r"^/cart/by-user/(\d+)/clear$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok().map(CartCustomer::User))
                        .map(|customer| Route::CartClear { customer }))
                    .with_route(r"^/cart/by-session/([a-zA-Z0-9-]+)/clear$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok().map(CartCustomer::Anonymous))
                        .map(|customer| Route::CartClear { customer }))
                    .with_route(r"^/cart/merge$", |_| Some(Route::CartMerge))
                    .with_route(r"^/orders$", |_| Some(Route::Orders))
                    .with_route(r"^/orders/create_from_cart$", |_| Some(
                        Route::OrderFromCart
                    ))
                    .with_route(r"^/orders/create_buy_now$", |_| Some(
                        Route::OrderFromBuyNow
                    ))
                    .with_route(r"^/orders/create_from_cart/revert$", |_| Some(
                        Route::OrderFromCartRevert
                    ))
                    .with_route(r"^/orders/search", |_| Some(Route::OrderSearch))
                    .with_route(r"^/orders/by-store/(\d+)$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok())
                        .map(|store_id| Route::OrdersByStore { store_id }))
                    .with_route(r"^/orders/by-id/([a-zA-Z0-9-]+)$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok().map(OrderIdentifier::Id))
                        .map(|order_id| Route::Order { order_id }))
                    .with_route(r"^/orders/by-slug/(\d+)$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok().map(OrderIdentifier::Slug))
                        .map(|order_id| Route::Order { order_id }))
                    .with_route(r"^/orders/by-id/([a-zA-Z0-9-]+)/status$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok().map(OrderIdentifier::Id))
                        .map(|order_id| Route::OrderStatus { order_id }))
                    .with_route(r"^/orders/by-slug/(\d+)/status$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok().map(OrderIdentifier::Slug))
                        .map(|order_id| Route::OrderStatus { order_id }))
                    .with_route(r"^/order_diffs/by-id/([a-zA-Z0-9-]+)$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok().map(OrderIdentifier::Id))
                        .map(|order_id| Route::OrderDiff { order_id }))
                    .with_route(r"^/order_diffs/by-slug/(\d+)$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok().map(OrderIdentifier::Slug))
                        .map(|order_id| Route::OrderDiff { order_id }))
                    .build();
        }

        ROUTER.test(s)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SetterPayload<T> {
    pub value: T,
}

pub type CartProductQuantityPayload = SetterPayload<Quantity>;
pub type CartProductSelectionPayload = SetterPayload<bool>;
pub type CartProductCommentPayload = SetterPayload<String>;
pub type CartProductDeliveryMethodIdPayload = SetterPayload<DeliveryMethodId>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CartProductIncrementPayload {
    pub store_id: StoreId,
    pub pre_order: bool,
    pub pre_order_days: i32,
    pub currency_type: CurrencyType,
    pub user_country_code: Option<UserCountryCodeUpdater>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UserCountryCodeUpdater {
    Reset,
    Set { value: Alpha3 },
}

impl UserCountryCodeUpdater {
    pub fn as_option(&self) -> Option<Alpha3> {
        match *self {
            UserCountryCodeUpdater::Reset => None,
            UserCountryCodeUpdater::Set { ref value } => Some(value.clone()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CartMergePayload {
    pub from: CartCustomer,
    pub to: CartCustomer,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeleteProductsFromCartsPayload {
    pub product_ids: Vec<ProductId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeleteDeliveryMethodFromCartsPayload {
    pub product_ids: Vec<ProductId>,
}

/// Service that provides operations for interacting with user carts
pub trait CartClient {
    /// Get user's cart contents
    fn get_cart(
        &self,
        customer: CartCustomer,
        currency_type: Option<CurrencyType>,
    ) -> ApiFuture<Cart>;
    /// Increase item's quantity by 1
    fn increment_item(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
        store_id: StoreId,
        pre_order: bool,
        pre_order_days: i32,
        currency_type: CurrencyType,
        user_country_code: Option<UserCountryCodeUpdater>,
    ) -> ApiFuture<Cart>;
    /// Set item to desired quantity in user's cart
    fn set_quantity(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
        value: Quantity,
    ) -> ApiFuture<Cart>;
    /// Set selection of the item in user's cart
    fn set_selection(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
        value: bool,
    ) -> ApiFuture<Cart>;
    /// Set comment for item in user's cart
    fn set_comment(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
        value: String,
    ) -> ApiFuture<Cart>;
    /// Delete item from user's cart
    fn delete_item(&self, customer: CartCustomer, product_id: ProductId) -> ApiFuture<Cart>;
    /// Clear user's cart
    fn clear_cart(&self, customer: CartCustomer) -> ApiFuture<Cart>;
    /// Iterate over cart
    fn list(&self, customer: CartCustomer, from: ProductId, count: i32) -> ApiFuture<Cart>;
    /// Merge carts
    fn merge(
        &self,
        from: CartCustomer,
        to: CartCustomer,
        currency_type: Option<CurrencyType>,
    ) -> ApiFuture<Cart>;
    /// Add coupon
    fn add_coupon(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
        coupon_id: CouponId,
    ) -> ApiFuture<Cart>;
    /// Delete coupon
    fn delete_coupon(&self, customer: CartCustomer, coupon_id: CouponId) -> ApiFuture<Cart>;
    /// Delete coupon by product id
    fn delete_coupon_by_product(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
    ) -> ApiFuture<Cart>;
    /// Set delivery method
    fn set_delivery_method(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
        delivery_method_id: DeliveryMethodId,
    ) -> ApiFuture<Cart>;
    /// Delete delivery method by product id
    fn delete_delivery_method_by_product(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
    ) -> ApiFuture<Cart>;
}

impl CartClient for RestApiClient {
    fn get_cart(
        &self,
        customer: CartCustomer,
        currency_type: Option<CurrencyType>,
    ) -> ApiFuture<Cart> {
        let url = if let Some(currency_type) = currency_type {
            format!(
                "{}?currency_type={}",
                self.build_route(&Route::CartProducts { customer }),
                currency_type
            )
        } else {
            self.build_route(&Route::CartProducts { customer })
        };

        http_req(self.http_client.get(&url))
    }

    fn increment_item(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
        store_id: StoreId,
        pre_order: bool,
        pre_order_days: i32,
        currency_type: CurrencyType,
        user_country_code: Option<UserCountryCodeUpdater>,
    ) -> ApiFuture<Cart> {
        http_req(
            self.http_client
                .post(&self.build_route(&Route::CartIncrementProduct {
                    customer,
                    product_id,
                }))
                .body(JsonPayload(&CartProductIncrementPayload {
                    store_id,
                    pre_order,
                    pre_order_days,
                    currency_type,
                    user_country_code,
                })),
        )
    }

    fn set_quantity(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
        value: Quantity,
    ) -> ApiFuture<Cart> {
        http_req(
            self.http_client
                .put(&self.build_route(&Route::CartProductQuantity {
                    customer,
                    product_id,
                }))
                .body(JsonPayload(&CartProductQuantityPayload { value })),
        )
    }

    fn set_selection(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
        value: bool,
    ) -> ApiFuture<Cart> {
        http_req(
            self.http_client
                .put(&self.build_route(&Route::CartProductSelection {
                    customer,
                    product_id,
                }))
                .body(JsonPayload(&CartProductSelectionPayload { value })),
        )
    }

    fn set_comment(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
        value: String,
    ) -> ApiFuture<Cart> {
        http_req(
            self.http_client
                .put(&self.build_route(&Route::CartProductComment {
                    customer,
                    product_id,
                }))
                .body(JsonPayload(&CartProductCommentPayload { value })),
        )
    }

    fn delete_item(&self, customer: CartCustomer, product_id: ProductId) -> ApiFuture<Cart> {
        http_req(
            self.http_client
                .delete(&self.build_route(&Route::CartProduct {
                    customer,
                    product_id,
                })),
        )
    }

    fn clear_cart(&self, customer: CartCustomer) -> ApiFuture<Cart> {
        http_req(
            self.http_client
                .post(&self.build_route(&Route::CartClear { customer })),
        )
    }

    fn list(&self, customer: CartCustomer, from: ProductId, count: i32) -> ApiFuture<Cart> {
        http_req(self.http_client.get(&format!(
            "{}?offset={}&count={}",
            self.build_route(&Route::Cart { customer }),
            from,
            count
        )))
    }

    fn merge(
        &self,
        from: CartCustomer,
        to: CartCustomer,
        currency_type: Option<CurrencyType>,
    ) -> ApiFuture<Cart> {
        let url = if let Some(currency_type) = currency_type {
            format!(
                "{}?currency_type={}",
                self.build_route(&Route::CartMerge),
                currency_type
            )
        } else {
            self.build_route(&Route::CartMerge)
        };

        http_req(
            self.http_client
                .post(&url)
                .body(JsonPayload(&CartMergePayload { from, to })),
        )
    }

    fn add_coupon(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
        coupon_id: CouponId,
    ) -> ApiFuture<Cart> {
        http_req(
            self.http_client
                .post(&self.build_route(&Route::AddCartCoupon {
                    customer,
                    product_id,
                    coupon_id,
                })),
        )
    }

    fn delete_coupon(&self, customer: CartCustomer, coupon_id: CouponId) -> ApiFuture<Cart> {
        http_req(
            self.http_client
                .delete(&self.build_route(&Route::DeleteCartCoupon {
                    customer,
                    coupon_id,
                })),
        )
    }

    fn delete_coupon_by_product(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
    ) -> ApiFuture<Cart> {
        http_req(
            self.http_client
                .delete(&self.build_route(&Route::DeleteCartCouponByProduct {
                    customer,
                    product_id,
                })),
        )
    }

    fn set_delivery_method(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
        value: DeliveryMethodId,
    ) -> ApiFuture<Cart> {
        http_req(
            self.http_client
                .post(&self.build_route(&Route::CartProductDeliveryMethod {
                    customer,
                    product_id,
                }))
                .body(JsonPayload(&CartProductDeliveryMethodIdPayload { value })),
        )
    }

    fn delete_delivery_method_by_product(
        &self,
        customer: CartCustomer,
        product_id: ProductId,
    ) -> ApiFuture<Cart> {
        http_req(
            self.http_client
                .delete(&self.build_route(&Route::CartProductDeliveryMethod {
                    customer,
                    product_id,
                })),
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AddressFull {
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub country: Option<String>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub address: Option<String>,
    pub place_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub created_from: CartItemId,
    pub conversion_id: ConversionId,
    pub slug: OrderSlug,
    pub customer: UserId,
    pub store: StoreId,
    pub product: ProductId,
    pub price: ProductPrice,
    pub currency: Currency,
    pub quantity: Quantity,
    pub address: AddressFull,
    pub receiver_name: String,
    pub receiver_phone: String,
    pub receiver_email: String,
    pub state: OrderState,
    pub payment_status: bool,
    pub delivery_company: Option<String>,
    pub track_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pre_order: bool,
    pub pre_order_days: i32,
    pub coupon_id: Option<CouponId>,
    pub coupon_percent: Option<i32>,
    pub coupon_discount: Option<ProductPrice>,
    pub product_discount: Option<ProductPrice>,
    pub total_amount: ProductPrice,
    pub company_package_id: Option<CompanyPackageId>,
    pub delivery_price: f64,
    pub shipping_id: Option<ShippingId>,
    pub product_cashback: Option<CashbackPercent>,
    pub currency_type: CurrencyType,
}

pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    lazy_static! {
        static ref PHONE_VALIDATION_RE: Regex = Regex::new(r"^\+?\d{7}\d*$").unwrap();
    }

    if PHONE_VALIDATION_RE.is_match(phone) {
        Ok(())
    } else {
        Err(ValidationError {
            code: "phone".into(),
            message: Some("Incorrect phone format".into()),
            params: HashMap::new(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct DeliveryInfo {
    pub company_package_id: CompanyPackageId,
    pub shipping_id: ShippingId,
    pub name: String,
    pub logo: String,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProductInfo {
    pub base_product_id: BaseProductId,
    pub cashback: Option<CashbackPercent>,
    pub pre_order: bool,
    pub pre_order_days: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
pub struct ConvertCartPayload {
    pub conversion_id: Option<ConversionId>,
    pub user_id: UserId,
    pub receiver_name: String,
    #[validate(custom = "validate_phone")]
    pub receiver_phone: String,
    pub receiver_email: String,
    #[serde(flatten)]
    pub address: AddressFull,
    pub seller_prices: HashMap<ProductId, ProductSellerPrice>,
    pub coupons: HashMap<CouponId, CouponInfo>,
    pub delivery_info: HashMap<ProductId, DeliveryInfo>,
    pub product_info: HashMap<ProductId, ProductInfo>,
    pub uuid: Uuid,
    pub currency_type: Option<CurrencyType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct BuyNow {
    pub product_id: ProductId,
    pub customer_id: UserId,
    pub store_id: StoreId,
    pub address: AddressFull,
    pub receiver_name: String,
    pub receiver_email: String,
    pub price: ProductSellerPrice,
    pub quantity: Quantity,
    pub currency: Currency,
    #[validate(custom = "validate_phone")]
    pub receiver_phone: String,
    pub pre_order: bool,
    pub pre_order_days: i32,
    pub coupon: Option<CouponInfo>,
    pub delivery_info: Option<DeliveryInfo>,
    pub product_info: ProductInfo,
    pub uuid: Uuid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CouponInfo {
    pub id: CouponId,
    pub percent: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BuyNowPayload {
    pub conversion_id: Option<ConversionId>,
    #[serde(flatten)]
    pub buy_now: BuyNow,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConvertCartRevertPayload {
    pub conversion_id: ConversionId,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OrderSearchTerms {
    pub slug: Option<OrderSlug>,
    pub created_from: Option<DateTime<Utc>>,
    pub created_to: Option<DateTime<Utc>>,
    pub updated_from: Option<DateTime<Utc>>,
    pub updated_to: Option<DateTime<Utc>>,
    pub payment_status: Option<bool>,
    pub customer: Option<UserId>,
    pub store: Option<StoreId>,
    pub state: Option<OrderState>,
    pub currency_type: Option<CurrencyType>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrderDiff {
    pub id: OrderDiffId,
    pub parent: OrderId,
    pub committer: UserId,
    pub committed_at: DateTime<Utc>,
    pub state: OrderState,
    pub comment: Option<String>,
    pub committer_role: CommitterRole,
}

pub trait OrderClient {
    fn convert_cart(
        &self,
        conversion_id: Option<ConversionId>,
        user_id: UserId,
        seller_prices: HashMap<ProductId, ProductSellerPrice>,
        address: AddressFull,
        receiver_name: String,
        receiver_phone: String,
        receiver_email: String,
        coupons: HashMap<CouponId, CouponInfo>,
        delivery_info: HashMap<ProductId, DeliveryInfo>,
        product_info: HashMap<ProductId, ProductInfo>,
        uuid: Uuid,
        currency_type: Option<CurrencyType>,
    ) -> ApiFuture<Vec<Order>>;
    fn create_buy_now(
        &self,
        payload: BuyNow,
        conversion_id: Option<ConversionId>,
    ) -> ApiFuture<Vec<Order>>;
    fn revert_cart_conversion(&self, conversion_id: ConversionId) -> ApiFuture<()>;
    fn get_order(&self, id: OrderIdentifier) -> ApiFuture<Option<Order>>;
    fn get_order_diff(&self, id: OrderIdentifier) -> ApiFuture<Vec<OrderDiff>>;
    fn get_orders_for_user(&self, user_id: UserId) -> ApiFuture<Vec<Order>>;
    fn get_orders_for_store(&self, store_id: StoreId) -> ApiFuture<Vec<Order>>;
    fn delete_order(&self, id: OrderIdentifier) -> ApiFuture<()>;
    fn set_order_state(
        &self,
        order_id: OrderIdentifier,
        state: OrderState,
        comment: Option<String>,
        track_id: Option<String>,
        committer_role: CommitterRole,
    ) -> ApiFuture<Option<Order>>;
    /// Search using the terms provided.
    fn search(&self, terms: OrderSearchTerms) -> ApiFuture<Vec<Order>>;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateStatePayload {
    pub state: OrderState,
    pub track_id: Option<String>,
    pub comment: Option<String>,
    pub committer_role: CommitterRole,
}

impl OrderClient for RestApiClient {
    fn convert_cart(
        &self,
        conversion_id: Option<ConversionId>,
        user_id: UserId,
        seller_prices: HashMap<ProductId, ProductSellerPrice>,
        address: AddressFull,
        receiver_name: String,
        receiver_phone: String,
        receiver_email: String,
        coupons: HashMap<CouponId, CouponInfo>,
        delivery_info: HashMap<ProductId, DeliveryInfo>,
        product_info: HashMap<ProductId, ProductInfo>,
        uuid: Uuid,
        currency_type: Option<CurrencyType>,
    ) -> ApiFuture<Vec<Order>> {
        http_req(
            self.http_client
                .post(&self.build_route(&Route::OrderFromCart))
                .body(JsonPayload(ConvertCartPayload {
                    conversion_id,
                    user_id,
                    seller_prices,
                    address,
                    receiver_name,
                    receiver_phone,
                    receiver_email,
                    coupons,
                    delivery_info,
                    product_info,
                    uuid,
                    currency_type,
                })),
        )
    }

    fn create_buy_now(
        &self,
        buy_now: BuyNow,
        conversion_id: Option<ConversionId>,
    ) -> ApiFuture<Vec<Order>> {
        http_req(
            self.http_client
                .post(&self.build_route(&Route::OrderFromBuyNow))
                .body(JsonPayload(BuyNowPayload {
                    conversion_id,
                    buy_now,
                })),
        )
    }

    fn revert_cart_conversion(&self, conversion_id: ConversionId) -> ApiFuture<()> {
        http_req(
            self.http_client
                .post(&self.build_route(&Route::OrderFromCartRevert))
                .body(JsonPayload(ConvertCartRevertPayload { conversion_id })),
        )
    }
    fn get_order(&self, order_id: OrderIdentifier) -> ApiFuture<Option<Order>> {
        http_req(
            self.http_client
                .get(&self.build_route(&Route::Order { order_id })),
        )
    }
    fn get_order_diff(&self, order_id: OrderIdentifier) -> ApiFuture<Vec<OrderDiff>> {
        http_req(
            self.http_client
                .get(&self.build_route(&Route::OrderDiff { order_id })),
        )
    }
    fn get_orders_for_user(&self, user: UserId) -> ApiFuture<Vec<Order>> {
        http_req(
            self.http_client
                .get(&self.build_route(&Route::OrdersByUser { user })),
        )
    }
    fn get_orders_for_store(&self, store_id: StoreId) -> ApiFuture<Vec<Order>> {
        http_req(
            self.http_client
                .get(&self.build_route(&Route::OrdersByStore { store_id })),
        )
    }
    fn delete_order(&self, order_id: OrderIdentifier) -> ApiFuture<()> {
        http_req(
            self.http_client
                .delete(&self.build_route(&Route::Order { order_id })),
        )
    }
    fn set_order_state(
        &self,
        order_id: OrderIdentifier,
        state: OrderState,
        comment: Option<String>,
        track_id: Option<String>,
        committer_role: CommitterRole,
    ) -> ApiFuture<Option<Order>> {
        http_req(
            self.http_client
                .put(&self.build_route(&Route::OrderStatus { order_id }))
                .body(JsonPayload(UpdateStatePayload {
                    state,
                    comment,
                    track_id,
                    committer_role,
                })),
        )
    }
    fn search(&self, terms: OrderSearchTerms) -> ApiFuture<Vec<Order>> {
        http_req(
            self.http_client
                .post(&self.build_route(&Route::OrderSearch))
                .body(JsonPayload(terms)),
        )
    }
}
