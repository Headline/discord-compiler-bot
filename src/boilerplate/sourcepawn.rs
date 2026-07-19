use std::fmt::Write as _;

use crate::boilerplate::generator::BoilerPlateGenerator;
use crate::utls::constants::C_LIKE_MAIN_REGEX;

pub struct SourcePawnGenerator {
    input: String,
}

impl BoilerPlateGenerator for SourcePawnGenerator {
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
            if trimmed.starts_with("#i") || trimmed.starts_with("#d") {
                writeln!(header, "{}", trimmed).unwrap();
            } else {
                writeln!(main_body, "{}", trimmed).unwrap();
            }
        }

        // spshell's natives (printf, printnum, ...) all live in shell.inc
        if !header.contains("<shell>") {
            header.insert_str(0, "#include <shell>\n");
        }

        format!("{}\nint main() {{\n{}return 0;\n}}", header, main_body)
    }

    fn needs_boilerplate(&self) -> bool {
        for m in C_LIKE_MAIN_REGEX.captures_iter(&self.input) {
            if m.name("main").is_some() {
                return false;
            }
        }

        true
    }
}
