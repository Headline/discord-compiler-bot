use crate::boilerplate::generator::BoilerPlateGenerator;
use crate::utls::constants::JAVA_MAIN_REGEX;

pub struct JavaGenerator {
    input : String
}

impl BoilerPlateGenerator for JavaGenerator {
    fn new(input: &str) -> Self {
        let mut formated = input.to_string();
        formated = formated.replace(";", ";\n"); // separate lines by ;

        Self {
            input : formated
        }
    }

    fn generate(&self) -> String {
        let mut main_body = String::default();
        let mut header = String::default();

        let mut lines = self.input.split("\n");
        while let Some(line) = lines.next() {
            let trimmed = line.trim();
            if trimmed.starts_with("import") {
                header.push_str(&format!("{}\n", trimmed));
            }
            else {
                main_body.push_str(&format!("{}\n", trimmed))
            }
        }

        format!("{}\nclass Main{{\npublic static void main(String[] args) {{\n{}}}}}", header, main_body)
    }

    fn needs_boilerplate(&self) -> bool {
        for m in JAVA_MAIN_REGEX.captures_iter(&self.input) {
            if let Some(_) = m.name("main") {
                return false;
            }
        }
        return true;
    }
}