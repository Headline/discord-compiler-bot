use crate::boilerplate::generator::BoilerPlateGenerator;
use crate::utls::constants::PHP_START_REGEX;

pub struct PHPGenerator {
    input: String,
}

impl BoilerPlateGenerator for PHPGenerator {
    fn new(input: &str) -> Self {
        let mut formated = input.to_string();
        formated = formated.replace(';', ";\n"); // separate lines by ;

        Self { input: formated }
    }

    fn generate(&self) -> String {
        format!("<?php\n{}", self.input)
    }

    fn needs_boilerplate(&self) -> bool {
        for m in PHP_START_REGEX.captures_iter(&self.input) {
            if m.name("php_start").is_some() {
                return false;
            }
        }
        true
    }
}
