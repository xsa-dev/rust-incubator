use std::marker::PhantomData;
use rand::Rng;

// Подключаем дополнительные примеры
mod phantom_examples;
use phantom_examples::demonstrate_phantom_types;

/// Структура Fact<T> использует phantom type для хранения информации о типе T
/// без фактического хранения значения этого типа в runtime.
/// 
/// PhantomData<T> позволяет компилятору отслеживать тип T на уровне типов,
/// но не занимает места в памяти во время выполнения программы.
#[derive(Debug, Clone)]
pub struct Fact<T> {
    /// PhantomData используется для "привязки" типа T к структуре Fact
    /// без фактического хранения значения типа T
    _phantom: PhantomData<T>,
    /// Хранилище случайных фактов для разных типов
    facts: Vec<&'static str>,
}

impl<T> Fact<T> {
    /// Создает новый экземпляр Fact<T> с пустым списком фактов
    /// 
    /// # Примеры
    /// 
    /// ```rust
    /// let fact: Fact<Vec<i32>> = Fact::new();
    /// ```
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
            facts: Vec::new(),
        }
    }
    
    /// Возвращает случайный факт о типе T
    /// 
    /// # Примеры
    /// 
    /// ```rust
    /// let fact: Fact<Vec<i32>> = Fact::new();
    /// println!("Факт о Vec: {}", fact.fact());
    /// ```
    pub fn fact(&self) -> &'static str {
        if self.facts.is_empty() {
            "Нет доступных фактов об этом типе"
        } else {
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..self.facts.len());
            self.facts[index]
        }
    }
}

/// Трейт для типов, о которых можно получить факты
pub trait FactProvider {
    /// Возвращает список фактов о данном типе
    fn get_facts() -> Vec<&'static str>;
}

/// Реализация FactProvider для Vec<T>
impl<T> FactProvider for Vec<T> {
    fn get_facts() -> Vec<&'static str> {
        vec![
            "Vec - это динамический массив, выделяемый в куче",
            "Vec может перераспределять память при росте",
            "Vec имеет O(1) амортизированное время добавления в конец",
            "Vec может содержать элементы разных типов только если они упакованы в enum",
            "Vec автоматически освобождает память при выходе из области видимости",
            "Vec может быть создан с предварительно выделенной емкостью",
            "Vec поддерживает итерацию по элементам",
            "Vec может быть преобразован в слайс &[T]",
        ]
    }
}

/// Реализация FactProvider для String
impl FactProvider for String {
    fn get_facts() -> Vec<&'static str> {
        vec![
            "String - это владеющая строка в UTF-8",
            "String может изменять свой размер во время выполнения",
            "String хранит данные в куче",
            "String автоматически освобождает память при выходе из области видимости",
            "String может быть создана из &str",
            "String поддерживает индексацию по байтам, но не по символам",
            "String может содержать нулевые байты",
            "String реализует Deref<Target = str>",
        ]
    }
}

/// Реализация FactProvider для i32
impl FactProvider for i32 {
    fn get_facts() -> Vec<&'static str> {
        vec![
            "i32 - это 32-битное знаковое целое число",
            "i32 имеет диапазон от -2,147,483,648 до 2,147,483,647",
            "i32 занимает 4 байта в памяти",
            "i32 может переполняться при арифметических операциях",
            "i32 имеет методы для работы с битами",
            "i32 может быть преобразован в другие числовые типы",
            "i32 реализует множество трейтов для арифметических операций",
            "i32 является Copy типом",
        ]
    }
}

/// Реализация FactProvider для bool
impl FactProvider for bool {
    fn get_facts() -> Vec<&'static str> {
        vec![
            "bool - это булев тип с двумя значениями: true и false",
            "bool занимает 1 байт в памяти",
            "bool может быть преобразован в целое число (true = 1, false = 0)",
            "bool реализует трейты для логических операций",
            "bool является Copy типом",
            "bool может быть создан из различных типов через as",
            "bool используется в условных выражениях",
            "bool может быть преобразован в String",
        ]
    }
}

/// Реализация FactProvider для Option<T>
impl<T> FactProvider for Option<T> {
    fn get_facts() -> Vec<&'static str> {
        vec![
            "Option<T> - это enum с двумя вариантами: Some(T) и None",
            "Option<T> используется для представления отсутствующих значений",
            "Option<T> является безопасной альтернативой null указателям",
            "Option<T> имеет множество полезных методов для работы с значениями",
            "Option<T> может быть преобразован в Result<T, E>",
            "Option<T> поддерживает итерацию",
            "Option<T> может быть развернут с помощью match или if let",
            "Option<T> является Copy типом если T является Copy",
        ]
    }
}

