use std::collections::VecDeque;
use std::fs;
use std::io::Write;

pub fn save(
    path: &str,
    file_name: &str,
    bytes: &[u8],
    name_modifiers: Vec<&str>,
    ext_modifiers: Vec<&str>,
) {
    match fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(format!(
            "{}/{}{}{}",
            path,
            name_modifiers
                .iter()
                .map(|s| format!("{}_", s.to_lowercase()))
                .collect::<String>(),
            file_name,
            ext_modifiers
                .iter()
                .map(|s| format!(".{}", s.to_lowercase()))
                .collect::<String>()
        )) {
        Ok(mut file) => {
            file.write_all(bytes).expect("Couldn't write to file");
            println!(
                "Wrote {} wasm to file",
                name_modifiers
                    .iter()
                    .map(|s| str::to_uppercase(s))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        Err(e) => println!("Error: {}", e),
    };
}

pub fn hexify_bytes(bytes: Vec<u8>) -> Vec<u8> {
    let mut hexified_bytes = bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
        .bytes()
        .collect::<VecDeque<_>>();
    hexified_bytes.push_front(b'x');
    hexified_bytes.push_front(b'0');

    hexified_bytes.into()
}
