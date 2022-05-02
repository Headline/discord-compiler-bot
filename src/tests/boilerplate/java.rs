use crate::boilerplate::generator::BoilerPlateGenerator;
#[cfg(test)]
use crate::boilerplate::java::JavaGenerator;

#[tokio::test]
async fn standard_needs_boilerplate() {
    let gen = JavaGenerator::new(
        "String str = \"test\";\n\
    System.out.println(str)",
    );
    assert!(gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate() {
    let gen = JavaGenerator::new(
        "class Test {\n\
    public static void main(String[] args) {\n\
    }\n\
    }\n",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate2() {
    let gen = JavaGenerator::new(
        "class Test {\n\
    public static void main (String[] args) {\n\
    }\n\
    }\n",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate3() {
    let gen = JavaGenerator::new(
        "class Test {\n\
    public static void main\t(String[] args) {\n\
    }\n\
    }\n",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate4() {
    let gen = JavaGenerator::new(
        "class Test {\n\
    public static void main                               (String[] args) {\n\
    }\n\
    }\n",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate5() {
    let gen = JavaGenerator::new(
        "class Test {\n\
    public static final void main                               (String[] args) {\n\
    }\n\
    }\n",
    );
    assert!(!gen.needs_boilerplate());
}
