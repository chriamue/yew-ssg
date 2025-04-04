use crate::processor::Processor;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct ProcessorCollection {
    processors: Vec<Box<dyn Processor>>,
}

impl ProcessorCollection {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn add<P: Processor + 'static>(&mut self, processor: P) {
        self.processors.push(Box::new(processor));
    }

    pub fn process_all(
        &self,
        html: &str,
        metadata: &HashMap<String, String>,
        generator_outputs: &HashMap<String, String>,
        content: &str,
    ) -> Result<String, Box<dyn Error>> {
        let mut result = html.to_string();

        for processor in &self.processors {
            result = processor.process(&result, metadata, generator_outputs, content)?;
        }

        Ok(result)
    }

    // Standard collection methods
    pub fn len(&self) -> usize {
        self.processors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.processors.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Box<dyn Processor>> {
        self.processors.iter()
    }
}

// Iterator implementations
impl<'a> IntoIterator for &'a ProcessorCollection {
    type Item = &'a Box<dyn Processor>;
    type IntoIter = std::slice::Iter<'a, Box<dyn Processor>>;

    fn into_iter(self) -> Self::IntoIter {
        self.processors.iter()
    }
}
