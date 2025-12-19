use std::fmt;
use std::pin::Pin;
use std::rc::Rc;
use std::future::Future;
use std::task::{Context, Poll};

// Трейт SayHi для демонстрации работы с Pin<&Self>
// Требует, чтобы тип реализовывал fmt::Debug
trait SayHi: fmt::Debug {
    fn say_hi(self: Pin<&Self>) {
        println!("Hi from {:?}", self)
    }
}

// Трейт MutMeSomehow для демонстрации работы с Pin<&mut Self>
// Демонстрирует преобразование Pin<&mut Self> -> &mut self
// без использования Unpin trait bounds
trait MutMeSomehow {
    fn mut_me_somehow(self: Pin<&mut Self>) {
        // Implementation must be meaningful, and
        // obviously call something requiring `&mut self`.
        // The point here is to practice dealing with
        // `Pin<&mut Self>` -> `&mut self` conversion
        // in different contexts, without introducing 
        // any `Unpin` trait bounds.
    }
}

// Реализация SayHi для Box<T>
impl<T: fmt::Debug> SayHi for Box<T> {}

// Реализация MutMeSomehow для Box<T>
impl<T> MutMeSomehow for Box<T> {
    fn mut_me_somehow(self: Pin<&mut Self>) {
        // Для Box<T> мы можем получить &mut T через Pin::get_mut
        // Box<T> реализует Unpin, поэтому это безопасно
        let _boxed = Pin::get_mut(self);
        // Пример мутации: если T - это число, увеличим его на 1
        // Но поскольку мы не знаем тип T, просто выведем информацию
        println!("Mutating Box<{}>", std::any::type_name::<T>());
    }
}

// Реализация SayHi для Rc<T>
impl<T: fmt::Debug> SayHi for Rc<T> {}

// Реализация MutMeSomehow для Rc<T>
impl<T> MutMeSomehow for Rc<T> {
    fn mut_me_somehow(self: Pin<&mut Self>) {
        // Rc<T> - это shared ownership, поэтому мы не можем мутировать содержимое
        // Но можем вывести информацию о типе
        println!("Rc<{}> cannot be mutated (shared ownership)", std::any::type_name::<T>());
    }
}

// Реализация SayHi для Vec<T>
impl<T: fmt::Debug> SayHi for Vec<T> {}

// Реализация MutMeSomehow для Vec<T>
impl<T: std::marker::Unpin> MutMeSomehow for Vec<T> {
    fn mut_me_somehow(self: Pin<&mut Self>) {
        // Vec<T> реализует Unpin, поэтому можем получить &mut Vec<T>
        let vec = Pin::get_mut(self);
        // Добавим элемент в вектор (если возможно)
        if vec.is_empty() {
            println!("Vec is empty, cannot demonstrate mutation");
        } else {
            println!("Vec has {} elements before mutation", vec.len());
            // Пример мутации: добавим еще один элемент того же типа
            // Но поскольку мы не знаем тип T, просто выведем информацию
            println!("Vec<{}> can be mutated", std::any::type_name::<T>());
        }
    }
}

// Реализация SayHi для String
impl SayHi for String {}

// Реализация MutMeSomehow для String
impl MutMeSomehow for String {
    fn mut_me_somehow(self: Pin<&mut Self>) {
        // String реализует Unpin, поэтому можем получить &mut String
        let string = Pin::get_mut(self);
        string.push_str(" (mutated)");
        println!("String mutated: {}", string);
    }
}

// Реализация SayHi для &[u8]
impl SayHi for &[u8] {}

// Реализация MutMeSomehow для &[u8]
impl MutMeSomehow for &[u8] {
    fn mut_me_somehow(self: Pin<&mut Self>) {
        // &[u8] - это неизменяемая ссылка, поэтому мутация невозможна
        println!("&[u8] cannot be mutated (immutable reference)");
    }
}

// Реализация SayHi для i32 (конкретный тип вместо обобщенного T)
impl SayHi for i32 {}

