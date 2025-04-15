use crate::LocalizedRoutable;
use std::iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator};
use std::marker::PhantomData;
use strum::IntoEnumIterator;

/// Iterator for localized routes that combines a base route enum with language variants
#[derive(Clone)]
pub struct LocalizedRouteIter<R, L>
where
    R: IntoEnumIterator + Clone,
    L: LocalizedRoutable<BaseRoute = R>,
{
    route_iter: Option<R::Iterator>,
    current_route: Option<R>,
    language_iter: Option<std::slice::Iter<'static, &'static str>>,
    languages: &'static [&'static str],
    exhausted_languages: bool,
    _phantom: PhantomData<L>,
}

impl<R, L> LocalizedRouteIter<R, L>
where
    R: IntoEnumIterator + Clone,
    L: LocalizedRoutable<BaseRoute = R>,
{
    pub fn new(languages: &'static [&'static str]) -> Self {
        let mut route_iter = Some(R::iter());

        // Get the first route
        let current_route = route_iter.as_mut().and_then(|iter| iter.next());

        Self {
            route_iter,
            current_route,
            language_iter: Some(languages.iter()),
            languages,
            exhausted_languages: false,
            _phantom: PhantomData,
        }
    }
}

impl<R, L> Iterator for LocalizedRouteIter<R, L>
where
    R: IntoEnumIterator + Clone,
    L: LocalizedRoutable<BaseRoute = R>,
{
    type Item = L;

    fn next(&mut self) -> Option<Self::Item> {
        // If we have a current route...
        if let Some(ref route) = self.current_route {
            // If we haven't started languages yet, return default route first
            if !self.exhausted_languages {
                self.exhausted_languages = true;
                self.language_iter = Some(self.languages.iter());
                return Some(L::from_route(route.clone(), None));
            }

            // If we have language iterator, try to get the next language
            if let Some(ref mut lang_iter) = self.language_iter {
                if let Some(lang) = lang_iter.next() {
                    return Some(L::from_route(route.clone(), Some(lang)));
                }
            }

            // If we've exhausted the languages, move to the next route
            self.exhausted_languages = false;
            if let Some(ref mut iter) = self.route_iter {
                self.current_route = iter.next();
                if self.current_route.is_some() {
                    return self.next();
                }
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let route_count = match &self.route_iter {
            Some(iter) => iter.size_hint().0,
            None => 0,
        };

        // If we have a current route, add 1 to the count
        let current_adds = if self.current_route.is_some() { 1 } else { 0 };

        // Calculate total variants:
        // For each route: 1 default + 1 per language
        let variants_per_route = 1 + self.languages.len();
        let total = (route_count + current_adds) * variants_per_route;

        // Subtract already consumed items
        let consumed = if self.exhausted_languages {
            1 + if let Some(ref lang_iter) = self.language_iter {
                self.languages.len() - lang_iter.len()
            } else {
                0
            }
        } else {
            0
        };

        let remaining = total.saturating_sub(consumed);
        (remaining, Some(remaining))
    }
}

impl<R, L> ExactSizeIterator for LocalizedRouteIter<R, L>
where
    R: IntoEnumIterator + Clone,
    L: LocalizedRoutable<BaseRoute = R>,
{
    fn len(&self) -> usize {
        self.size_hint().0
    }
}

impl<R, L> FusedIterator for LocalizedRouteIter<R, L>
where
    R: IntoEnumIterator + Clone,
    L: LocalizedRoutable<BaseRoute = R>,
{
}

impl<R, L> DoubleEndedIterator for LocalizedRouteIter<R, L>
where
    R: IntoEnumIterator + Clone + PartialEq,
    L: LocalizedRoutable<BaseRoute = R>,
    R::Iterator: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(ref mut iter) = self.route_iter {
            if let Some(route) = iter.next_back() {
                // First return the localized versions of this route (in reverse)
                let mut lang_variants = Vec::new();
                for &lang in self.languages.iter().rev() {
                    lang_variants.push(L::from_route(route.clone(), Some(lang)));
                }

                // Then add the default version
                lang_variants.push(L::from_route(route, None));

                // Return the items one by one when next_back is called
                return lang_variants.pop();
            }
        }
        None
    }
}
