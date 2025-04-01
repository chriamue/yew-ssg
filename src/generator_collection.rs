use crate::generator::Generator;
use crate::processors::GeneratorOutputSupport;
use std::slice::Iter;

#[derive(Debug, Clone)]
pub struct GeneratorCollection {
    pub(crate) generators: Vec<Box<dyn Generator>>,
}

impl GeneratorCollection {
    pub fn new() -> Self {
        Self {
            generators: Vec::new(),
        }
    }

    pub fn add<G: Generator + 'static>(&mut self, generator: G) {
        self.generators.push(Box::new(generator));
    }

    /// Try to extract GeneratorOutputSupport from a generator
    pub fn try_get_output_support<'a>(
        &self,
        generator: &'a Box<dyn Generator>,
    ) -> Option<&'a dyn GeneratorOutputSupport> {
        use crate::generators::{
            MetaTagGenerator, OpenGraphGenerator, RobotsMetaGenerator, TitleGenerator,
            TwitterCardGenerator,
        };

        if let Some(g) = generator.as_any().downcast_ref::<MetaTagGenerator>() {
            return Some(g);
        } else if let Some(g) = generator.as_any().downcast_ref::<OpenGraphGenerator>() {
            return Some(g);
        } else if let Some(g) = generator.as_any().downcast_ref::<RobotsMetaGenerator>() {
            return Some(g);
        } else if let Some(g) = generator.as_any().downcast_ref::<TitleGenerator>() {
            return Some(g);
        } else if let Some(g) = generator.as_any().downcast_ref::<TwitterCardGenerator>() {
            return Some(g);
        }

        // Testing support: mock generator
        #[cfg(test)]
        {
            if let Some(g) = generator
                .as_any()
                .downcast_ref::<crate::generator::tests::MockGenerator>()
            {
                return Some(g);
            }
        }

        None
    }

    /// Returns the number of generators in the collection
    pub fn len(&self) -> usize {
        self.generators.len()
    }

    /// Returns true if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.generators.len() == 0
    }

    /// Returns an iterator over the generators
    pub fn iter(&self) -> Iter<'_, Box<dyn Generator>> {
        self.generators.iter()
    }
}

impl IntoIterator for GeneratorCollection {
    type Item = Box<dyn Generator>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.generators.into_iter()
    }
}

impl<'a> IntoIterator for &'a GeneratorCollection {
    type Item = &'a Box<dyn Generator>;
    type IntoIter = Iter<'a, Box<dyn Generator>>;

    fn into_iter(self) -> Self::IntoIter {
        self.generators.iter()
    }
}
