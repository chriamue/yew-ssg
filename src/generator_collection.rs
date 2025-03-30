use crate::generator::Generator;
use std::collections::HashMap;
use std::error::Error;
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

    pub fn run_all(
        &self,
        route: &str,
        content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let mut results = HashMap::new();

        for generator in &self.generators {
            let result = generator.generate(route, content, metadata)?;
            results.insert(generator.name().to_string(), result);
        }

        Ok(results)
    }

    /// Returns the number of generators in the collection
    pub fn len(&self) -> usize {
        self.generators.len()
    }

    /// Returns true if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.generators.is_empty()
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
