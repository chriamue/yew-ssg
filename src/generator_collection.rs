use crate::generator::Generator;
use crate::processors::GeneratorOutputSupport;
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
            // Generate content using the generator's name as the key
            let name = generator.name();
            let result = generator.generate(name, route, content, metadata)?;
            results.insert(name.to_string(), result);

            // Check for additional output keys
            if let Some(support) = self.try_get_output_support(generator) {
                for key in support.supported_outputs() {
                    // Skip the main output we already processed
                    if key == name {
                        continue;
                    }

                    // Generate specific output for this key
                    match generator.generate(key, route, content, metadata) {
                        Ok(output) => {
                            results.insert(key.to_string(), output);
                        }
                        Err(_) => {
                            // Silently skip errors for additional outputs
                            // We already have the main output
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Try to extract GeneratorOutputSupport from a generator
    fn try_get_output_support<'a>(
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
