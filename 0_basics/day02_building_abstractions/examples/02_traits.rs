trait Summary {
    fn summarize(&self) -> String;
}

struct Article {
    title: String,
    author: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("'{}' by {}", self.title, self.author)
    }
}

fn main() {
    let a = Article {
        title: "Learning Rust".to_string(),
        author: "Alice".to_string(),
    };
    println!("{}", a.summarize());
}