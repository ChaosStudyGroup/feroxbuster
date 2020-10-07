mod utils;
use assert_cmd::prelude::*;
use httpmock::Method::GET;
use httpmock::{Mock, MockServer};
use predicates::prelude::*;
use std::process::Command;
use utils::{setup_tmp_directory, teardown_tmp_directory};

#[test]
/// send a single valid request, expect a 200 response
fn test_single_request_scan() -> Result<(), Box<dyn std::error::Error>> {
    let srv = MockServer::start();
    let (tmp_dir, file) = setup_tmp_directory(&["LICENSE".to_string()], "wordlist")?;

    let mock = Mock::new()
        .expect_method(GET)
        .expect_path("/LICENSE")
        .return_status(200)
        .return_body("this is a test")
        .create_on(&srv);

    let cmd = Command::cargo_bin("feroxbuster")
        .unwrap()
        .arg("--url")
        .arg(srv.url("/"))
        .arg("--wordlist")
        .arg(file.as_os_str())
        .arg("-vvvv")
        .unwrap();

    cmd.assert().success().stdout(
        predicate::str::contains("/LICENSE")
            .and(predicate::str::contains("200"))
            .and(predicate::str::contains("14")),
    );

    assert_eq!(mock.times_called(), 1);
    teardown_tmp_directory(tmp_dir);
    Ok(())
}

#[test]
/// send a valid request, follow redirects into new directories, expect 301/200 responses
fn scanner_recursive_request_scan() -> Result<(), Box<dyn std::error::Error>> {
    let srv = MockServer::start();
    let urls = [
        "js".to_string(),
        "prod".to_string(),
        "dev".to_string(),
        "file.js".to_string(),
    ];
    let (tmp_dir, file) = setup_tmp_directory(&urls, "wordlist")?;

    let js_mock = Mock::new()
        .expect_method(GET)
        .expect_path("/js")
        .return_status(301)
        .return_header("Location", &srv.url("/js/"))
        .create_on(&srv);

    let js_prod_mock = Mock::new()
        .expect_method(GET)
        .expect_path("/js/prod")
        .return_status(301)
        .return_header("Location", &srv.url("/js/prod/"))
        .create_on(&srv);

    let js_dev_mock = Mock::new()
        .expect_method(GET)
        .expect_path("/js/dev")
        .return_status(301)
        .return_header("Location", &srv.url("/js/dev/"))
        .create_on(&srv);

    let js_dev_file_mock = Mock::new()
        .expect_method(GET)
        .expect_path("/js/dev/file.js")
        .return_status(200)
        .return_body("this is a test and is more bytes than other ones")
        .create_on(&srv);

    let cmd = Command::cargo_bin("feroxbuster")
        .unwrap()
        .arg("--url")
        .arg(srv.url("/"))
        .arg("--wordlist")
        .arg(file.as_os_str())
        .arg("-vvvv")
        .arg("-t")
        .arg("1")
        .unwrap();

    cmd.assert().success().stdout(
        predicate::str::is_match("301.*js")
            .unwrap()
            .and(predicate::str::is_match("301.*js/prod").unwrap())
            .and(predicate::str::is_match("301.*js/dev").unwrap())
            .and(predicate::str::is_match("200.*js/dev/file.js").unwrap()),
    );

    assert_eq!(js_mock.times_called(), 1);
    assert_eq!(js_prod_mock.times_called(), 1);
    assert_eq!(js_dev_mock.times_called(), 1);
    assert_eq!(js_dev_file_mock.times_called(), 1);

    teardown_tmp_directory(tmp_dir);

    Ok(())
}

#[test]
/// send a single valid request, get a response, and write it to disk
fn scanner_single_request_scan_with_file_output() -> Result<(), Box<dyn std::error::Error>> {
    let srv = MockServer::start();
    let (tmp_dir, file) = setup_tmp_directory(&["LICENSE".to_string()], "wordlist")?;

    let mock = Mock::new()
        .expect_method(GET)
        .expect_path("/LICENSE")
        .return_status(200)
        .return_body("this is a test")
        .create_on(&srv);

    let outfile = tmp_dir.path().join("output");

    Command::cargo_bin("feroxbuster")
        .unwrap()
        .arg("--url")
        .arg(srv.url("/"))
        .arg("--wordlist")
        .arg(file.as_os_str())
        .arg("-vvvv")
        .arg("-o")
        .arg(outfile.as_os_str())
        .unwrap();

    let contents = std::fs::read_to_string(outfile)?;

    assert!(contents.contains("/LICENSE"));
    assert!(contents.contains("200"));
    assert!(contents.contains("14"));

    assert_eq!(mock.times_called(), 1);
    teardown_tmp_directory(tmp_dir);
    Ok(())
}
