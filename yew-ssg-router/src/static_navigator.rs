use crate::utils::get_base_url;
use std::fmt::Debug;
use yew::prelude::*;
use yew_router::Routable;

/// A navigator implementation for static rendering (SSR/SSG)
#[derive(Debug, Clone, PartialEq)]
pub struct StaticNavigator {
    path: String,
    base_url: String,
    basename: Option<String>,
}

/// Properties for the StaticNavigator
#[derive(Properties, PartialEq, Clone, Debug)]
pub struct StaticNavigatorProps {
    #[prop_or_default]
    pub children: Html,
    #[prop_or_default]
    pub basename: Option<String>,
}

impl StaticNavigator {
    pub fn new() -> Self {
        let path = crate::get_static_path().unwrap_or_else(|| "/".to_string());
        let base_url = get_base_url();

        Self {
            path,
            base_url,
            basename: None,
        }
    }

    // For tests - create with specific values
    #[cfg(test)]
    pub fn new_with_path_and_base(path: &str, base_url: &str) -> Self {
        Self {
            path: path.to_string(),
            base_url: base_url.to_string(),
            basename: None,
        }
    }

    // Add a new method that accepts basename
    pub fn with_basename(mut self, basename: Option<String>) -> Self {
        self.basename = basename;
        self
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn basename(&self) -> Option<&str> {
        self.basename.as_deref()
    }

    /// Logs push navigation attempt during static rendering
    pub fn push<R>(&self, route: &R)
    where
        R: Routable + Debug,
    {
        log::debug!(
            "StaticNavigator: Push route {:?} attempted during static render",
            route
        );
    }

    /// Logs replace navigation attempt during static rendering
    pub fn replace<R>(&self, route: &R)
    where
        R: Routable + Debug,
    {
        log::debug!(
            "StaticNavigator: Replace route {:?} attempted during static render",
            route
        );
    }

    /// Logs back navigation attempt during static rendering
    pub fn back(&self) {
        log::debug!("StaticNavigator: Back navigation attempted during static render");
    }

    /// Logs forward navigation attempt during static rendering
    pub fn forward(&self) {
        log::debug!("StaticNavigator: Forward navigation attempted during static render");
    }

    /// Logs go navigation attempt during static rendering
    pub fn go(&self, delta: isize) {
        log::debug!(
            "StaticNavigator: Go({}) navigation attempted during static render",
            delta
        );
    }
}

/// Hook to access the StaticNavigator within components
#[hook]
pub fn use_navigator() -> Option<StaticNavigator> {
    // Get from context, maintain API compatibility with yew-router
    use_context::<StaticNavigator>()
}

/// Component that provides StaticNavigator context
#[function_component(StaticNavigatorProvider)]
pub fn static_navigator_provider(props: &StaticNavigatorProps) -> Html {
    let navigator = StaticNavigator::new().with_basename(props.basename.clone());

    html! {
        <ContextProvider<StaticNavigator> context={navigator}>
            { props.children.clone() }
        </ContextProvider<StaticNavigator>>
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    #[derive(Debug, Clone, PartialEq, Routable)]
    enum TestRoute {
        #[at("/")]
        Home,
        #[at("/about")]
        About,
    }

    #[test]
    #[serial]
    fn test_static_navigator() {
        // Create a navigator with controlled values instead of reading from env vars
        let navigator = StaticNavigator::new_with_path_and_base("/", "");

        // Test path and base_url
        assert_eq!(navigator.path(), "/");
        assert_eq!(navigator.base_url(), "");
        assert_eq!(navigator.basename(), None);

        // Clone before moving
        let navigator_with_basename = navigator.clone().with_basename(Some("/app".to_string()));
        assert_eq!(navigator_with_basename.basename(), Some("/app"));

        // Test navigation methods (they should just log)
        navigator.push(&TestRoute::About);
        navigator.replace(&TestRoute::Home);
        navigator.back();
        navigator.forward();
        navigator.go(1);
    }

    // Test that simply creates the component without trying to render it
    #[test]
    #[serial]
    fn test_navigator_props() {
        // Just test the props structure
        let props = StaticNavigatorProps {
            children: html! { <div>{"Test"}</div> },
            basename: Some("/app".to_string()),
        };

        // Verify props can be cloned and compared
        let props_clone = props.clone();
        assert_eq!(props, props_clone);
    }
}
