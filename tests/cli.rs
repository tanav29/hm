use std::{
    io::Write,
    process::{Command, Stdio},
};

fn run(input: &str, args: &[&str]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_hm"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn mixed_content_and_stdout() {
    let result = run("p { \"Hello \" strong \"world\" \"!\" }", &["--fragment"]);
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "<p>Hello <strong>world</strong>!</p>\n"
    );
}

#[test]
fn custom_title_is_escaped_and_omitted_from_fragments() {
    let result = run("p \"Hello\"", &["--title", "A & <B>"]);
    assert!(result.status.success());
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("<title>A &amp; &lt;B&gt;</title>")
    );
    let fragment = run("p \"Hello\"", &["--fragment", "--title", "Example"]);
    assert_eq!(
        String::from_utf8(fragment.stdout).unwrap(),
        "<p>Hello</p>\n"
    );
    assert!(!run("", &["--title"]).status.success());
}

#[test]
fn title_declaration_and_cli_precedence() {
    let source = "// comment\ntitle \"A & <B>\" // title\np \"Hello\"";
    let result = run(source, &[]);
    assert!(result.status.success());
    let html = String::from_utf8(result.stdout).unwrap();
    assert!(html.contains("<title>A &amp; &lt;B&gt;</title>"));
    assert_eq!(html.matches("<title>").count(), 1);
    let overridden = run(source, &["--title", "Override"]);
    assert!(
        String::from_utf8(overridden.stdout)
            .unwrap()
            .contains("<title>Override</title>")
    );
    let fragment = run(source, &["--fragment"]);
    assert_eq!(
        String::from_utf8(fragment.stdout).unwrap(),
        "<p>Hello</p>\n"
    );
    assert!(!run("title \"unterminated", &[]).status.success());
}

#[test]
fn rejects_invalid_input_and_excessive_depth() {
    for input in [
        "IMG \"content\"".to_string(),
        "div[1bad=\"x\"]".to_string(),
        "div {".repeat(130),
    ] {
        let result = run(&input, &["--fragment"]);
        assert!(!result.status.success(), "accepted {input}");
        assert!(result.stdout.is_empty());
    }
}

#[test]
fn checked_in_examples_match() {
    for entry in std::fs::read_dir("examples").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|x| x == "hm") {
            let result = run(&std::fs::read_to_string(&path).unwrap(), &[]);
            assert!(
                result.status.success(),
                "{}: {}",
                path.display(),
                String::from_utf8_lossy(&result.stderr)
            );
            assert_eq!(
                result.stdout,
                std::fs::read(path.with_extension("html")).unwrap(),
                "{}",
                path.display()
            );
        }
    }
}
