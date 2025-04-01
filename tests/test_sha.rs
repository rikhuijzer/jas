use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("jas").unwrap()
}

#[test]
fn test_sha() {
    let mut cmd = bin();
    cmd.arg("sha").arg("123");
}
