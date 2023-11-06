use rpc_client::*;
use types::*;
use util::*;

use geo::Point as GeoPoint;
use std::collections::HashMap;
use stq_roles;
use stq_router::{Builder as RouterBuilder, Router};
use stq_types::*;

#[derive(Clone, Debug)]
pub enum Route {
    Warehouses,
    WarehousesByStore {
        store_id: StoreId,
    },
    Warehouse {
        warehouse_id: WarehouseIdentifier,
    },
    StocksInWarehouse {
        warehouse_id: WarehouseId,
    },
    StockInWarehouse {
        warehouse_id: WarehouseId,
        product_id: ProductId,
    },
    StocksByProductId {
        product_id: ProductId,
    },
    StockById {
        stock_id: StockId,
    },
    Stocks,
    Roles(stq_roles::routing::Route),
}

impl From<stq_roles::routing::Route> for Route {
    fn from(v: stq_roles::routing::Route) -> Self {
        Route::Roles(v)
    }
}

fn warehouse_identifier_route(id: &WarehouseIdentifier) -> String {
    use self::WarehouseIdentifier::*;

    match id {
        Id(id) => format!("by-id/{}", id),
        Slug(slug) => format!("by-slug/{}", slug),
    }
}

impl RouteBuilder for Route {
    fn route(&self) -> String {
        use self::Route::*;

        match self {
            Warehouses => "warehouses".to_string(),
            WarehousesByStore { store_id } => format!("warehouses/by-store/{}", store_id),
            Warehouse { warehouse_id } => {
                format!("warehouses/{}", warehouse_identifier_route(warehouse_id))
            }
            StocksInWarehouse { warehouse_id } => format!(
                "warehouses/{}/products",
                warehouse_identifier_route(&WarehouseIdentifier::Id(*warehouse_id))
            ),
            StockInWarehouse {
                warehouse_id,
                product_id,
            } => format!(
                "warehouses/{}/products/{}",
                warehouse_identifier_route(&WarehouseIdentifier::Id(*warehouse_id)),
                product_id
            ),
            StocksByProductId { product_id } => format!("stocks/by-product-id/{}", product_id),
            StockById { stock_id } => format!("stocks/by-id/{}", stock_id),
            Stocks => "stocks".to_string(),
            Roles(route) => route.route(),
        }
    }
}

