use regex::Regex;
use crate::boilerplate::generator::BoilerPlateGenerator;

pub struct CSharpGenerator {
    input : String
}

impl BoilerPlateGenerator for CSharpGenerator {
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
            if trimmed.starts_with("using") {
                header.push_str(&format!("{}\n", trimmed));
            }
            else {
                main_body.push_str(&format!("{}\n", trimmed))
            }
        }

        // if they included nothing, we can just manually include System since they probably want it
        if header.is_empty() {
            header.push_str("using System;");
        }
        format!("{}\nnamespace Main{{\nclass Program {{\n static void Main(string[] args) {{\n{}}}}}}}", header, main_body)
    }

    fn needs_boilerplate(&self) -> bool {
        let cpp_main_regex: Regex = Regex::new("\"[^\"]+\"|(?P<main>static[\\s]+?void[\\s]+?Main[\\s]*?\\()").unwrap();
        for m in cpp_main_regex.captures_iter(&self.input) {
            if let Some(_) = m.name("main") {
                return false;
            }
        }
        return true;
    }
}