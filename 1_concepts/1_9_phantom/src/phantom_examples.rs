//! Дополнительные примеры использования Phantom Types
//! 
//! Этот файл содержит расширенные примеры демонстрации концепций phantom types
//! и их практического применения в Rust.

use std::marker::PhantomData;

/// Пример 1: Типобезопасные единицы измерения
/// 
/// Этот пример показывает, как использовать phantom types для создания
/// типобезопасных единиц измерения, которые предотвращают ошибки
/// смешивания разных единиц (например, метров и километров).

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Length<Unit> {
    value: f64,
    _unit: PhantomData<Unit>,
}

// Маркеры для разных единиц измерения
#[derive(Debug, Clone, Copy)]
pub struct Meter;
#[derive(Debug, Clone, Copy)]
pub struct Kilometer;
#[derive(Debug, Clone, Copy)]
pub struct Centimeter;

impl<Unit> Length<Unit> {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            _unit: PhantomData,
        }
    }
    
    pub fn value(&self) -> f64 {
        self.value
    }
}

// Реализация для метров
impl Length<Meter> {
    pub fn meters(value: f64) -> Self {
        Self::new(value)
    }
    
    pub fn to_kilometers(self) -> Length<Kilometer> {
        Length::new(self.value / 1000.0)
    }
    
    pub fn to_centimeters(self) -> Length<Centimeter> {
        Length::new(self.value * 100.0)
    }
}

// Реализация для километров
impl Length<Kilometer> {
    pub fn kilometers(value: f64) -> Self {
        Self::new(value)
    }
    
    pub fn to_meters(self) -> Length<Meter> {
        Length::new(self.value * 1000.0)
    }
}

// Реализация для сантиметров
impl Length<Centimeter> {
    pub fn centimeters(value: f64) -> Self {
        Self::new(value)
    }
    
    pub fn to_meters(self) -> Length<Meter> {
        Length::new(self.value / 100.0)
    }
}

// Арифметические операции только для одинаковых единиц
impl<Unit> std::ops::Add for Length<Unit> {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Self::new(self.value + other.value)
    }
}

impl<Unit> std::ops::Sub for Length<Unit> {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        Self::new(self.value - other.value)
    }
}

/// Пример 2: Типобезопасные состояния
/// 
/// Этот пример показывает, как использовать phantom types для создания
/// типобезопасных состояний, которые предотвращают выполнение операций
/// в неподходящих состояниях.

#[derive(Debug, Clone)]
pub struct Connection<State> {
    id: u32,
    _state: PhantomData<State>,
}

// Маркеры состояний
#[derive(Debug, Clone, Copy)]
pub struct Disconnected;
#[derive(Debug, Clone, Copy)]
pub struct Connected;
#[derive(Debug, Clone, Copy)]
pub struct Authenticated;

impl Connection<Disconnected> {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            _state: PhantomData,
        }
    }
    
    pub fn connect(self) -> Connection<Connected> {
        println!("Подключение к серверу с ID: {}", self.id);
        Connection {
            id: self.id,
            _state: PhantomData,
        }
    }
}

impl Connection<Connected> {
    pub fn authenticate(self, token: &str) -> Result<Connection<Authenticated>, std::string::String> {
        if token.is_empty() {
            Err("Неверный токен".to_string())
        } else {
            println!("Аутентификация успешна для соединения ID: {}", self.id);
            Ok(Connection {
                id: self.id,
                _state: PhantomData,
            })
        }
    }
    
    pub fn disconnect(self) -> Connection<Disconnected> {
        println!("Отключение от сервера ID: {}", self.id);
        Connection {
            id: self.id,
            _state: PhantomData,
        }
    }
}

impl Connection<Authenticated> {
    pub fn send_message(&self, message: &str) {
        println!("Отправка сообщения '{}' через соединение ID: {}", message, self.id);
    }
    
    pub fn disconnect(self) -> Connection<Disconnected> {
        println!("Отключение аутентифицированного соединения ID: {}", self.id);
        Connection {
            id: self.id,
            _state: PhantomData,
        }
    }
}

/// Пример 3: Типобезопасные указатели
/// 
/// Этот пример показывает, как использовать phantom types для создания
/// типобезопасных указателей, которые предотвращают смешивание разных типов.

#[derive(Debug, Clone)]
pub struct Pointer<T> {
    address: usize,
    _phantom: PhantomData<T>,
}

impl<T> Pointer<T> {
    pub fn new(address: usize) -> Self {
        Self {
            address,
            _phantom: PhantomData,
        }
    }
    
    pub fn address(&self) -> usize {
        self.address
    }
    
    pub fn cast<U>(self) -> Pointer<U> {
        Pointer::new(self.address)
    }
}

// Маркеры для разных типов данных
#[derive(Debug, Clone, Copy)]
pub struct Integer;
#[derive(Debug, Clone, Copy)]
pub struct Float;
#[derive(Debug, Clone, Copy)]
pub struct StringType;

impl Pointer<Integer> {
    pub fn read_int(&self) -> i32 {
        println!("Чтение целого числа по адресу: 0x{:x}", self.address);
        // В реальном коде здесь было бы небезопасное чтение из памяти
        42
    }
}

impl Pointer<Float> {
    pub fn read_float(&self) -> f64 {
        println!("Чтение числа с плавающей точкой по адресу: 0x{:x}", self.address);
        // В реальном коде здесь было бы небезопасное чтение из памяти
        3.14
    }
}

/// Пример 4: Типобезопасные контейнеры
/// 
/// Этот пример показывает, как использовать phantom types для создания
/// типобезопасных контейнеров с разными стратегиями хранения.

