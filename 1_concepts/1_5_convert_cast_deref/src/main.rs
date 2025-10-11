//! # Step 1.5: Conversions, casting and dereferencing
//! 
//! Этот модуль демонстрирует ключевые концепции конвертации типов и разыменования в Rust:
//! 
//! ## Value-to-value conversion (Конвертация значение-в-значение)
//! - `From<T>` и `Into<T>` - небезопасная конвертация, может паниковать
//! - `TryFrom<T>` и `TryInto<T>` - безопасная конвертация с обработкой ошибок
//! - Эти трейты потребляют владение исходного значения
//! 
//! ## Reference-to-reference conversion (Конвертация ссылка-в-ссылку)
//! - `AsRef<T>` и `AsMut<T>` - дешевая конвертация без потребления владения
//! - `Borrow<T>` и `BorrowMut<T>` - семантически эквивалентная конвертация
//! - Разница: AsRef для "содержит", Borrow для "эквивалентно"
//! 
//! ## Dereferencing (Разыменование)
//! - `Deref<T>` и `DerefMut<T>` - для создания умных указателей
//! - Позволяют использовать кастомные типы как обычные ссылки
//! - Должны использоваться только для умных указателей, не для newtype паттерна
//! 
//! ## Casting (Приведение типов)
//! - Ключевое слово `as` - только для ограниченного набора преобразований
//! - Не рекомендуется использовать, когда доступны другие способы конвертации

use std::ops::{Deref, DerefMut};
use std::convert::From;
use std::borrow::Borrow;
use std::fmt;
use std::error::Error;

/// Ошибка валидации email адреса
#[derive(Debug, Clone, PartialEq)]
pub struct EmailValidationError {
    message: String,
}

impl EmailValidationError {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for EmailValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Email validation error: {}", self.message)
    }
}

impl Error for EmailValidationError {}

/// Тип для хранения валидного email адреса
/// 
/// Этот тип гарантирует, что содержащаяся строка является валидным email адресом.
/// Валидация происходит при создании экземпляра через конструкторы.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmailString {
    inner: String,
}

impl EmailString {
    /// Создает новый EmailString, если переданная строка является валидным email
    /// 
    /// # Аргументы
    /// * `email` - строка для проверки и сохранения
    /// 
    /// # Возвращает
    /// * `Result<Self, EmailValidationError>` - успешный результат или ошибка валидации
    /// 
    /// # Примеры
    /// ```
    /// use step_1_5::EmailString;
    /// 
    /// let valid_email = EmailString::new("user@example.com").unwrap();
    /// let invalid_email = EmailString::new("not-an-email"); // Err
    /// ```
    pub fn new(email: &str) -> Result<Self, EmailValidationError> {
        if Self::is_valid_email(email) {
            Ok(Self {
                inner: email.to_string(),
            })
        } else {
            Err(EmailValidationError::new("Invalid email format"))
        }
    }

    /// Простая валидация email адреса
    /// 
    /// Проверяет базовые требования к формату email:
    /// - содержит символ '@'
    /// - содержит хотя бы один символ до '@'
    /// - содержит хотя бы один символ после '@'
    /// - содержит точку в доменной части
    fn is_valid_email(email: &str) -> bool {
        if email.is_empty() {
            return false;
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }

        let (local_part, domain_part) = (parts[0], parts[1]);
        
        // Локальная часть не должна быть пустой
        if local_part.is_empty() {
            return false;
        }

        // Доменная часть должна содержать точку и не быть пустой
        if domain_part.is_empty() || !domain_part.contains('.') {
            return false;
        }

        true
    }

    /// Возвращает email как строку
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

// ===== РЕАЛИЗАЦИЯ ТРЕЙТОВ ДЛЯ КОНВЕРСИИ =====

/// From<&str> - позволяет создавать EmailString из строкового литерала
/// Это небезопасная конвертация, которая может паниковать при невалидном email
impl From<&str> for EmailString {
    fn from(s: &str) -> Self {
        Self::new(s).expect("Invalid email provided to From<&str>")
    }
}

/// From<String> - позволяет создавать EmailString из String
impl From<String> for EmailString {
    fn from(s: String) -> Self {
        Self::new(&s).expect("Invalid email provided to From<String>")
    }
}

// TryFrom не реализуем, так как есть конфликт с blanket implementation
// Вместо этого используем метод new() для безопасной конвертации

/// AsRef<str> - позволяет получать &str из EmailString
/// Это дешевая операция, которая не потребляет владение
impl AsRef<str> for EmailString {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

/// Borrow<str> - семантически эквивалентно str
/// EmailString и str семантически эквивалентны для Hash, Eq, Ord
impl Borrow<str> for EmailString {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

/// Display - позволяет печатать EmailString
impl fmt::Display for EmailString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

/// Умный указатель Random<T>
/// 
/// Хранит 3 значения типа T и при каждом обращении случайно выбирает одно из них.
/// Это демонстрирует использование трейтов Deref и DerefMut для создания умного указателя.
#[derive(Debug)]
pub struct Random<T> {
    values: [T; 3],
    current_index: usize,
}

impl<T> Random<T> {
    /// Создает новый Random указатель с тремя значениями
    /// 
    /// # Аргументы
    /// * `val1`, `val2`, `val3` - три значения для хранения
    /// 
    /// # Примеры
    /// ```
    /// use step_1_5::Random;
    /// 
    /// let random = Random::new(1, 2, 3);
    /// println!("{}", *random); // Случайно выведет 1, 2 или 3
    /// ```
    pub fn new(val1: T, val2: T, val3: T) -> Self {
        let mut instance = Self {
            values: [val1, val2, val3],
            current_index: 0,
        };
        instance.select_random();
        instance
    }

