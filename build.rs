fn main() {
    #[cfg(feature = "add_path")]
    {
        use std::io::Write;
        use std::path::PathBuf;

        if std::env::var("CI").unwrap_or("false".to_string()) == "true" {
            // echo "$HOME/.jas/bin" >> $GITHUB_PATH
            let path = match std::env::var("GITHUB_PATH") {
                Ok(path) => path,
                Err(_) => return,
            };

            let home = match std::env::var("HOME") {
                Ok(home) => home,
                Err(_) => return,
            };
            let bin_dir = PathBuf::from(home).join(".jas").join("bin");
            let text = bin_dir.to_str().unwrap().as_bytes();
            let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
            file.write_all(text).unwrap();
        }
    }
}
