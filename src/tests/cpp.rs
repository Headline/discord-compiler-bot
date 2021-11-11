use crate::cppeval::eval;

#[tokio::test]
async fn eval_output() {
    let mut eval = eval::CppEval::new("<< \"test\"");
    assert!(eval.evaluate().is_ok());
}

#[tokio::test]
async fn eval_output_implicit_main() {
    let mut eval = eval::CppEval::new("{cout << \"test\"}");
    assert!(eval.evaluate().is_ok());
}

#[tokio::test]
async fn eval_output_explicit_main() {
    let mut eval = eval::CppEval::new("int main(void)\n{cout << \"test\";}");
    assert!(eval.evaluate().is_ok());
}

#[tokio::test]
async fn eval_output_balance_brace() {
    let mut eval = eval::CppEval::new("{cout << \"{{{{{\";}");
    assert!(eval.evaluate().is_ok());
}

#[tokio::test]
async fn eval_output_balance_brace2() {
    let mut eval = eval::CppEval::new("{cout << \"}}}}}\";}");
    assert!(eval.evaluate().is_ok());
}

#[tokio::test]
async fn eval_output_balance_brace_fail() {
    let mut eval = eval::CppEval::new("{ {cout << \"}}}}}\";}");
    assert!(eval.evaluate().is_err()); // expecting error
}

#[tokio::test]
async fn eval_output_custom_func() {
    let mut eval = eval::CppEval::new("<< f(2); int f(int a) { return a * 4; }");
    assert!(eval.evaluate().is_ok());
}

#[tokio::test]
async fn eval_output_discord_escape() {
    let mut eval = eval::CppEval::new("`<< f(2); int f(int a) { return a * 4; }`");
    assert!(eval.evaluate().is_ok());
}

#[tokio::test]
async fn eval_output_conditional() {
    let mut eval = eval::CppEval::new("{ int a = 4; if (a > 3) { cout << \"true\"; } }");
    assert!(eval.evaluate().is_ok());
}
