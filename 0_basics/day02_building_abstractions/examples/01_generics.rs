fn largest<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut max = list[0];
    for &item in list.iter() {
        if item > max {
            max = item;
        }
    }
    max
}

fn main() {
    let nums = vec![1, 5, 3, 9];
    let chars = vec!['a', 'z', 'm'];

    println!("Максимум: {}", largest(&nums));
    println!("Максимум: {}", largest(&chars));
}