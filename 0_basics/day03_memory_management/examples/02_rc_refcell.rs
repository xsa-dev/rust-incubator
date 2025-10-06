//! Rc<T> + RefCell<T>: разделяемое владение и внутренняя изменяемость (однопоточно)

use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Rc дает разделяемое владение, RefCell — изменяемость в runtime
    let shared_counter: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));

    let a = Rc::clone(&shared_counter);
    let b = Rc::clone(&shared_counter);

    println!("strong_count после двух клонов: {}", Rc::strong_count(&shared_counter));

    // Изменяем через один клон...
    *a.borrow_mut() += 5;
    // ...и через второй
    *b.borrow_mut() += 10;

    // Чтение
    let current = *shared_counter.borrow();
    println!("Текущее значение: {current} (ожидаем 15)");

    // borrow checker в runtime:
    // Если раскомментировать, будет panic из-за двойного borrow_mut:
    // let _m1 = shared_counter.borrow_mut();
    // let _m2 = shared_counter.borrow_mut(); // <- panic
}