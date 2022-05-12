#[cfg(test)]
use crate::boilerplate::cpp::CppGenerator;
use crate::boilerplate::generator::BoilerPlateGenerator;

#[tokio::test]
async fn standard_needs_boilerplate() {
    let gen = CppGenerator::new(
        "std::string str = \"test\";\n\
    std::cout << str;",
    );
    assert!(gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate() {
    let gen = CppGenerator::new(
        "int main() {\n\
    std::string str = \"test\";\n\
    std::cout << str;\n\
    return 0;\n\
    }",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate2() {
    let gen = CppGenerator::new(
        "int main (    ) {\n\
    std::string str = \"test\";\n\
    std::cout << str;\n\
    return 0;\n\
    }",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate3() {
    let gen = CppGenerator::new(
        "int main\n(void) {\n\
    std::string str = \"test\";\n\
    std::cout << str;\n\
    return 0;\n\
    }",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate4() {
    let gen = CppGenerator::new(
        "void main    (void) {\n\
    std::string str = \"test\";\n\
    std::cout << str;\n\
    return 0;\n\
    }",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn no_bplate_emptystrlit() {
    let gen = CppGenerator::new(
        "#include \"stdio.h\"
        int x = sizeof \"\" + '0';
        int main() {
          printf(\"%d\n\", x);
        }",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn no_bplate_narfd_main() {
    let gen = CppGenerator::new(
        "#include \"stdio.h\"
            #define NARF void
            #define ZORT signed
            int x = '1';
            ZORT main (NARF) {
              printf(\"%d\n\", x);
            }",
    );
    assert!(!gen.needs_boilerplate());
}
