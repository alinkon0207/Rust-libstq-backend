extern crate regex;

use regex::Regex;
use std::sync::Arc;

pub type ParamsConverter<T> = Arc<Fn(Vec<&str>) -> Option<T> + Send + Sync>;

/// `Router` class maps regex to type-safe list of routes, defined by `enum Route`
#[derive(Clone)]
pub struct Router<T> {
    regex_and_converters: Vec<(Regex, ParamsConverter<T>)>,
}

/// The builder for `Router`
#[derive(Clone)]
pub struct Builder<T>(Router<T>);

impl<T> Default for Builder<T> {
    fn default() -> Self {
        Builder(Router {
            regex_and_converters: Default::default(),
        })
    }
}

impl<T> Builder<T> {
    /// Adds mapping between regex and route with params
    /// converter is a function with argument being a set of regex matches (strings) for route params in regex
    /// this is needed if you want to convert params from strings to int or some other types
    ///
    /// #Examples
    ///
    /// ```
    /// use stq_router::Builder as RouterBuilder;
    ///
    /// #[derive(Debug)]
    /// pub enum Route {
    ///     User(i32),
    /// }
    ///
    /// let mut router = RouterBuilder::default().with_route(
    ///     r"^/users/(\d+)$", |params| {
    ///         params.get(0)
    ///            .and_then(|string_id| string_id.parse::<i32>().ok())
    ///            .map(|user_id| Route::User(user_id))
    ///     }
    /// );
    /// ```
    pub fn with_route<F>(mut self, regex_pattern: &str, converter: F) -> Self
    where
        F: Fn(Vec<&str>) -> Option<T> + Send + Sync + 'static,
    {
        let regex = Regex::new(regex_pattern).unwrap();
        self.0.regex_and_converters.push((regex, Arc::new(converter)));
        self
    }

    pub fn build(self) -> Router<T> {
        self.0
    }
}

impl<T> Router<T> {
    /// Tests string router for matches
    /// Returns Some(route) if there's a match
    /// #Examples
    ///
    /// ```
    /// use stq_router::Builder as RouterBuilder;
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub enum Route {
    ///     Users,
    /// }
    ///
    /// let router = RouterBuilder::default().with_route(r"^/users$", |_| Some(Route::Users)).build();
    /// let route = router.test("/users").unwrap();
    /// assert_eq!(route, Route::Users);
    /// ```
    pub fn test(&self, route: &str) -> Option<T> {
        for (pattern, test_func) in &self.regex_and_converters {
            if let Some(v) = Self::get_matches(&pattern, route) {
                return test_func(v);
            }
        }
        None
    }

    fn get_matches<'a>(regex: &Regex, string: &'a str) -> Option<Vec<&'a str>> {
        regex.captures(string).and_then(|captures| {
            captures
                .iter()
                .skip(1)
                .fold(Some(Vec::<&str>::new()), |mut maybe_acc, maybe_match| {
                    if let Some(ref mut acc) = maybe_acc {
                        if let Some(mtch) = maybe_match {
                            acc.push(mtch.as_str());
                        }
                    }
                    maybe_acc
                })
        })
    }
}

/// Legacy router
pub struct RouteParser<T> {
    regex_and_converters: Vec<(Regex, ParamsConverter<T>)>,
}

impl<T> Default for RouteParser<T> {
    fn default() -> Self {
        Self {
            regex_and_converters: Default::default(),
        }
    }
}

impl<T> RouteParser<T> {
    /// Adds mapping between regex and route
    /// #Examples
    ///
    /// ```
    /// use stq_router::RouteParser;
    ///
    /// #[derive(Debug)]
    /// pub enum Route {
    ///     Users,
    /// }
    ///
    /// let mut router = RouteParser::default();
    /// router.add_route(r"^/users$", || Route::Users);
    /// ```
    pub fn add_route<F>(&mut self, regex_pattern: &str, f: F) -> &Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.add_route_with_params(regex_pattern, move |_| Some(f()));
        self
    }

    /// Adds mapping between regex and route with params
    /// converter is a function with argument being a set of regex matches (strings) for route params in regex
    /// this is needed if you want to convert params from strings to int or some other types
    ///
    /// #Examples
    ///
    /// ```
    /// use stq_router::RouteParser;
    ///
    /// #[derive(Debug)]
    /// pub enum Route {
    ///     User(i32),
    /// }
    ///
    /// let mut router = RouteParser::default();
    /// router.add_route_with_params(r"^/users/(\d+)$", |params| {
    ///     params.get(0)
    ///        .and_then(|string_id| string_id.parse::<i32>().ok())
    ///        .map(|user_id| Route::User(user_id))
    /// });
    /// ```
    pub fn add_route_with_params<F>(&mut self, regex_pattern: &str, converter: F) -> &Self
    where
        F: Fn(Vec<&str>) -> Option<T> + Send + Sync + 'static,
    {
        let regex = Regex::new(regex_pattern).unwrap();
        self.regex_and_converters.push((regex, Arc::new(converter)));
        self
    }

    /// Tests string router for matches
    /// Returns Some(route) if there's a match
    /// #Examples
    ///
    /// ```
    /// use stq_router::RouteParser;
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub enum Route {
    ///     Users,
    /// }
    ///
    /// let mut router = RouteParser::default();
    /// router.add_route(r"^/users$", || Route::Users);
    /// let route = router.test("/users").unwrap();
    /// assert_eq!(route, Route::Users);
    /// ```
    pub fn test(&self, route: &str) -> Option<T> {
        for (pattern, test_func) in &self.regex_and_converters {
            if let Some(v) = Self::get_matches(&pattern, route) {
                return test_func(v);
            }
        }
        None
    }

    fn get_matches<'a>(regex: &Regex, string: &'a str) -> Option<Vec<&'a str>> {
        regex.captures(string).and_then(|captures| {
            captures
                .iter()
                .skip(1)
                .fold(Some(Vec::<&str>::new()), |mut maybe_acc, maybe_match| {
                    if let Some(ref mut acc) = maybe_acc {
                        if let Some(mtch) = maybe_match {
                            acc.push(mtch.as_str());
                        }
                    }
                    maybe_acc
                })
        })
    }
}
