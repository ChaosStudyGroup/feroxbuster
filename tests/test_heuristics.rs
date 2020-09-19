mod utils;
use assert_cmd::prelude::*;
use assert_cmd::Command;
use httpmock::Method::GET;
use httpmock::{Mock, MockServer, Regex};
use predicates::prelude::*;
use utils::{setup_tmp_directory, teardown_tmp_directory};

#[test]
/// test passes one bad target via -u to the scanner, expected result is that the
/// scanner dies
fn test_single_target_cannot_connect() -> Result<(), Box<dyn std::error::Error>> {
    let (tmp_dir, file) = setup_tmp_directory(&["LICENSE".to_string()])?;

    let cmd = std::panic::catch_unwind(|| {
        Command::cargo_bin("feroxbuster")
            .unwrap()
            .arg("--url")
            .arg("http://fjdksafjkdsajfkdsajkfdsajkfsdjkdsfdsafdsafdsajkr3l2ajfdskafdsjk")
            .arg("--wordlist")
            .arg(file.as_os_str())
            .unwrap()
    });

    assert!(cmd.is_err());

    teardown_tmp_directory(tmp_dir);
    Ok(())
}

#[test]
/// test pipes two bad targets to the scanner, expected result is that the
/// scanner dies
fn test_two_targets_cannot_connect() -> Result<(), Box<dyn std::error::Error>> {
    let not_real =
        String::from("http://fjdksafjkdsajfkdsajkfdsajkfsdjkdsfdsafdsafdsajkr3l2ajfdskafdsjk");
    let urls = vec![not_real.clone(), not_real];
    let (tmp_dir, file) = setup_tmp_directory(&urls)?;

    let cmd = std::panic::catch_unwind(|| {
        Command::cargo_bin("feroxbuster")
            .unwrap()
            .arg("--stdin")
            .arg("--wordlist")
            .arg(file.as_os_str())
            .pipe_stdin(file)
            .unwrap()
            .unwrap()
    });

    assert!(cmd.is_err());

    teardown_tmp_directory(tmp_dir);
    Ok(())
}

#[test]
/// test pipes one good target and one bad to the scanner, expected result is that the
/// good target is scanned successfully while the bad target is ignored and handled properly
fn test_one_good_and_one_bad_target_scan_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    let srv = MockServer::start();

    let not_real =
        String::from("http://fjdksafjkdsajfkdsajkfdsajkfsdjkdsfdsafdsafdsajkr3l2ajfdskafdsjk");
    let urls = vec![
        not_real,
        srv.url("/"),
        String::from("LICENSE"),
    ];
    let (tmp_dir, file) = setup_tmp_directory(&urls)?;

    let mock = Mock::new()
        .expect_method(GET)
        .expect_path("/LICENSE")
        .return_status(200)
        .return_body("this is a test")
        .create_on(&srv);

    let mut cmd = Command::cargo_bin("feroxbuster").unwrap();

    cmd.arg("--stdin")
        .arg("--wordlist")
        .arg(file.as_os_str())
        .pipe_stdin(file)
        .unwrap()
        .assert()
        .success()
        .stdout(
            predicate::str::contains("/LICENSE")
                .and(predicate::str::contains("200 OK"))
                .and(predicate::str::contains("[14 bytes]")),
        );
    assert_eq!(mock.times_called(), 1);

    teardown_tmp_directory(tmp_dir);
    Ok(())
}

#[test]
fn test_wildcard_request_found() -> Result<(), Box<dyn std::error::Error>> {
    let srv = MockServer::start();
    let (tmp_dir, file) = setup_tmp_directory(&["LICENSE".to_string()])?;

    let mock = Mock::new()
        .expect_method(GET)
        .expect_path_matches(Regex::new("/[a-zA-Z0-9]{32}/").unwrap())
        .return_status(200)
        .return_body("this is a test")
        .create_on(&srv);

    let cmd = Command::cargo_bin("feroxbuster")
        .unwrap()
        .arg("--url")
        .arg(srv.url("/"))
        .arg("--wordlist")
        .arg(file.as_os_str())
        .arg("--addslash")
        .unwrap();

    cmd.assert().success().stdout(
        predicate::str::contains("WILDCARD")
            .and(predicate::str::contains("Received"))
            .and(predicate::str::contains("200 OK"))
            .and(predicate::str::contains("(14 bytes)"))
    );

    assert_eq!(mock.times_called(), 1);
    teardown_tmp_directory(tmp_dir);
    Ok(())
}
