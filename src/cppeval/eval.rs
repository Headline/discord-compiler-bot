use core::fmt;

#[derive(Debug, Clone)]
pub struct EvalError {
    details: String
}
impl EvalError {
    fn new(msg: &str) -> EvalError {
        EvalError{details: msg.to_string()}
    }
}
impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}
impl std::error::Error for EvalError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct CppEval {
    input : String,
    output : String
}

impl CppEval {
    pub fn new(input : &str) -> CppEval {
        CppEval {
            input: input.trim().to_owned(),
            output : String::default()
        }
    }

    pub fn evaluate(& mut self) -> Result<String, EvalError> {
        // allow inline code
        if self.input.starts_with('`') && self.input.ends_with('`') {
            self.input.remove(0);
            self.input.remove(self.input.len()-1);
            self.input = self.input.trim().to_string();
        }

        // add bits we need for every request
        self.add_headers();

        if self.input.starts_with('{') { // parsing a statement here
            if let Err(e) = self.do_statements() {
                return Err(e)
            }
        }
        else if self.input.starts_with("<<") { // just outputting
            self.do_prints();
        }
        else { // they're handling their own main
            if let Err(e) = self.do_user_handled() {
                return Err(e)
            }
        }


        Ok(self.output.clone())
    }

    fn do_user_handled(& mut self) -> Result<(), EvalError> {
        let re = regex::Regex::new(r"(([a-zA-Z]*?)[\s]+main\((.*?)\)[\s]+\{[\s\S]*?\})").unwrap();
        if let Some(capture) = re.captures_iter(&self.input).next() {
            let main = capture[1].trim().to_string();
            let rest = self.input.replacen(&main, "", 1).trim().to_string();

            self.output.push_str(&format!("{}\n", rest));
            self.output.push_str(&format!("{}\n", main));
        }
        else {
            return Err(EvalError::new("No main() specified. Invalid request"))

        }

        Ok(())
    }

    fn do_statements(& mut self) -> Result<(), EvalError>  {
        let end = self.get_statement_end();
        if end == 0 {
            return Err(EvalError::new("Parsing failure, detected unbalanced curly-brackets."))
        }

        self.do_rest(end+1);

        let statements = self.input[1..end].to_owned();
        self.build_main(&statements);

        Ok(())
    }

    fn get_statement_end(&self) -> usize {
        let mut balance = 0;
        let mut stop_idx = 0;
        let mut ignore = false;
        let mut inline_comment = false;
        let mut multiline_comment = false;
        let mut last = '\0';
        for (index, char) in self.input.chars().enumerate() {
            // prevent non-syntactic }'s from messing up our balance
            if (char == '\'' || char == '"') && last != '\\' {
                ignore = !ignore;
            }
            if ignore && last != '\\' {
                last = char;
                continue;
            }

            if char == '/' && last == '/' {
                inline_comment = true;
            }

            if inline_comment {
                if char == '\n' {
                    inline_comment = false;
                }
                last = char;
                continue;
            }

            /* awd */
            if char == '*' && last == '/' {
                multiline_comment = true;
            }

            if multiline_comment {
                if char == '/' && last == '*' {
                    multiline_comment = false;
                }
                last = char;
                continue;
            }

            // balance our braces
            if char == '{' {
                balance += 1;
            }
            if char == '}' {
                balance -= 1;
            }
            if balance == 0 {
                stop_idx = index;
                break;
            }
            last = char;
        }
        stop_idx
    }

    fn do_rest(& mut self, start_idx : usize) {
        let rest = &self.input[start_idx..];
        self.output.push_str(rest.trim());
    }

    fn do_prints(& mut self) {
        let input;
        if let Some(statement_end) = self.input.find(';') {
            self.do_rest(statement_end+1);

            input = self.input[..statement_end].to_owned();
        }
        else {
            input = self.input.clone();
        }
        self.build_main(&format!("cout {};", input));
    }

    fn add_headers(& mut self) {
        self.output.push_str("#include <bits/stdc++.h>\n");
        self.output.push_str("using namespace std;\n");
        //self.add_ostreaming();
    }

    fn build_main(& mut self, statements : &str) {
        self.output.push_str(&format!("\nint main (void) {{\n{}\n}}", statements));

    }

/*    fn add_ostreaming(& mut self) {
        let vec_print = include_str!("more_ostreaming.in");
        self.output.push_str(vec_print);
        self.output.push_str("\n\n");
    }
*/
}