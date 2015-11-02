extern crate hyper;

mod testutil;
use testutil::{run_example, read_url};

#[test]
fn basic_example() {
    run_example("example", |port| {
        let url = format!("http://localhost:{}", port);
        let s = read_url(&url);

        assert_eq!(s, "Hello World\n\n");
    })
}

#[test]
fn with_layout() {
    run_example("with_layout", |port| {
        let url = format!("http://localhost:{}/no_helpers", port);
        let s = read_url(&url);
        // FIXME: Should the first trailing newline be stripped from renders?
        assert_eq!(s, "**Before\nHello World\n\n\nAfter**\n");
    })
}

#[test]
fn helpers_with_layout() {
    run_example("with_layout", |port| {
        let url = format!("http://localhost:{}/with_helpers", port);
        let s = read_url(&url);
        // FIXME: Should the first trailing newline be stripped from renders?
        assert_eq!(s, "**Before\nHello World\n\n<b>Hello World from a helper \
                       function</b>\nAfter**\n");
    })
}

#[test]
fn caching_example_runs() {
    run_example("caching", |port| {
        let url = format!("http://localhost:{}", port);
        let s = read_url(&url);

        assert_eq!(s, "Hello Cache?\n\n");
    })
}

#[test]
fn helper_functions() {
    run_example("helper_functions", |port| {
        let url = format!("http://localhost:{}", port);
        let s = read_url(&url);

        assert_eq!(s,
r#"Hello World

<b>Hello World from a helper function</b>"#);
    })
}

#[test]
fn path_adjustment() {
    run_example("path_adjustment", |port| {
        let url = format!("http://localhost:{}", port);
        let s = read_url(&url);
        assert_eq!(s, "Hello World\n\n");
    })
}

#[test]
fn path_adjustment_no_file() {
    use std::fs::metadata;
    use std::env;

    println!("{:?}", env::current_dir());

    // ensure the file doesn't exist without the adjusted path (so we're not rendering that one)
    assert!(metadata("my_template").is_err());
    assert!(metadata("my_template.mustache").is_err());

    // sanity check - adjusted path exists
    assert!(metadata("examples/assets/my_template.mustache").is_ok());
}
