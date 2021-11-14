#[cfg(test)]

use serenity::model::user::User;
use crate::utls::parser::get_components;
use crate::managers::compilation::CompilationManager;
use tokio::sync::RwLock;
use std::sync::Arc;

#[tokio::test]
async fn standard_parse() {
    let dummy_user = User::default();
    let input = indoc::indoc!(
        ";compile c++
        ```
        int main() {}
        ```"
    );

    let reply = None;
    let result = get_components(input, &dummy_user, None, &reply).await;
    if let Err(_) = &result {
        assert!(false, "Parser failed.");
    }

    let parser_result = result.unwrap();
    assert_eq!(parser_result.target, "c++");
    assert_eq!(parser_result.args.len(), 0);
    assert_eq!(parser_result.options.len(), 0);
    assert_eq!(parser_result.stdin, "");
    assert_eq!(parser_result.url, "");
    assert_eq!(parser_result.code, "int main() {}\n");
}

#[tokio::test]
async fn standard_parse_args() {
    let dummy_user = User::default();
    let input = indoc::indoc!(
        ";compile c++ -O3 -Wall -Werror
        ```
        int main() {}
        ```"
    );

    let reply = None;
    let result = get_components(input, &dummy_user, None, &reply).await;
    if let Err(_) = &result {
        assert!(false, "Parser failed.");
    }

    let parser_result = result.unwrap();
    assert_eq!(parser_result.target, "c++");
    assert_eq!(parser_result.args.len(), 0);
    assert_eq!(parser_result.options, ["-O3", "-Wall", "-Werror"]);
    assert_eq!(parser_result.stdin, "");
    assert_eq!(parser_result.url, "");
    assert_eq!(parser_result.code, "int main() {}\n");
}

#[tokio::test]
async fn standard_parse_url() {
    let dummy_user = User::default();
    let input = indoc::indoc!(
        ";compile c++ < https://pastebin.com/raw/ERqDRZva"
    );

    let reply = None;
    let result = get_components(input, &dummy_user, None, &reply).await;
    if let Err(_) = &result {
        assert!(false, "Parser failed.");
    }

    let parser_result = result.unwrap();
    assert_eq!(parser_result.target, "c++");
    assert_eq!(parser_result.args.len(), 0);
    assert_eq!(parser_result.options.len(), 0);
    assert_eq!(parser_result.stdin, "");
    assert_eq!(parser_result.url, "https://pastebin.com/raw/ERqDRZva");
    assert_eq!(parser_result.code, "int main() {}");
}

#[tokio::test]
async fn standard_parse_stdin() {
    let dummy_user = User::default();
    let input = indoc::indoc!(
        ";compile c++ | testing 1 2 3
        ```
        int main() {}
        ```"
    );

    let reply = None;
    let result = get_components(input, &dummy_user, None, &reply).await;
    if let Err(_) = &result {
        assert!(false, "Parser failed.");
    }

    let parser_result = result.unwrap();
    assert_eq!(parser_result.target, "c++");
    assert_eq!(parser_result.args.len(), 0);
    assert_eq!(parser_result.options.len(), 0);
    assert_eq!(parser_result.stdin, "testing 1 2 3");
    assert_eq!(parser_result.url, "");
    assert_eq!(parser_result.code, "int main() {}\n");
}

#[tokio::test]
async fn standard_parse_block_stdin() {
    let dummy_user = User::default();
    let input = indoc::indoc!(
        ";compile c++
        ```
        testing 1 2 3
        ```
        ```
        int main() {}
        ```"
    );

    let reply = None;
    let result = get_components(input, &dummy_user, None, &reply).await;
    if let Err(_) = &result {
        assert!(false, "Parser failed.");
    }

    let parser_result = result.unwrap();
    assert_eq!(parser_result.target, "c++");
    assert_eq!(parser_result.args.len(), 0);
    assert_eq!(parser_result.options.len(), 0);
    assert_eq!(parser_result.stdin, "testing 1 2 3\n");
    assert_eq!(parser_result.url, "");
    assert_eq!(parser_result.code, "int main() {}\n");
}

#[tokio::test]
async fn standard_parse_deduce_compiler() {
    let dummy_user = User::default();
    let input = indoc::indoc!(
        ";compile c++
        ```
        int main() {}
        ```"
    );

    let reply = None;
    let cm = Arc::new(RwLock::new(CompilationManager::new().await.unwrap()));
    let result = get_components(input, &dummy_user, Some(&cm), &reply).await;
    if let Err(_) = &result {
        assert!(false, "Parser failed.");
    }

    let parser_result = result.unwrap();
    assert_eq!(parser_result.target, "c++");
    assert_eq!(parser_result.args.len(), 0);
    assert_eq!(parser_result.options.len(), 0);
    assert_eq!(parser_result.stdin, "");
    assert_eq!(parser_result.url, "");
    assert_eq!(parser_result.code, "int main() {}\n");
}

