use std::collections::HashMap;

fn word_count(text: &str) -> HashMap<&str, u32> {
    let mut map = HashMap::new();
    for word in text.split_whitespace() {
        *map.entry(word).or_insert(0) += 1;
    }
    map
}

fn main() {
    let text = String::from("hello world wonderful world");
    let counts = word_count(&text);
    for (word, count) in &counts {
        println!("{word}: {count}");
    }
}