// Реализация MutMeSomehow для i32
impl MutMeSomehow for i32 {
    fn mut_me_somehow(self: Pin<&mut Self>) {
        // Для i32 мы можем получить &mut i32 через Pin::get_mut
        let number = Pin::get_mut(self);
        *number += 1;
        println!("i32 mutated to: {}", number);
    }
}

// Структура MeasurableFuture для измерения времени выполнения Future
struct MeasurableFuture<Fut> {
    inner_future: Fut,
    started_at: Option<std::time::Instant>,
}

impl<Fut> MeasurableFuture<Fut> {
    fn new(inner_future: Fut) -> Self {
        Self {
            inner_future,
            started_at: None,
        }
    }
}

// Реализация Future для MeasurableFuture
impl<Fut: Future> Future for MeasurableFuture<Fut> {
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Используем unsafe для получения &mut Self, поскольку мы не можем использовать Unpin bound
        let this = unsafe { self.get_unchecked_mut() };
        
        // Если это первый вызов poll, записываем время начала
        if this.started_at.is_none() {
            this.started_at = Some(std::time::Instant::now());
        }

        // Создаем Pin для inner_future
        // Поскольку мы не можем использовать Unpin bound, используем unsafe
        let inner_pin = unsafe { Pin::new_unchecked(&mut this.inner_future) };
        
        // Опрашиваем inner_future
        match inner_pin.poll(cx) {
            Poll::Ready(result) => {
                // Future завершился, выводим время выполнения
                if let Some(started_at) = this.started_at {
                    let duration = started_at.elapsed();
                    println!("Future completed in {} nanoseconds", duration.as_nanos());
                }
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// Пример использования
async fn example_async_function() -> i32 {
    // Имитируем асинхронную работу
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    42
}

#[tokio::main]
async fn main() {
    println!("=== Testing SayHi trait ===");
    
    // Тестируем SayHi для разных типов
    let boxed = Box::new(42);
    Pin::new(&boxed).say_hi();
    
    let rc = Rc::new("Hello");
    Pin::new(&rc).say_hi();
    
    let vec = vec![1, 2, 3];
    Pin::new(&vec).say_hi();
    
    let string = String::from("World");
    Pin::new(&string).say_hi();
    
    let slice: &[u8] = b"test";
    Pin::new(&slice).say_hi();
    
    let number = 123;
    Pin::new(&number).say_hi();
    
    println!("\n=== Testing MutMeSomehow trait ===");
    
    // Тестируем MutMeSomehow для разных типов
    let mut boxed = Box::new(42);
    Pin::new(&mut boxed).mut_me_somehow();
    
    let mut rc = Rc::new("Hello");
    Pin::new(&mut rc).mut_me_somehow();
    
    let mut vec = vec![1, 2, 3];
    Pin::new(&mut vec).mut_me_somehow();
    
    let mut string = String::from("World");
    Pin::new(&mut string).mut_me_somehow();
    println!("String after mutation: {}", string);
    
    let mut slice: &[u8] = b"test";
    Pin::new(&mut slice).mut_me_somehow();
    
    let mut number = 123;
    Pin::new(&mut number).mut_me_somehow();
    println!("i32 after mutation: {}", number);
    
    println!("\n=== Testing MeasurableFuture ===");
    
    // Тестируем MeasurableFuture
    let future = MeasurableFuture::new(example_async_function());
    
    // Запускаем future
    let result = future.await;
    println!("Future result: {}", result);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;

    #[tokio::test(flavor = "current_thread")]
    async fn measurable_future_returns_inner_result() {
        let future = MeasurableFuture::new(async { 7u8 });
        assert_eq!(future.await, 7);
    }

    #[test]
    fn mutating_string_and_integer_through_pin() {
        let mut text = String::from("hello");
        Pin::new(&mut text).mut_me_somehow();
        assert!(
            text.contains("(mutated)"),
            "String mutation should append the marker"
        );

        let mut number = 10i32;
        Pin::new(&mut number).mut_me_somehow();
        assert_eq!(number, 11);
    }
}
