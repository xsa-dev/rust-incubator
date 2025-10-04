fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b' ' {
            return &s[..i];
        }
    }
    return &s[..];
}

fn main() {
    let s = String::from("hello world");
    println!("Первое слово: {}", first_word(&s));
}