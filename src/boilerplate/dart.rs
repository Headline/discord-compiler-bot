use std::fmt::Write as _;

use crate::boilerplate::generator::BoilerPlateGenerator;
// dart uses the same pattern as java
use crate::utls::constants::JAVA_MAIN_REGEX;

pub struct DartGenerator {
    input: String,
}

impl BoilerPlateGenerator for DartGenerator {
    fn new(input: &str) -> Self {
        let mut formated = input.to_string();
        formated = formated.replace(';', ";\n"); // separate lines by ;

        Self { input: formated }
    }

    fn generate(&self) -> String {
        let mut main_body = String::default();
        let mut header = String::default();

        let lines = self.input.split('\n');
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("import") {
                writeln!(header, "{}", trimmed).unwrap();
            } else {
                writeln!(main_body, "{}", trimmed).unwrap();
            }
        }

        format!(
            "{}\nvoid main(List<String> args) {{\n{}}}",
            header, main_body
        )
    }

    fn needs_boilerplate(&self) -> bool {
        for m in JAVA_MAIN_REGEX.captures_iter(&self.input) {
            if m.name("main").is_some() {
                return false;
            }
        }
        true
    }
}
