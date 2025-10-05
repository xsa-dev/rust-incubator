trait Animal {
    fn speak(&self) -> String;
    fn eat(&self) -> String;
}

struct Dog;
struct Cat;

impl Animal for Dog {
    fn speak(&self) -> String { "Woof!".to_string() }
    fn eat(&self) -> String { "Dog eats kibble 🦴".to_string() }
}

impl Animal for Cat {
    fn speak(&self) -> String { "Meow!".to_string() }
    fn eat(&self) -> String { "Cat eats fish 🐟".to_string() }
}

// ---------- 1️⃣ Static Dispatch ----------
fn static_zoo<T: Animal>(animals: &[T]) {
    for a in animals {
        println!("{} | {}", a.speak(), a.eat());
    }
}

// ---------- 2️⃣ Dynamic Dispatch ----------
fn dynamic_zoo(animals: &[Box<dyn Animal>]) {
    for a in animals {
        println!("{} | {}", a.speak(), a.eat());
    }
}

fn main() {
    // Static dispatch — работает с одним типом
    let dogs = vec![Dog, Dog];
    static_zoo(&dogs);

    println!("----------------------------");

    // Dynamic dispatch — работает с разными типами
    let animals: Vec<Box<dyn Animal>> = vec![Box::new(Dog), Box::new(Cat)];
    dynamic_zoo(&animals);
}