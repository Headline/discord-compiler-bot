//! Integration tests for the SourcePawn service client. These require a
//! running sourcepawn-api instance (SOURCEPAWN_API_URL) and are ignored by
//! default; run with `cargo test sourcepawn -- --ignored`.

use crate::apis::sourcepawn::SourcePawnService;

async fn service() -> SourcePawnService {
    let url = std::env::var("SOURCEPAWN_API_URL")
        .unwrap_or_else(|_| String::from("http://localhost:8080"));
    SourcePawnService::new(reqwest::Client::new(), &url)
        .await
        .expect("sourcepawn-api is not reachable")
}

#[tokio::test]
#[ignore]
async fn compiles_and_executes() {
    let sp = service().await;
    assert!(sp.compiler_name().starts_with("spcomp "));

    let code = "#include <shell>\nint main() { printnum(42); return 0; }";
    let response = sp.compile(code, true, false).await.unwrap();

    assert!(response.compile.success);
    let run = response.run.expect("run stage missing");
    assert!(run.success);
    assert_eq!(run.stdout, "42\n");
    assert!(response.asm.is_none());
}

#[tokio::test]
#[ignore]
async fn produces_assembly() {
    let sp = service().await;

    let code = "#include <shell>\nint main() { return 7; }";
    let response = sp.compile(code, false, true).await.unwrap();

    assert!(response.compile.success);
    assert!(response.run.is_none());
    let asm = response.asm.expect("asm stage missing");
    assert!(asm.success);
    assert!(asm.stdout.contains(".method main"));
}

#[tokio::test]
#[ignore]
async fn boilerplate_output_executes() {
    use crate::boilerplate::generator::boilerplate_factory;

    let generated = {
        let generator = boilerplate_factory("sourcepawn", "printnum(42);");
        assert!(generator.needs_boilerplate());
        generator.generate()
    };

    let sp = service().await;
    let response = sp.compile(&generated, true, false).await.unwrap();

    assert!(response.compile.success);
    let run = response.run.expect("run stage missing");
    assert!(run.success);
    assert_eq!(run.stdout, "42\n");
}

#[tokio::test]
#[ignore]
async fn reports_compile_errors() {
    let sp = service().await;

    let response = sp
        .compile("int main() { return undefined; }", true, false)
        .await
        .unwrap();

    assert!(!response.compile.success);
    assert!(response.compile.stdout.contains("error"));
    assert!(response.run.is_none());
}
