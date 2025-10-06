//! Arc<Mutex<T>>: разделяемые изменяемые данные между потоками

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0i64));
    let mut handles = Vec::new();

    for _ in 0..8 {
        let c = Arc::clone(&counter);
        let h = thread::spawn(move || {
            for _ in 0..100_000 {
                // блокировка мьютекса — безопасный доступ из нескольких потоков
                *c.lock().expect("mutex poisoned") += 1;
            }
        });
        handles.push(h);
    }

    for h in handles {
        h.join().expect("thread panicked");
    }

    let result = *counter.lock().expect("mutex poisoned");
    println!("Итоговое значение счётчика: {result} (ожидаем 800_000)");
}