    /// Возвращает ссылку на текущее выбранное значение
    fn get_current(&self) -> &T {
        &self.values[self.current_index]
    }

    /// Возвращает мутабельную ссылку на текущее выбранное значение
    fn get_current_mut(&mut self) -> &mut T {
        &mut self.values[self.current_index]
    }

    /// Выбирает случайное значение для следующего обращения
    fn select_random(&mut self) {
        // Используем текущее время как источник случайности
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        self.current_index = (now as usize) % 3;
    }

    /// Принудительно выбирает новое случайное значение
    /// Полезно для демонстрации случайности
    pub fn shuffle(&mut self) {
        self.select_random();
    }
}

/// Deref - позволяет использовать Random<T> как обычную ссылку на T
/// При каждом обращении случайно выбирается одно из трех значений
impl<T> Deref for Random<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Для immutable deref мы не можем изменить current_index
        // Поэтому просто возвращаем текущее значение
        self.get_current()
    }
}

/// DerefMut - позволяет мутировать Random<T> как обычную ссылку на T
/// При каждом обращении случайно выбирается одно из трех значений
impl<T> DerefMut for Random<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.select_random();
        self.get_current_mut()
    }
}

fn main() {
    println!("=== Демонстрация EmailString ===");
    println!("EmailString демонстрирует концепции value-to-value и reference-to-reference конвертации");
    
    // Создание EmailString с валидацией
    // Демонстрация безопасного создания с обработкой ошибок
    match EmailString::new("user@example.com") {
        Ok(email) => println!("✅ Валидный email: {}", email),
        Err(e) => println!("❌ Ошибка: {}", e),
    }

    match EmailString::new("invalid-email") {
        Ok(email) => println!("✅ Валидный email: {}", email),
        Err(e) => println!("❌ Ошибка: {}", e),
    }

    // Использование From трейта - небезопасная конвертация
    // From<&str> автоматически реализуется, Into<EmailString> тоже работает
    let email_from_str: EmailString = "admin@rust-lang.org".into();
    println!("📧 Email из From<&str>: {}", email_from_str);

    // Использование безопасного метода new()
    // Это предпочтительный способ для безопасной конвертации
    match EmailString::new("test@domain.com") {
        Ok(email) => println!("📧 Email из new(): {}", email),
        Err(e) => println!("❌ Ошибка new(): {}", e),
    }

    // Использование AsRef и Borrow - reference-to-reference конвертация
    // AsRef<str> - дешевая операция, не потребляет владение
    // Borrow<str> - семантически эквивалентно str (для Hash, Eq, Ord)
    let email = EmailString::new("hello@world.com").unwrap();
    let as_ref_str: &str = email.as_ref();
    let borrow_str: &str = email.borrow();
    println!("🔗 AsRef: {}, Borrow: {}", as_ref_str, borrow_str);

    println!("\n=== Демонстрация Random<T> ===");
    println!("Random<T> демонстрирует концепции Deref и DerefMut для создания умного указателя");
    
    // Создание Random указателя
    // Random<T> - это умный указатель, который хранит 3 значения и случайно выбирает одно
    let mut random_numbers = Random::new(10, 20, 30);
    println!("🎲 Random числа (с перемешиванием между обращениями):");
    for i in 1..=5 {
        // Использование Deref - *random_numbers автоматически вызывает deref()
        println!("  Обращение {}: {}", i, *random_numbers);
        random_numbers.shuffle(); // Принудительно выбираем новое значение
    }

    // Демонстрация с мутацией
    let mut random_strings = Random::new(
        String::from("Привет"),
        String::from("Мир"),
        String::from("Rust")
    );
    
    println!("\n🎲 Random строки:");
    for i in 1..=3 {
        // Deref позволяет использовать Random<String> как &String
        println!("  Обращение {}: {}", i, *random_strings);
        random_strings.shuffle();
    }

    // Мутация через DerefMut
    // DerefMut позволяет мутировать значение через умный указатель
    println!("\n🎲 Мутация Random строк:");
    *random_strings = String::from("Изменено!");
    println!("  После мутации: {}", *random_strings);

    // Демонстрация с EmailString
    // Random<T> работает с любым типом T, включая наши кастомные типы
    let email1 = EmailString::new("first@example.com").unwrap();
    let email2 = EmailString::new("second@example.com").unwrap();
    let email3 = EmailString::new("third@example.com").unwrap();
    
    let mut random_emails = Random::new(email1, email2, email3);
    println!("\n🎲 Random email адреса:");
    for i in 1..=3 {
        // Deref автоматически разыменовывает Random<EmailString> в &EmailString
        println!("  Обращение {}: {}", i, *random_emails);
        random_emails.shuffle();
    }
}
