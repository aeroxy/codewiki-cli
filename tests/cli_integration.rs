use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;

#[test]
fn help_shows_usage() {
    let mut cmd = cargo_bin_cmd!("codewiki");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("Query GitHub repo wikis via Google Code Wiki"));
}

#[test]
fn ask_command_prints_mocked_output() {
    let mut cmd = cargo_bin_cmd!("codewiki");
    cmd.env("CODEWIKI_MOCK_TEXT", "mocked answer")
        .args(["ask", "facebook/react", "How does useEffect work?"])
        .assert()
        .success()
        .stdout(contains("## CodeWiki: facebook/react (ask)"))
        .stdout(contains("mocked answer"));
}

#[test]
fn structure_command_prints_mocked_output() {
    let mut cmd = cargo_bin_cmd!("codewiki");
    cmd.env("CODEWIKI_MOCK_TEXT", "topic1\ntopic2")
        .args(["structure", "facebook/react"])
        .assert()
        .success()
        .stdout(contains("## CodeWiki: facebook/react (structure)"))
        .stdout(contains("topic1"));
}

#[test]
fn read_command_prints_mocked_output() {
    let mut cmd = cargo_bin_cmd!("codewiki");
    cmd.env("CODEWIKI_MOCK_TEXT", "full wiki content")
        .args(["read", "facebook/react"])
        .assert()
        .success()
        .stdout(contains("## CodeWiki: facebook/react (read)"))
        .stdout(contains("full wiki content"));
}

#[test]
fn ask_missing_question_fails() {
    let mut cmd = cargo_bin_cmd!("codewiki");
    cmd.args(["ask", "facebook/react"])
        .assert()
        .failure()
        .stderr(contains("Usage:"));
}
