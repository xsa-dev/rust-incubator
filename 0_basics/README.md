Step 0: Become familiar with Rust basics
========================================

__Estimated time__: 3 days

Read through [the Rust Book][Rust Book], [Rust FAQ], and become familiar with basic [Rust] concepts, syntax, the memory model, and the type and module systems.

Polish your familiarity by completing [Rust By Example] and [Rustlings][rustlings].

Read through [the Cargo Book][Cargo Book] and become familiar with [Cargo] and its workspaces.

After completing these steps, you should be able to answer (and understand why) the following questions:
- What memory model does [Rust] have? Is it single-threaded or multiple-threaded? Is it synchronous or asynchronous?

Rust использует модель владения (ownership model) и строгое управление памятью без GC.
Он поддерживает многопоточность (multi-threaded) из коробки, с безопасностью на уровне типов (Send, Sync).
Исполнение по умолчанию синхронное, но можно использовать async/await и Future для асинхронного кода.

💡 Итог: Rust — многопоточный, синхронный по умолчанию, без автоматического GC.

- What runtime does [Rust] have? Does it use a GC (garbage collector)?

Rust имеет минимальный runtime — почти “zero-cost abstractions”.
Нет собственного планировщика потоков, сборщика мусора или виртуальной машины.
Память освобождается детерминированно при выходе из области видимости (Drop).

💡 Итог: Rust не имеет GC; ресурсы освобождаются через RAII.

- What does static typing mean? What is a benefit of using it?

Статическая типизация — это когда типы проверяются во время компиляции, а не во время выполнения.
Это предотвращает типовые ошибки до запуска программы и позволяет компилятору делать оптимизации.

💡 Преимущество: надёжность и производительность.

- What are generics and parametric polymorphism? Which problems do they solve?

  Generics — это механизм, позволяющий писать обобщённый код для разных типов.
  Parametric polymorphism означает, что функция работает одинаково для любых типов, не зная их конкретно.

💡 Зачем: уменьшает дублирование и повышает гибкость.

```rust
fn max<T: Ord>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
```

- What are traits? How are they used? How do they compare to interfaces? What are auto traits and blanket impls? What is a marker trait?

Traits — это контракты поведения: набор методов, которые тип должен реализовать.
Аналог интерфейсов в других языках, но мощнее (могут содержать реализацию по умолчанию).
•	Auto traits: автоматически реализуются компилятором, например Send, Sync.
•	Blanket impls: реализации трейта для всех типов, удовлетворяющих условию:

```rust
impl<T: Display> ToString for T { ... }
```

- What are static and dynamic dispatch? Which should you use, and when?

•	Static dispatch: выбирает реализацию во время компиляции (fn foo<T: Trait>()).
✅ Быстрее, без накладных расходов.
•	Dynamic dispatch: выбирает реализацию во время выполнения через dyn Trait.
✅ Гибче, но с затратами на указатель и таблицу виртуальных методов (vtable).

💡 Используй static dispatch почти всегда; dynamic — когда нужен полиморфизм во время выполнения.

- What is a crate and what is a module in [Rust]? How do they differ? How are they used?

•	Crate — единица компиляции (пакет или библиотека).
•	Module (mod) — способ организации кода внутри crate.

💡 Разница:
•	crate → “проект”
•	module → “файл или пространство имён внутри проекта”

- What are move semantics? What are borrowing rules? What is the benefit of using them?

•	Move semantics: при присваивании владение данными передаётся, а исходная переменная становится недействительной.
•	Borrowing: временное использование данных без владения (&T или &mut T).

💡 Польза:
Безопасное управление памятью без GC и data race.

- What is immutability? What is the benefit of using it?

По умолчанию переменные неизменяемы (let).
Это уменьшает количество ошибок, делает код предсказуемым и потокобезопасным.

Можно сделать изменяемой через let mut.

- What is cloning? What is copying? How do they compare?

•	Copy: простое побитовое копирование (для простых типов: i32, bool).
•	Clone: глубокое копирование данных (реализуется вручную через Clone трейт).

💡 Clone требует ресурсов, Copy — нет.

- What is RAII? How is it implemented in [Rust]? What is the benefit of using it?

RAII (Resource Acquisition Is Initialization): ресурсы освобождаются автоматически, когда объект выходит из области видимости.
Реализовано через Drop трейт.

💡 Польза: освобождение памяти, файлов, сокетов, без утечек.

- What is an iterator? What is a collection? How do they differ? How are they used?

•	Collection: хранит элементы (Vec, HashMap, String).
•	Iterator: выдаёт элементы по одному, лениво.

```rust
let v = vec![1, 2, 3];
for x in v.iter() {
    println!("{x}");
}
```
💡 Итераторы экономят память и позволяют функциональный стиль (map, filter).

- What are macros? Which problems do they solve? What is the difference between declarative and procedural macros?

