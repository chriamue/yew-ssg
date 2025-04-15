#[macro_export]
macro_rules! impl_localized_route {
    ($route_type:ty, $localized_route_type:ident, $languages:expr, $default_lang:expr) => {
        #[derive(Clone, PartialEq, Debug)]
        pub enum $localized_route_type {
            Default($route_type),
            Localized { lang: String, route: $route_type },
        }

        impl Default for $localized_route_type {
            fn default() -> Self {
                let default_route = <$route_type as Default>::default();
                $localized_route_type::Default(default_route)
            }
        }

        impl $crate::LocalizedRoutable for $localized_route_type {
            type BaseRoute = $route_type;

            fn get_lang(&self) -> Option<String> {
                match self {
                    Self::Default(_) => None,
                    Self::Localized { lang, .. } => Some(Self::validate_lang(lang)),
                }
            }

            fn get_route(&self) -> &Self::BaseRoute {
                match self {
                    Self::Default(route) => route,
                    Self::Localized { route, .. } => route,
                }
            }

            fn from_route(route: Self::BaseRoute, lang: Option<&str>) -> Self {
                match lang {
                    Some(lang) if Self::supported_languages().contains(&lang) => Self::Localized {
                        lang: lang.to_string(),
                        route,
                    },
                    _ => Self::Default(route),
                }
            }

            fn validate_lang(lang: &str) -> String {
                if Self::supported_languages().contains(&lang) {
                    lang.to_string()
                } else {
                    Self::default_language().to_string()
                }
            }

            fn supported_languages() -> &'static [&'static str] {
                $languages
            }

            fn default_language() -> &'static str {
                $default_lang
            }
        }

        impl yew_router::Routable for $localized_route_type {
            fn from_path(
                path: &str,
                params: &std::collections::HashMap<&str, &str>,
            ) -> Option<Self> {
                let (lang, remaining_path) = Self::extract_lang_from_path(path);

                if let Some(lang) = lang {
                    // This is a localized route
                    <$route_type as yew_router::Routable>::from_path(&remaining_path, params)
                        .map(|route| Self::Localized { lang, route })
                } else {
                    // This is a default route
                    <$route_type as yew_router::Routable>::from_path(path, params)
                        .map(Self::Default)
                }
            }

            fn to_path(&self) -> String {
                match self {
                    Self::Default(route) => route.to_path(),
                    Self::Localized { lang, route } => {
                        let route_path = route.to_path();
                        if route_path == "/" {
                            format!("/{}", lang)
                        } else {
                            format!("/{}{}", lang, route_path)
                        }
                    }
                }
            }

            fn routes() -> Vec<&'static str> {
                let mut routes = <$route_type as yew_router::Routable>::routes();

                // Add language prefixed routes
                for &lang in Self::supported_languages() {
                    for route in <$route_type as yew_router::Routable>::routes() {
                        let localized_route = if route == "/" {
                            format!("/{}", lang)
                        } else {
                            format!("/{}{}", lang, route)
                        };

                        // Leak to get static lifetime
                        let localized_route = Box::leak(localized_route.into_boxed_str());
                        routes.push(localized_route);
                    }
                }

                routes
            }

            fn not_found_route() -> Option<Self> {
                <$route_type as yew_router::Routable>::not_found_route().map(Self::Default)
            }

            fn recognize(pathname: &str) -> Option<Self> {
                let (lang, remaining_path) = Self::extract_lang_from_path(pathname);

                if let Some(lang) = lang {
                    // This is a localized route
                    <$route_type as yew_router::Routable>::recognize(&remaining_path)
                        .map(|route| Self::Localized { lang, route })
                } else {
                    // This is a default route
                    <$route_type as yew_router::Routable>::recognize(pathname).map(Self::Default)
                }
            }
        }

        impl strum::IntoEnumIterator for $localized_route_type
        where
            $route_type: strum::IntoEnumIterator + Clone,
        {
            type Iterator = $crate::LocalizedRouteIter<$route_type, Self>;

            fn iter() -> Self::Iterator {
                $crate::LocalizedRouteIter::new(Self::supported_languages())
            }
        }
    };
}