#[tokio::test]
async fn standard_parse_deduce_compiler_upper_case() {
    let dummy_user = User::default();
    let input = indoc::indoc!(
        ";compile JAVASCRIPT
        ```
        console.log(\"beehee\");
        ```"
    );

    let reply = None;
    let cm = Arc::new(RwLock::new(CompilationManager::new().await.unwrap()));
    let result = get_components(input, &dummy_user, Some(&cm), &reply).await;
    if let Err(_) = &result {
        assert!(false, "Parser failed.");
    }

    let parser_result = result.unwrap();
    assert_eq!(parser_result.target, "javascript");
    assert_eq!(parser_result.args.len(), 0);
    assert_eq!(parser_result.options.len(), 0);
    assert_eq!(parser_result.stdin, "");
    assert_eq!(parser_result.url, "");
    assert_eq!(parser_result.code, "console.log(\"beehee\");\n");
}

#[tokio::test]
async fn standard_parse_late_deduce_compiler() {
    let dummy_user = User::default();
    let input = indoc::indoc!(
        ";compile
        ```js
        console.log(\"beehee\");
        ```"
    );

    let reply = None;
    let cm = Arc::new(RwLock::new(CompilationManager::new().await.unwrap()));
    let result = get_components(input, &dummy_user, Some(&cm), &reply).await;
    if let Err(_) = &result {
        assert!(false, "Parser failed.");
    }

    let parser_result = result.unwrap();
    assert_eq!(parser_result.target, "javascript");
    assert_eq!(parser_result.args.len(), 0);
    assert_eq!(parser_result.options.len(), 0);
    assert_eq!(parser_result.stdin, "");
    assert_eq!(parser_result.url, "");
    assert_eq!(parser_result.code, "console.log(\"beehee\");\n");
}

#[tokio::test]
async fn standard_parse_late_deduce_compiler_block_stdin() {
    let dummy_user = User::default();
    let input = indoc::indoc!(
        ";compile
        ```
        testing 1 2 3
        ```
        ```js
        console.log(\"beehee\");
        ```"
    );

    let reply = None;
    let cm = Arc::new(RwLock::new(CompilationManager::new().await.unwrap()));
    let result = get_components(input, &dummy_user, Some(&cm), &reply).await;
    if let Err(_) = &result {
        assert!(false, "Parser failed.");
    }

    let parser_result = result.unwrap();
    assert_eq!(parser_result.target, "javascript");
    assert_eq!(parser_result.args.len(), 0);
    assert_eq!(parser_result.options.len(), 0);
    assert_eq!(parser_result.stdin, "testing 1 2 3\n");
    assert_eq!(parser_result.url, "");
    assert_eq!(parser_result.code, "console.log(\"beehee\");\n");
}

#[tokio::test]
async fn standard_parse_one_line() {
    let dummy_user = User::default();
    let input = indoc::indoc!(
        ";compile js ```console.log(\"beehee\");```"
    );

    let reply = None;
    let cm = Arc::new(RwLock::new(CompilationManager::new().await.unwrap()));
    let result = get_components(input, &dummy_user, Some(&cm), &reply).await;
    if let Err(_) = &result {
        assert!(false, "Parser failed.");
    }

    let parser_result = result.unwrap();
    assert_eq!(parser_result.target, "javascript");
    assert_eq!(parser_result.args.len(), 0);
    assert_eq!(parser_result.options.len(), 0);
    assert_eq!(parser_result.stdin, "");
    assert_eq!(parser_result.url, "");
    assert_eq!(parser_result.code, "console.log(\"beehee\");");
}

#[tokio::test]
async fn standard_parse_args_one_line() {
    let dummy_user = User::default();
    let input = indoc::indoc!(
        ";compile c -O3```int main() {return 232;}```"
    );

    let reply = None;
    let result = get_components(input, &dummy_user, None, &reply).await;
    if let Err(_) = &result {
        assert!(false, "Parser failed.");
    }

    let parser_result = result.unwrap();
    assert_eq!(parser_result.target, "c");
    assert_eq!(parser_result.args.len(), 0);
    assert_eq!(parser_result.options, ["-O3"]);
    assert_eq!(parser_result.stdin, "");
    assert_eq!(parser_result.url, "");
    assert_eq!(parser_result.code, "int main() {return 232;}");
}