use crate::generator::Generator;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct GeneratorCollection {
    generators: Vec<Box<dyn Generator>>,
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
}
