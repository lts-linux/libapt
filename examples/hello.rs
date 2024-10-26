fn message() -> String {
    "Hello from an example!".to_string()
}

fn main() {
    println!("{}", message());
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_cmd::prelude::*;
    use std::process::Command;

    #[test]
    fn hello_message() {
        assert_eq!(message(), "Hello from an example!");
    }

    #[test]
    fn hello() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("hello")?;

        cmd.assert().stdout("Hello from an example!\n");

        Ok(())
    }
}