#[derive(Debug, Clone)]
pub struct Container<T, Strategy> {
    data: Vec<u8>,
    _phantom: PhantomData<(T, Strategy)>,
}

// Маркеры стратегий
#[derive(Debug, Clone, Copy)]
pub struct Stack;
#[derive(Debug, Clone, Copy)]
pub struct Heap;
#[derive(Debug, Clone, Copy)]
pub struct Pool;

impl<T, Strategy> Container<T, Strategy> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            _phantom: PhantomData,
        }
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<T> Container<T, Stack> {
    pub fn push(&mut self, _item: T) {
        println!("Добавление элемента в стек-контейнер");
        // В реальном коде здесь было бы сериализация и добавление в стек
        self.data.push(0);
    }
    
    pub fn pop(&mut self) -> Option<T> {
        println!("Извлечение элемента из стек-контейнера");
        // В реальном коде здесь было бы десериализация из стека
        self.data.pop().map(|_| unsafe { std::mem::zeroed() })
    }
}

impl<T> Container<T, Heap> {
    pub fn insert(&mut self, _item: T) {
        println!("Вставка элемента в кучу-контейнер");
        // В реальном коде здесь было бы сериализация и добавление в кучу
        self.data.push(1);
    }
    
    pub fn remove(&mut self, index: usize) -> Option<T> {
        println!("Удаление элемента из кучи-контейнера по индексу: {}", index);
        // В реальном коде здесь было бы десериализация из кучи
        if index < self.data.len() {
            self.data.remove(index);
            Some(unsafe { std::mem::zeroed() })
        } else {
            None
        }
    }
}

impl<T> Container<T, Pool> {
    pub fn allocate(&mut self, _item: T) -> usize {
        println!("Выделение элемента в пуле-контейнере");
        // В реальном коде здесь было бы выделение из пула
        let index = self.data.len();
        self.data.push(2);
        index
    }
    
    pub fn deallocate(&mut self, index: usize) -> Option<T> {
        println!("Освобождение элемента из пула-контейнера по индексу: {}", index);
        // В реальном коде здесь было бы освобождение в пул
        if index < self.data.len() {
            self.data[index] = 0;
            Some(unsafe { std::mem::zeroed() })
        } else {
            None
        }
    }
}

/// Демонстрация всех примеров
pub fn demonstrate_phantom_types() {
    println!("\n=== Расширенные примеры Phantom Types ===\n");
    
    // Пример 1: Единицы измерения
    println!("1. Типобезопасные единицы измерения:");
    let distance1 = Length::meters(1000.0);
    let distance2 = Length::kilometers(2.0);
    let distance3 = Length::centimeters(50000.0);
    
    println!("   Расстояние 1: {} метров", distance1.value());
    println!("   Расстояние 2: {} километров", distance2.value());
    println!("   Расстояние 3: {} сантиметров", distance3.value());
    
    // Преобразование единиц
    let distance1_km = distance1.to_kilometers();
    let distance2_m = distance2.to_meters();
    let distance3_m = distance3.to_meters();
    
    println!("   Расстояние 1 в км: {}", distance1_km.value());
    println!("   Расстояние 2 в м: {}", distance2_m.value());
    println!("   Расстояние 3 в м: {}", distance3_m.value());
    
    // Арифметические операции (только для одинаковых единиц)
    let distance1_copy = Length::meters(1000.0);
    let sum = distance1_copy + distance1_copy;
    println!("   Сумма двух одинаковых расстояний: {} метров", sum.value());
    
    // Пример 2: Состояния соединения
    println!("\n2. Типобезопасные состояния:");
    let conn = Connection::new(123);
    let connected = conn.connect();
    
    match connected.authenticate("valid_token") {
        Ok(auth_conn) => {
            auth_conn.send_message("Привет, сервер!");
            let _disconnected = auth_conn.disconnect();
        }
        Err(e) => println!("   Ошибка аутентификации: {}", e),
    }
    
    // Пример 3: Типобезопасные указатели
    println!("\n3. Типобезопасные указатели:");
    let int_ptr = Pointer::<Integer>::new(0x1000);
    let float_ptr = Pointer::<Float>::new(0x2000);
    
    let int_value = int_ptr.read_int();
    let float_value = float_ptr.read_float();
    
    println!("   Целое число: {}", int_value);
    println!("   Число с плавающей точкой: {}", float_value);
    
    // Преобразование типов указателей
    let casted_ptr = int_ptr.cast::<Float>();
    println!("   Преобразованный указатель: 0x{:x}", casted_ptr.address());
    
    // Пример 4: Типобезопасные контейнеры
    println!("\n4. Типобезопасные контейнеры:");
    
    // Стек-контейнер
    let mut stack_container: Container<i32, Stack> = Container::new();
    stack_container.push(42);
    stack_container.push(24);
    let _popped = stack_container.pop();
    
    // Куча-контейнер
    let mut heap_container: Container<StringType, Heap> = Container::new();
    heap_container.insert(StringType);
    heap_container.insert(StringType);
    let _removed = heap_container.remove(0);
    
    // Пул-контейнер
    let mut pool_container: Container<f64, Pool> = Container::new();
    let index1 = pool_container.allocate(3.14);
    let _index2 = pool_container.allocate(2.71);
    let _deallocated = pool_container.deallocate(index1);
    
    println!("\n=== Преимущества Phantom Types ===");
    println!("1. Безопасность типов на этапе компиляции");
    println!("2. Предотвращение ошибок программиста");
    println!("3. Нулевые накладные расходы во время выполнения");
    println!("4. Выразительность кода и самодокументирование");
    println!("5. Возможность создания типобезопасных API");
}