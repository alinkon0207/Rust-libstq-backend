use rpc_client::RestApiClient;
use types::*;
use util::*;

use std::time::SystemTime;
use stq_router::{Builder as RouterBuilder, Router};
use stq_types::*;

#[derive(Clone, Debug)]
pub enum Route {
    Pages,
    Page { identifier: PageIdentifier },
}

fn page_identifier_route(id: &PageIdentifier) -> String {
    use self::PageIdentifier::*;

    match id {
        Id(id) => format!("by-id/{}", id),
        Slug(slug) => format!("by-slug/{}", slug),
    }
}

impl RouteBuilder for Route {
    fn route(&self) -> String {
        use self::Route::*;

        match self {
            Pages => "pages".to_string(),
            Page { identifier } => format!("pages/{}", page_identifier_route(identifier)),
        }
    }
}

impl Route {
    pub fn from_path(s: &str) -> Option<Self> {
        lazy_static! {
            static ref ROUTER: Router<Route> = RouterBuilder::default()
                .with_route(r"^/pages$", |_| Some(Route::Pages))
                .with_route(r"^/pages/by-id/([a-zA-Z0-9-]+)$", |params| params
                    .into_iter()
                    .next()
                    .and_then(|string_id| string_id.parse().ok().map(PageIdentifier::Id))
                    .map(|identifier| Route::Page { identifier }))
                .with_route(r"^/pages/by-slug/([a-zA-Z0-9-]+)$", |params| params
                    .into_iter()
                    .next()
                    .and_then(|string_id| string_id
                        .parse()
                        .ok()
                        .map(|s: PageSlug| s.0.to_lowercase().into())
                        .map(PageIdentifier::Slug))
                    .map(|identifier| Route::Page { identifier }))
                .build();
        }

        ROUTER.test(s)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Page {
    pub id: PageId,
    pub slug: PageSlug,
    pub html: String,
    pub css: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewPage {
    pub id: PageId,
    pub slug: PageSlug,
    pub html: String,
    pub css: String,
}

pub trait PageClient {
    fn get_page(&self, identifier: PageIdentifier) -> ApiFuture<Option<Page>>;
    fn insert_page(&self, item: NewPage) -> ApiFuture<Page>;
}

impl PageClient for RestApiClient {
    fn get_page(&self, identifier: PageIdentifier) -> ApiFuture<Option<Page>> {
        http_req(
            self.http_client
                .get(&self.build_route(&Route::Page { identifier })),
        )
    }

    fn insert_page(&self, item: NewPage) -> ApiFuture<Page> {
        http_req(
            self.http_client
                .post(&self.build_route(&Route::Pages))
                .body(JsonPayload(item)),
        )
    }
}
