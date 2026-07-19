#[cfg(test)]
use crate::boilerplate::generator::BoilerPlateGenerator;
use crate::boilerplate::sourcepawn::SourcePawnGenerator;

#[tokio::test]
async fn standard_needs_boilerplate() {
    let gen = SourcePawnGenerator::new("printf(\"hello %d\\n\", 4);");
    assert!(gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate() {
    let gen = SourcePawnGenerator::new(
        "#include <shell>\n\
    int main() {\n\
    printf(\"hello\\n\");\n\
    return 0;\n\
    }",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn main_in_string_needs_boilerplate() {
    let gen = SourcePawnGenerator::new("print(\"main(\");");
    assert!(gen.needs_boilerplate());
}

#[tokio::test]
async fn generates_shell_include_and_main() {
    let gen = SourcePawnGenerator::new("printnum(42);");
    let output = gen.generate();
    assert!(output.contains("#include <shell>"));
    assert!(output.contains("int main() {"));
    assert!(output.contains("printnum(42);"));
    assert!(output.contains("return 0;"));
}

#[tokio::test]
async fn hoists_includes_without_duplicating_shell() {
    let gen = SourcePawnGenerator::new("#include <shell>\nprintnum(42);");
    let output = gen.generate();
    assert_eq!(output.matches("#include <shell>").count(), 1);
    // the include must end up above main, not inside it
    let include_pos = output.find("#include <shell>").unwrap();
    let main_pos = output.find("int main()").unwrap();
    assert!(include_pos < main_pos);
}
