fn main() {
    let s1 = String::from("hi");
    print_str(&s1);
    println!("{}", s1);
}

fn print_str(s: &String) {
    println!("{}", s);
}