impl Route {
    pub fn from_path(s: &str) -> Option<Self> {
        lazy_static! {
            static ref ROUTER: Router<Route> =
                stq_roles::routing::add_routes(RouterBuilder::default())
                    .with_route(r"^/warehouses$", |_| Some(Route::Warehouses))
                    .with_route(r"^/warehouses/by-id/([a-zA-Z0-9-]+)/products$", |params| {
                        params
                            .get(0)
                            .and_then(|string_id| string_id.parse().ok())
                            .map(|warehouse_id| Route::StocksInWarehouse { warehouse_id })
                    })
                    .with_route(
                        r"^/warehouses/by-id/([a-zA-Z0-9-]+)/products/(\d+)$",
                        |params| {
                            if let Some(warehouse_id_s) = params.get(0) {
                                if let Some(product_id_s) = params.get(1) {
                                    if let Ok(warehouse_id) =
                                        warehouse_id_s.parse().map(WarehouseId)
                                    {
                                        if let Ok(product_id) = product_id_s.parse().map(ProductId)
                                        {
                                            return Some(Route::StockInWarehouse {
                                                warehouse_id,
                                                product_id,
                                            });
                                        }
                                    }
                                }
                            }
                            None
                        }
                    )
                    .with_route(r"^/warehouses/by-id/([a-zA-Z0-9-]+)$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok().map(WarehouseIdentifier::Id))
                        .map(|warehouse_id| Route::Warehouse { warehouse_id }))
                    .with_route(r"^/warehouses/by-slug/([a-zA-Z0-9-]+)$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok().map(WarehouseIdentifier::Slug))
                        .map(|warehouse_id| Route::Warehouse { warehouse_id }))
                    .with_route(r"^/warehouses/by-store/(\d+)$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok())
                        .map(|store_id| Route::WarehousesByStore { store_id }))
                    .with_route(r"^/stocks/by-product-id/(\d+)$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok())
                        .map(|product_id| Route::StocksByProductId { product_id }))
                    .with_route(r"^/stocks/by-id/([a-zA-Z0-9-]+)$", |params| params
                        .get(0)
                        .and_then(|string_id| string_id.parse().ok())
                        .map(|stock_id| Route::StockById { stock_id }))
                    .with_route(r"^/stocks$", |_| Some(Route::Stocks))
                    .build();
        }

        ROUTER.test(s)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Warehouse {
    pub id: WarehouseId,
    pub store_id: StoreId,
    pub slug: WarehouseSlug,
    pub name: Option<String>,
    pub location: Option<GeoPoint<f64>>,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<Alpha3>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub address: Option<String>,
    pub place_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WarehouseInput {
    #[serde(default = "WarehouseId::new")]
    pub id: WarehouseId,
    pub store_id: StoreId,
    pub name: Option<String>,
    pub location: Option<GeoPoint<f64>>,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<Alpha3>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub address: Option<String>,
    pub place_id: Option<String>,
}

impl WarehouseInput {
    pub fn new(store_id: StoreId) -> Self {
        Self {
            store_id,
            id: WarehouseId::new(),
            name: Default::default(),
            location: Default::default(),
            administrative_area_level_1: Default::default(),
            administrative_area_level_2: Default::default(),
            country: Default::default(),
            country_code: Default::default(),
            locality: Default::default(),
            political: Default::default(),
            postal_code: Default::default(),
            route: Default::default(),
            street_number: Default::default(),
            address: Default::default(),
            place_id: Default::default(),
        }
    }

    pub fn split_slug(v: Warehouse) -> (WarehouseInput, WarehouseSlug) {
        (
            WarehouseInput {
                id: v.id,
                store_id: v.store_id,
                name: v.name,
                location: v.location,
                administrative_area_level_1: v.administrative_area_level_1,
                administrative_area_level_2: v.administrative_area_level_2,
                country: v.country,
                country_code: v.country_code,
                locality: v.locality,
                political: v.political,
                postal_code: v.postal_code,
                route: v.route,
                street_number: v.street_number,
                address: v.address,
                place_id: v.place_id,
            },
            v.slug,
        )
    }

    pub fn with_slug(self, slug: WarehouseSlug) -> Warehouse {
        Warehouse {
            id: self.id,
            store_id: self.store_id,
            slug,
            name: self.name,
            location: self.location,
            administrative_area_level_1: self.administrative_area_level_1,
            administrative_area_level_2: self.administrative_area_level_2,
            country: self.country,
            country_code: self.country_code,
            locality: self.locality,
            political: self.political,
            postal_code: self.postal_code,
            route: self.route,
            street_number: self.street_number,
            address: self.address,
            place_id: self.place_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stock {
    pub id: StockId,
    pub warehouse_id: WarehouseId,
    pub product_id: ProductId,
    pub quantity: Quantity,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StockMeta {
    pub quantity: Quantity,
}

impl From<Stock> for (ProductId, StockMeta) {
    fn from(v: Stock) -> (ProductId, StockMeta) {
        (
            v.product_id,
            StockMeta {
                quantity: v.quantity,
            },
        )
    }
}

impl From<Stock> for (StockId, WarehouseId, ProductId, StockMeta) {
    fn from(v: Stock) -> Self {
        (
            v.id,
            v.warehouse_id,
            v.product_id,
            StockMeta {
                quantity: v.quantity,
            },
        )
    }
}

pub type StockMap = HashMap<ProductId, StockMeta>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockSetPayload {
    pub quantity: Quantity,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WarehouseUpdateData {
    pub slug: Option<ValueContainer<WarehouseSlug>>,
    pub name: Option<ValueContainer<Option<String>>>,
    pub location: Option<ValueContainer<Option<GeoPoint<f64>>>>,
    pub administrative_area_level_1: Option<ValueContainer<Option<String>>>,
    pub administrative_area_level_2: Option<ValueContainer<Option<String>>>,
    pub country: Option<ValueContainer<Option<String>>>,
    pub country_code: Option<ValueContainer<Option<Alpha3>>>,
    pub locality: Option<ValueContainer<Option<String>>>,
    pub political: Option<ValueContainer<Option<String>>>,
    pub postal_code: Option<ValueContainer<Option<String>>>,
    pub route: Option<ValueContainer<Option<String>>>,
    pub street_number: Option<ValueContainer<Option<String>>>,
    pub address: Option<ValueContainer<Option<String>>>,
    pub place_id: Option<ValueContainer<Option<String>>>,
}

pub trait WarehouseClient {
    fn create_warehouse(&self, new_warehouse: WarehouseInput) -> ApiFuture<Warehouse>;
    fn get_warehouse(&self, warehouse_id: WarehouseIdentifier) -> ApiFuture<Option<Warehouse>>;
    fn update_warehouse(
        &self,
        warehouse_id: WarehouseIdentifier,
        update_data: WarehouseUpdateData,
    ) -> ApiFuture<Option<Warehouse>>;
    fn delete_warehouse(&self, warehouse_id: WarehouseIdentifier) -> ApiFuture<Option<Warehouse>>;
    fn delete_all_warehouses(&self) -> ApiFuture<Vec<Warehouse>>;
    fn get_warehouses_for_store(&self, store_id: StoreId) -> ApiFuture<Vec<Warehouse>>;

    fn set_product_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
        quantity: Quantity,
    ) -> ApiFuture<Stock>;
    fn get_product_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
    ) -> ApiFuture<Option<Stock>>;
    fn list_products_in_warehouse(&self, warehouse_id: WarehouseId) -> ApiFuture<StockMap>;

    fn get_warehouse_product(&self, warehouse_product_id: StockId) -> ApiFuture<Option<Stock>>;

    /// Find all products with id in all warehouses
    fn find_by_product_id(&self, product_id: ProductId) -> ApiFuture<Vec<Stock>>;
}

impl WarehouseClient for RestApiClient {
    fn create_warehouse(&self, new_warehouse: WarehouseInput) -> ApiFuture<Warehouse> {
        http_req(
            self.http_client
                .post(&self.build_route(&Route::Warehouses))
                .body(JsonPayload(new_warehouse)),
        )
    }
    fn get_warehouse(&self, warehouse_id: WarehouseIdentifier) -> ApiFuture<Option<Warehouse>> {
        http_req(
            self.http_client
                .get(&self.build_route(&Route::Warehouse { warehouse_id })),
        )
    }
    fn update_warehouse(
        &self,
        warehouse_id: WarehouseIdentifier,
        update_data: WarehouseUpdateData,
    ) -> ApiFuture<Option<Warehouse>> {
        http_req(
            self.http_client
                .put(&self.build_route(&Route::Warehouse { warehouse_id }))
                .body(JsonPayload(update_data)),
        )
    }
    fn delete_warehouse(&self, warehouse_id: WarehouseIdentifier) -> ApiFuture<Option<Warehouse>> {
        http_req(
            self.http_client
                .delete(&self.build_route(&Route::Warehouse { warehouse_id })),
        )
    }
    fn delete_all_warehouses(&self) -> ApiFuture<Vec<Warehouse>> {
        http_req(
            self.http_client
                .delete(&self.build_route(&Route::Warehouses)),
        )
    }
    fn get_warehouses_for_store(&self, store_id: StoreId) -> ApiFuture<Vec<Warehouse>> {
        http_req(
            self.http_client
                .get(&self.build_route(&Route::WarehousesByStore { store_id })),
        )
    }

    fn set_product_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
        quantity: Quantity,
    ) -> ApiFuture<Stock> {
        http_req(
            self.http_client
                .put(&self.build_route(&Route::StockInWarehouse {
                    warehouse_id,
                    product_id,
                }))
                .body(JsonPayload(StockSetPayload { quantity })),
        )
    }
    fn get_product_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
    ) -> ApiFuture<Option<Stock>> {
        http_req(
            self.http_client
                .get(&self.build_route(&Route::StockInWarehouse {
                    warehouse_id,
                    product_id,
                })),
        )
    }
    fn list_products_in_warehouse(&self, warehouse_id: WarehouseId) -> ApiFuture<StockMap> {
        http_req(
            self.http_client
                .get(&self.build_route(&Route::StocksInWarehouse { warehouse_id })),
        )
    }

    fn get_warehouse_product(&self, stock_id: StockId) -> ApiFuture<Option<Stock>> {
        http_req(
            self.http_client
                .get(&self.build_route(&Route::StockById { stock_id })),
        )
    }

    /// Find all products with id in all warehouses
    fn find_by_product_id(&self, product_id: ProductId) -> ApiFuture<Vec<Stock>> {
        http_req(
            self.http_client
                .get(&self.build_route(&Route::StocksByProductId { product_id })),
        )
    }
}