Macros — метапрограммирование в Rust.
Позволяют генерировать код во время компиляции.
•	Declarative (macro_rules!) — шаблонное сопоставление.
•	Procedural (#[proc_macro]) — анализ и генерация AST, мощнее, но сложнее.

💡 Используются для сокращения шаблонного кода, автогенерации boilerplate.

- How is code tested in [Rust]? Where should you put tests and why?

•	Модульные тесты — внутри файла, под модулем #[cfg(test)] mod tests.
•	Интеграционные тесты — в отдельной папке tests/.

💡 Почему так: модульные тесты проверяют детали реализации, интеграционные — поведение публичного API.

- Why does [Rust] have `&str` and `String` types? How do they differ? When should you use them?

•	String — владеющий тип (heap).
•	&str — срез строки, ссылка на существующие данные.

💡 Используй &str для входных аргументов, String — когда тебе нужно владение или модификация.

- What are lifetimes? Which problems do they solve? Which benefits do they give?

Lifetimes описывают, как долго ссылки остаются действительными.
Они предотвращают висячие ссылки и утечки.

💡 Польза: безопасность без сборщика мусора и runtime-проверок.

- Is [Rust] an OOP language? Is it possible to use SOLID/GRASP? Does it have inheritance?

Rust не является классическим ООП-языком, но поддерживает:
•	инкапсуляцию через pub/mod,
•	полиморфизм через traits,
•	композицию вместо наследования.

💡 SOLID/GRASP применимы, но через композицию и трейты.
Наследования нет, но его заменяют трейты и делегирование.

_Additional_ articles, which may help to understand the above topic better:
- [George He: Thinking in Rust: Ownership, Access, and Memory Safety][19]
- [Chris Morgan: Rust ownership, the hard way][1]
- [Adolfo Ochagavía: You are holding it wrong][12]
- [Vikram Fugro: Beyond Pointers: How Rust outshines C++ with its Borrow Checker][15]
- [Sabrina Jewson: Why the “Null” Lifetime Does Not Exist][16]
- [HashRust: A guide to closures in Rust][13]
- [Ludwig Stecher: Rusts Module System Explained][2]
- [Tristan Hume: Models of Generics and Metaprogramming: Go, Rust, Swift, D and More][3]
- [Jeff Anderson: Generics Demystified Part 1][4]
- [Jeff Anderson: Generics Demystified Part 2][5]
- [Bradford Hovinen: Demystifying trait generics in Rust][14]
- [Brandon Smith: Three Kinds of Polymorphism in Rust][6]
- [Jeremy Steward: C++ & Rust: Generics and Specialization][7]
- [Lukasz Uszko: Safe and Secure Coding in Rust: A Comparative Analysis of Rust and C/C++][18]
- [cooscoos: &stress about &Strings][8]
- [Jimmy Hartzell: RAII: Compile-Time Memory Management in C++ and Rust][9]
- [Georgios Antonopoulos: Rust vs Common C++ Bugs][10]
- [Yurii Shymon: True Observer Pattern with Unsubscribe mechanism using Rust][11]
- [Clayton Ramsey: I built a garbage collector for a language that doesn't need one][17]




[Cargo]: https://github.com/rust-lang/cargo
[Cargo Book]: https://doc.rust-lang.org/cargo
[Rust]: https://www.rust-lang.org
[Rust Book]: https://doc.rust-lang.org/book
[Rust By Example]: https://doc.rust-lang.org/rust-by-example
[Rust FAQ]: https://prev.rust-lang.org/faq.html
[rustlings]: https://rustlings.cool

[1]: https://chrismorgan.info/blog/rust-ownership-the-hard-way
[2]: https://aloso.github.io/2021/03/28/module-system.html
[3]: https://thume.ca/2019/07/14/a-tour-of-metaprogramming-models-for-generics
[4]: https://web.archive.org/web/20220525213911/http://jeffa.io/rust_guide_generics_demystified_part_1
[5]: https://web.archive.org/web/20220328114028/https://jeffa.io/rust_guide_generics_demystified_part_2
[6]: https://www.brandons.me/blog/polymorphism-in-rust
[7]: https://www.tangramvision.com/blog/c-rust-generics-and-specialization#substitution-ordering--failures
[8]: https://cooscoos.github.io/blog/stress-about-strings
[9]: https://www.thecodedmessage.com/posts/raii
[10]: https://geo-ant.github.io/blog/2022/common-cpp-errors-vs-rust
[11]: https://web.archive.org/web/20230319015854/https://ybnesm.github.io/blah/articles/true-observer-pattern-rust
[12]: https://ochagavia.nl/blog/you-are-holding-it-wrong
[13]: https://hashrust.com/blog/a-guide-to-closures-in-rust
[14]: https://gruebelinchen.wordpress.com/2023/06/06/demystifying-trait-generics-in-rust
[15]: https://dev.to/vikram2784/beyond-pointers-how-rust-outshines-c-with-its-borrow-checker-1mad
[16]: https://sabrinajewson.org/blog/null-lifetime
[17]: https://claytonwramsey.github.io/2023/08/14/dumpster.html
[18]: https://luk6xff.github.io/other/safe_secure_rust_book/intro/index.html
[19]: https://cocoindex.io/blogs/rust-ownership-access