/// Реализация FactProvider для Result<T, E>
impl<T, E> FactProvider for Result<T, E> {
    fn get_facts() -> Vec<&'static str> {
        vec![
            "Result<T, E> - это enum с двумя вариантами: Ok(T) и Err(E)",
            "Result<T, E> используется для обработки ошибок",
            "Result<T, E> является основой системы обработки ошибок в Rust",
            "Result<T, E> имеет множество методов для работы с ошибками",
            "Result<T, E> может быть преобразован в Option<T>",
            "Result<T, E> поддерживает оператор ? для распространения ошибок",
            "Result<T, E> может быть развернут с помощью match или if let",
            "Result<T, E> является Copy типом если T и E являются Copy",
        ]
    }
}

/// Расширенная реализация Fact<T> с автоматическим получением фактов
impl<T: FactProvider> Fact<T> {
    /// Создает новый экземпляр Fact<T> с фактами, полученными от FactProvider
    pub fn with_facts() -> Self {
        Self {
            _phantom: PhantomData,
            facts: T::get_facts(),
        }
    }
}

/// Реализация Default для Fact<T>
impl<T> Default for Fact<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    println!("=== Демонстрация Phantom Types в Rust ===\n");
    
    // Пример 1: Fact<Vec<T>> с автоматическими фактами
    println!("1. Факты о Vec<T>:");
    let vec_fact: Fact<Vec<i32>> = Fact::with_facts();
    for i in 1..=3 {
        println!("   Факт {}: {}", i, vec_fact.fact());
    }
    
    // Пример 2: Fact<String> с автоматическими фактами
    println!("\n2. Факты о String:");
    let string_fact: Fact<String> = Fact::with_facts();
    for i in 1..=3 {
        println!("   Факт {}: {}", i, string_fact.fact());
    }
    
    // Пример 3: Fact<i32> с автоматическими фактами
    println!("\n3. Факты о i32:");
    let int_fact: Fact<i32> = Fact::with_facts();
    for i in 1..=3 {
        println!("   Факт {}: {}", i, int_fact.fact());
    }
    
    // Пример 4: Fact<bool> с автоматическими фактами
    println!("\n4. Факты о bool:");
    let bool_fact: Fact<bool> = Fact::with_facts();
    for i in 1..=3 {
        println!("   Факт {}: {}", i, bool_fact.fact());
    }
    
    // Пример 5: Fact<Option<T>> с автоматическими фактами
    println!("\n5. Факты о Option<T>:");
    let option_fact: Fact<Option<i32>> = Fact::with_facts();
    for i in 1..=3 {
        println!("   Факт {}: {}", i, option_fact.fact());
    }
    
    // Пример 6: Fact<Result<T, E>> с автоматическими фактами
    println!("\n6. Факты о Result<T, E>:");
    let result_fact: Fact<Result<i32, String>> = Fact::with_facts();
    for i in 1..=3 {
        println!("   Факт {}: {}", i, result_fact.fact());
    }
    
    // Пример 7: Демонстрация того, что разные типы дают разные факты
    println!("\n7. Сравнение фактов для разных типов:");
    let fact1: Fact<Vec<i32>> = Fact::with_facts();
    let fact2: Fact<String> = Fact::with_facts();
    let fact3: Fact<i32> = Fact::with_facts();
    
    println!("   Vec<i32>: {}", fact1.fact());
    println!("   String: {}", fact2.fact());
    println!("   i32: {}", fact3.fact());
    
    // Пример 8: Демонстрация PhantomData
    println!("\n8. Демонстрация PhantomData:");
    let _phantom_fact: Fact<Vec<String>> = Fact::new();
    println!("   Размер Fact<Vec<String>>: {} байт", std::mem::size_of::<Fact<Vec<String>>>());
    println!("   Размер PhantomData<Vec<String>>: {} байт", std::mem::size_of::<PhantomData<Vec<String>>>());
    println!("   Размер Vec<String>: {} байт", std::mem::size_of::<Vec<String>>());
    
    // Пример 9: Демонстрация того, что PhantomData не влияет на auto traits
    println!("\n9. Демонстрация auto traits:");
    let send_fact: Fact<Vec<i32>> = Fact::new();
    let _ = std::thread::spawn(move || {
        println!("   Fact<Vec<i32>> можно отправить в другой поток (Send)");
        println!("   Факт: {}", send_fact.fact());
    }).join();
    
    println!("\n=== Объяснение Phantom Types ===");
    println!("Phantom Types (фантомные типы) - это типы, которые используются");
    println!("только на уровне типов для обеспечения безопасности типов,");
    println!("но не занимают места в памяти во время выполнения.");
    println!("\nОсновные преимущества:");
    println!("1. Безопасность типов на этапе компиляции");
    println!("2. Нулевые накладные расходы во время выполнения");
    println!("3. Возможность выражать инварианты на уровне типов");
    println!("4. Предотвращение ошибок программиста");
    
    // Демонстрация дополнительных примеров
    demonstrate_phantom_types();
}
