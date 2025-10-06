//! Box<T>: выделение в куче и рекурсивные типы

use std::mem::size_of;

fn main() {
    // 1) Простое размещение значения в куче
    let x = 42u64;
    let y = Box::new(x); // значение в heap, указатель на stack

    println!("x (stack): {x}");
    println!("y (heap value via Box): {y}");
    println!("size_of::<u64>() = {} bytes", size_of::<u64>());
    println!("size_of::<Box<u64>>() = {} bytes (указатель)", size_of::<Box<u64>>());

    // 2) Рекурсивный тип через Box
    #[derive(Debug)]
    enum List {
        Cons(i32, Box<List>),
        Nil,
    }

    use List::{Cons, Nil};

    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
    println!("Рекурсивный список: {list:?}");

    // 3) Демонстрация владения: при выходе из области видимости память heap освобождается автоматически (Drop у Box)
}