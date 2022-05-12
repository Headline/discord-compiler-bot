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

#[tokio::test]
async fn standard_doesnt_need_boilerplate6() {
    let gen = JavaGenerator::new(
        "class Test {\n\
    static public final void main                               (String[] args) {\n\
    }\n\
    }\n",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate7() {
    let gen = JavaGenerator::new(
        "class Test {\n\
    static final public void main                               (String[] args) {\n\
    }\n\
    }\n",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate8() {
    let gen = JavaGenerator::new(
        "class Test {\n\
    final static public void main                               (String[] args) {\n\
    }\n\
    }\n",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate9() {
    let gen = JavaGenerator::new(
        "class Test {\n\
    final public static void main                               (String[] args) {\n\
    }\n\
    }\n",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate10() {
    let gen = JavaGenerator::new(
        "class Test {\n\
    public @SuppressWarnings(\"all\") final static void main ( final String args[] ) {\n\
    }\n\
    }\n",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate11() {
    let gen = JavaGenerator::new(
        "class Test {\n\
    public @SuppressWarnings(\"all\") static final void main ( final String args[] ) {\n\
    }\n\
    }\n",
    );
    assert!(!gen.needs_boilerplate());
}

#[tokio::test]
async fn standard_doesnt_need_boilerplate12() {
    let gen = JavaGenerator::new(
        "class Test {\n\
    public @SuppressWarnings(\"all\") static void main ( final String args[] ) {\n\
    }\n\
    }\n",
    );
    assert!(!gen.needs_boilerplate());
}
