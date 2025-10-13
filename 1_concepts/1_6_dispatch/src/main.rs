use std::borrow::Cow;
use std::collections::HashMap;

// ============================================================================
// БАЗОВЫЕ СТРУКТУРЫ И ТРЕЙТЫ
// ============================================================================

/// Трейт для хранения данных с ключами типа K и значениями типа V
/// 
/// Этот трейт определяет базовый интерфейс для различных реализаций хранилища.
/// Используется как для статической, так и для динамической диспетчеризации.
trait Storage<K, V> {
    /// Устанавливает значение по ключу
    fn set(&mut self, key: K, val: V);
    
    /// Получает ссылку на значение по ключу
    fn get(&self, key: &K) -> Option<&V>;
    
    /// Удаляет значение по ключу и возвращает его
    fn remove(&mut self, key: &K) -> Option<V>;
}

/// Структура пользователя
/// 
/// Использует Cow<'static, str> для эффективного хранения строк,
/// что позволяет избежать лишних аллокаций при работе с литералами.
#[derive(Debug, Clone, PartialEq)]
struct User {
    id: u64,
    email: Cow<'static, str>,
    activated: bool,
}

impl User {
    /// Создает нового пользователя
    fn new(id: u64, email: impl Into<Cow<'static, str>>, activated: bool) -> Self {
        Self {
            id,
            email: email.into(),
            activated,
        }
    }
}

// ============================================================================
// КОНКРЕТНЫЕ РЕАЛИЗАЦИИ STORAGE
// ============================================================================

/// Простая реализация Storage на основе HashMap
/// 
/// Эта реализация используется для демонстрации работы с обеими
/// типами диспетчеризации.
#[derive(Debug, Default)]
pub struct HashMapStorage<K, V> 
where 
    K: std::hash::Hash + Eq + Clone,
{
    data: HashMap<K, V>,
}

impl<K, V> HashMapStorage<K, V> 
where 
    K: std::hash::Hash + Eq + Clone,
{
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<K, V> Storage<K, V> for HashMapStorage<K, V>
where 
    K: std::hash::Hash + Eq + Clone,
{
    fn set(&mut self, key: K, val: V) {
        self.data.insert(key, val);
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.data.remove(key)
    }
}

// ============================================================================
// USER REPOSITORY С ДИНАМИЧЕСКОЙ ДИСПЕТЧЕРИЗАЦИЕЙ
// ============================================================================

/// Репозиторий пользователей с динамической диспетчеризацией
/// 
/// Использует trait object (Box<dyn Storage<u64, User>>) для хранения
/// конкретной реализации Storage. Тип стирается во время компиляции,
/// и вызовы методов разрешаются во время выполнения через vtable.
/// 
/// Преимущества:
/// - Позволяет менять реализацию Storage во время выполнения
/// - Подходит для гетерогенных коллекций
/// - Более гибкий в использовании
/// 
/// Недостатки:
/// - Накладные расходы на виртуальные вызовы (vtable lookup)
/// - Небольшая потеря производительности
/// - Ограничения object safety
pub struct DynamicUserRepository {
    storage: Box<dyn Storage<u64, User>>,
}

impl DynamicUserRepository {
    /// Создает новый репозиторий с указанной реализацией Storage
    pub fn new<S>(storage: S) -> Self 
    where 
        S: Storage<u64, User> + 'static,
    {
        Self {
            storage: Box::new(storage),
        }
    }

    /// Добавляет пользователя в хранилище
    pub fn add_user(&mut self, user: User) {
        self.storage.set(user.id, user);
    }

    /// Получает пользователя по ID
    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.storage.get(&id)
    }

    /// Обновляет пользователя
    pub fn update_user(&mut self, user: User) -> Option<User> {
        self.storage.remove(&user.id).map(|_| {
            self.storage.set(user.id, user.clone());
            user
        })
    }

    /// Удаляет пользователя по ID
    pub fn remove_user(&mut self, id: u64) -> Option<User> {
        self.storage.remove(&id)
    }

    /// Получает все ID пользователей (для демонстрации)
    pub fn get_all_user_ids(&self) -> Vec<u64> {
        // В реальной реализации здесь был бы итератор по ключам
        // Для простоты возвращаем пустой вектор
        vec![]
    }
}

// ============================================================================
// USER REPOSITORY СО СТАТИЧЕСКОЙ ДИСПЕТЧЕРИЗАЦИЕЙ
// ============================================================================

/// Репозиторий пользователей со статической диспетчеризацией
/// 
/// Использует generic параметр S для хранения конкретной реализации Storage.
/// Тип известен во время компиляции, и компилятор генерирует отдельный код
/// для каждого используемого типа (monomorphization).
/// 
/// Преимущества:
/// - Нет накладных расходов на виртуальные вызовы
/// - Лучшая производительность
/// - Возможность инлайнинга
/// - Нет ограничений object safety
/// 
/// Недостатки:
/// - Увеличение размера бинарного файла (code bloat)
/// - Менее гибкий (тип должен быть известен во время компиляции)
/// - Не подходит для гетерогенных коллекций
pub struct StaticUserRepository<S> 
where 
    S: Storage<u64, User>,
{
    storage: S,
}

impl<S> StaticUserRepository<S> 
where 
    S: Storage<u64, User>,
{
    /// Создает новый репозиторий с указанной реализацией Storage
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Добавляет пользователя в хранилище
    pub fn add_user(&mut self, user: User) {
        self.storage.set(user.id, user);
    }

    /// Получает пользователя по ID
    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.storage.get(&id)
    }

    /// Обновляет пользователя
    pub fn update_user(&mut self, user: User) -> Option<User> {
        self.storage.remove(&user.id).map(|_| {
            self.storage.set(user.id, user.clone());
            user
        })
    }

    /// Удаляет пользователя по ID
    pub fn remove_user(&mut self, id: u64) -> Option<User> {
        self.storage.remove(&id)
    }

    /// Получает все ID пользователей (для демонстрации)
    pub fn get_all_user_ids(&self) -> Vec<u64> {
        // В реальной реализации здесь был бы итератор по ключам
        // Для простоты возвращаем пустой вектор
        vec![]
    }
}

// ============================================================================
// ДОПОЛНИТЕЛЬНАЯ РЕАЛИЗАЦИЯ STORAGE ДЛЯ ДЕМОНСТРАЦИИ
// ============================================================================

/// Реализация Storage на основе Vec для демонстрации
/// 
/// Эта реализация менее эффективна, но показывает, как можно
/// легко переключаться между различными реализациями Storage.
#[derive(Debug, Default)]
pub struct VecStorage<V> {
    data: Vec<(u64, V)>,
}

impl<V> VecStorage<V> {
    fn new() -> Self {
        Self { data: Vec::new() }
    }
}

impl<V> Storage<u64, V> for VecStorage<V> 
where 
    V: Clone,
{
    fn set(&mut self, key: u64, val: V) {
        // Ищем существующую запись
        if let Some((_, existing_val)) = self.data.iter_mut().find(|(k, _)| *k == key) {
            *existing_val = val;
        } else {
            // Добавляем новую запись
            self.data.push((key, val));
        }
    }

    fn get(&self, key: &u64) -> Option<&V> {
        self.data.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    fn remove(&mut self, key: &u64) -> Option<V> {
        if let Some(pos) = self.data.iter().position(|(k, _)| k == key) {
            Some(self.data.remove(pos).1)
        } else {
            None
        }
    }
}

// ============================================================================
// ОСНОВНАЯ ФУНКЦИЯ И ДЕМОНСТРАЦИЯ
// ============================================================================

fn main() {
    println!("=== Демонстрация Static и Dynamic Dispatch в Rust ===\n");

    // Создаем тестовых пользователей
    let user1 = User::new(1, "alice@example.com", true);
    let user2 = User::new(2, "bob@example.com", false);
    let user3 = User::new(3, "charlie@example.com", true);

    println!("Созданы пользователи:");
    println!("  {:?}", user1);
    println!("  {:?}", user2);
    println!("  {:?}", user3);
    println!();

    // ========================================================================
    // ДЕМОНСТРАЦИЯ СТАТИЧЕСКОЙ ДИСПЕТЧЕРИЗАЦИИ
    // ========================================================================
    
    println!("=== СТАТИЧЕСКАЯ ДИСПЕТЧЕРИЗАЦИЯ ===");
    println!("Тип Storage известен во время компиляции");
    println!("Компилятор генерирует отдельный код для каждого типа\n");

    // Создаем репозиторий с HashMapStorage
    let mut static_repo_hashmap = StaticUserRepository::new(HashMapStorage::new());
    
    // Добавляем пользователей
    static_repo_hashmap.add_user(user1.clone());
    static_repo_hashmap.add_user(user2.clone());
    
    // Получаем пользователя
    if let Some(user) = static_repo_hashmap.get_user(1) {
        println!("Найден пользователь через HashMapStorage: {:?}", user);
    }
    
    // Создаем репозиторий с VecStorage
    let mut static_repo_vec = StaticUserRepository::new(VecStorage::new());
    
    // Добавляем пользователей
    static_repo_vec.add_user(user2.clone());
    static_repo_vec.add_user(user3.clone());
    
    // Получаем пользователя
    if let Some(user) = static_repo_vec.get_user(2) {
        println!("Найден пользователь через VecStorage: {:?}", user);
    }
    
    println!();

    // ========================================================================
    // ДЕМОНСТРАЦИЯ ДИНАМИЧЕСКОЙ ДИСПЕТЧЕРИЗАЦИИ
    // ========================================================================
    
    println!("=== ДИНАМИЧЕСКАЯ ДИСПЕТЧЕРИЗАЦИЯ ===");
    println!("Тип Storage стирается во время компиляции");
    println!("Вызовы методов разрешаются во время выполнения через vtable\n");

    // Создаем репозиторий с HashMapStorage через trait object
    let mut dynamic_repo = DynamicUserRepository::new(HashMapStorage::new());
    
    // Добавляем пользователей
    dynamic_repo.add_user(user1.clone());
    dynamic_repo.add_user(user2.clone());
    dynamic_repo.add_user(user3.clone());
    
    // Получаем пользователя
    if let Some(user) = dynamic_repo.get_user(1) {
        println!("Найден пользователь через DynamicUserRepository: {:?}", user);
    }
    
    // Обновляем пользователя
    let updated_user = User::new(2, "bob.updated@example.com", true);
    if let Some(old_user) = dynamic_repo.update_user(updated_user.clone()) {
        println!("Обновлен пользователь: {:?} -> {:?}", old_user, updated_user);
    }
    
    // Удаляем пользователя
    if let Some(removed_user) = dynamic_repo.remove_user(3) {
        println!("Удален пользователь: {:?}", removed_user);
    }
    
    println!();

    // ========================================================================
    // ДЕМОНСТРАЦИЯ ГИБКОСТИ ДИНАМИЧЕСКОЙ ДИСПЕТЧЕРИЗАЦИИ
    // ========================================================================
    
    println!("=== ГИБКОСТЬ ДИНАМИЧЕСКОЙ ДИСПЕТЧЕРИЗАЦИИ ===");
    println!("Можно легко переключаться между реализациями Storage\n");

    // Функция, которая принимает любой Storage через trait object
    fn demonstrate_storage(storage: Box<dyn Storage<u64, User>>) {
        let mut repo = DynamicUserRepository { storage };
        
        let test_user = User::new(999, "test@example.com", true);
        repo.add_user(test_user.clone());
        
        if let Some(user) = repo.get_user(999) {
            println!("Пользователь успешно сохранен и получен: {:?}", user);
        }
    }
    
    // Используем HashMapStorage
    println!("Используем HashMapStorage:");
    demonstrate_storage(Box::new(HashMapStorage::new()));
    
    // Используем VecStorage
    println!("Используем VecStorage:");
    demonstrate_storage(Box::new(VecStorage::new()));
    
    println!();

    // ========================================================================
    // ОБЪЯСНЕНИЕ РАЗЛИЧИЙ
    // ========================================================================
    
    println!("=== КЛЮЧЕВЫЕ РАЗЛИЧИЯ ===");
    println!();
    println!("СТАТИЧЕСКАЯ ДИСПЕТЧЕРИЗАЦИЯ:");
    println!("  ✓ Нет накладных расходов на виртуальные вызовы");
    println!("  ✓ Лучшая производительность");
    println!("  ✓ Возможность инлайнинга");
    println!("  ✓ Нет ограничений object safety");
    println!("  ✗ Увеличение размера бинарного файла (code bloat)");
    println!("  ✗ Менее гибкий (тип должен быть известен во время компиляции)");
    println!("  ✗ Не подходит для гетерогенных коллекций");
    println!();
    println!("ДИНАМИЧЕСКАЯ ДИСПЕТЧЕРИЗАЦИЯ:");
    println!("  ✓ Позволяет менять реализацию Storage во время выполнения");
    println!("  ✓ Подходит для гетерогенных коллекций");
    println!("  ✓ Более гибкий в использовании");
    println!("  ✗ Накладные расходы на виртуальные вызовы (vtable lookup)");
    println!("  ✗ Небольшая потеря производительности");
    println!("  ✗ Ограничения object safety");
    println!();
    println!("ВЫВОД:");
    println!("  - Используйте статическую диспетчеризацию, когда это возможно");
    println!("  - Используйте динамическую диспетчеризацию, когда нужна гибкость");
    println!("  - Рассмотрите enum-based подход для закрытых наборов типов");
    
    // ========================================================================
    // ДЕМОНСТРАЦИЯ ENUM-BASED ПОДХОДА (ОПТИМИЗАЦИЯ ДИНАМИЧЕСКОЙ ДИСПЕТЧЕРИЗАЦИИ)
    // ========================================================================
    
    println!("\n=== ENUM-BASED ПОДХОД (ОПТИМИЗАЦИЯ) ===");
    println!("Для закрытых наборов типов можно заменить динамическую");
    println!("диспетчеризацию на статическую через enum\n");
    
    demonstrate_enum_based_approach();
}

// ============================================================================
// ENUM-BASED ПОДХОД ДЛЯ ОПТИМИЗАЦИИ
// ============================================================================

/// Enum-based реализация Storage для демонстрации оптимизации
/// 
/// Этот подход используется когда у нас есть закрытый набор типов Storage,
/// которые мы хотим использовать. Вместо динамической диспетчеризации
/// мы используем enum с match-выражениями, что дает нам:
/// 
/// Преимущества:
/// - Статическая диспетчеризация (нет vtable lookup)
/// - Лучшая производительность чем dynamic dispatch
/// - Меньший размер бинарного файла чем полная monomorphization
/// - Гибкость в выборе конкретного типа во время выполнения
/// 
/// Недостатки:
/// - Нужно знать все возможные типы заранее
/// - Boilerplate код для каждого нового типа
/// - Не подходит для открытых наборов типов
#[derive(Debug)]
pub enum StorageEnum<V> {
    HashMap(HashMapStorage<u64, V>),
    Vec(VecStorage<V>),
}

impl<V> StorageEnum<V> 
where 
    V: Clone,
{
    pub fn new_hashmap() -> Self {
        Self::HashMap(HashMapStorage::new())
    }
    
    pub fn new_vec() -> Self {
        Self::Vec(VecStorage::new())
    }
}

impl<V> Storage<u64, V> for StorageEnum<V>
where 
    V: Clone,
{
    fn set(&mut self, key: u64, val: V) {
        match self {
            StorageEnum::HashMap(storage) => storage.set(key, val),
            StorageEnum::Vec(storage) => storage.set(key, val),
        }
    }

    fn get(&self, key: &u64) -> Option<&V> {
        match self {
            StorageEnum::HashMap(storage) => storage.get(key),
            StorageEnum::Vec(storage) => storage.get(key),
        }
    }

    fn remove(&mut self, key: &u64) -> Option<V> {
        match self {
            StorageEnum::HashMap(storage) => storage.remove(key),
            StorageEnum::Vec(storage) => storage.remove(key),
        }
    }
}

/// Репозиторий с enum-based диспетчеризацией
/// 
/// Этот репозиторий демонстрирует как можно получить преимущества
/// статической диспетчеризации при сохранении гибкости выбора
/// конкретной реализации во время выполнения.
pub struct EnumUserRepository {
    storage: StorageEnum<User>,
}

impl EnumUserRepository {
    pub fn new(storage: StorageEnum<User>) -> Self {
        Self { storage }
    }

    pub fn add_user(&mut self, user: User) {
        self.storage.set(user.id, user);
    }

    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.storage.get(&id)
    }

    pub fn update_user(&mut self, user: User) -> Option<User> {
        self.storage.remove(&user.id).map(|_| {
            self.storage.set(user.id, user.clone());
            user
        })
    }

    pub fn remove_user(&mut self, id: u64) -> Option<User> {
        self.storage.remove(&id)
    }
}

fn demonstrate_enum_based_approach() {
    // Создаем тестовых пользователей
    let user1 = User::new(100, "enum_user1@example.com", true);
    let user2 = User::new(101, "enum_user2@example.com", false);
    
    println!("Созданы пользователи для enum-based подхода:");
    println!("  {:?}", user1);
    println!("  {:?}", user2);
    println!();
    
    // Демонстрируем работу с HashMap через enum
    println!("Используем HashMap через enum:");
    let mut enum_repo_hashmap = EnumUserRepository::new(StorageEnum::new_hashmap());
    enum_repo_hashmap.add_user(user1.clone());
    enum_repo_hashmap.add_user(user2.clone());
    
    if let Some(user) = enum_repo_hashmap.get_user(100) {
        println!("  Найден пользователь: {:?}", user);
    }
    
    // Демонстрируем работу с Vec через enum
    println!("Используем Vec через enum:");
    let mut enum_repo_vec = EnumUserRepository::new(StorageEnum::new_vec());
    enum_repo_vec.add_user(user1.clone());
    enum_repo_vec.add_user(user2.clone());
    
    if let Some(user) = enum_repo_vec.get_user(101) {
        println!("  Найден пользователь: {:?}", user);
    }
    
    println!();
    println!("ПРЕИМУЩЕСТВА ENUM-BASED ПОДХОДА:");
    println!("  ✓ Статическая диспетчеризация (нет vtable lookup)");
    println!("  ✓ Лучшая производительность чем dynamic dispatch");
    println!("  ✓ Меньший размер бинарного файла чем полная monomorphization");
    println!("  ✓ Гибкость в выборе конкретного типа во время выполнения");
    println!("  ✗ Нужно знать все возможные типы заранее");
    println!("  ✗ Boilerplate код для каждого нового типа");
    println!("  ✗ Не подходит для открытых наборов типов");
    println!();
    println!("ПРИМЕНЕНИЕ:");
    println!("  - Когда у вас есть закрытый набор типов Storage");
    println!("  - Когда нужна производительность лучше чем dynamic dispatch");
    println!("  - Когда не нужна полная гибкость dynamic dispatch");
    println!("  - Когда размер бинарного файла критичен");
}

// ============================================================================
// ТЕСТЫ
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_dispatch_with_hashmap() {
        let mut repo = StaticUserRepository::new(HashMapStorage::new());
        
        let user = User::new(1, "test@example.com", true);
        repo.add_user(user.clone());
        
        assert_eq!(repo.get_user(1), Some(&user));
        assert_eq!(repo.get_user(2), None);
        
        let removed = repo.remove_user(1);
        assert_eq!(removed, Some(user));
        assert_eq!(repo.get_user(1), None);
    }

    #[test]
    fn test_static_dispatch_with_vec() {
        let mut repo = StaticUserRepository::new(VecStorage::new());
        
        let user = User::new(1, "test@example.com", true);
        repo.add_user(user.clone());
        
        assert_eq!(repo.get_user(1), Some(&user));
        assert_eq!(repo.get_user(2), None);
        
        let removed = repo.remove_user(1);
        assert_eq!(removed, Some(user));
        assert_eq!(repo.get_user(1), None);
    }

    #[test]
    fn test_dynamic_dispatch() {
        let mut repo = DynamicUserRepository::new(HashMapStorage::new());
        
        let user = User::new(1, "test@example.com", true);
        repo.add_user(user.clone());
        
        assert_eq!(repo.get_user(1), Some(&user));
        assert_eq!(repo.get_user(2), None);
        
        let removed = repo.remove_user(1);
        assert_eq!(removed, Some(user));
        assert_eq!(repo.get_user(1), None);
    }

    #[test]
    fn test_user_update() {
        let mut repo = DynamicUserRepository::new(HashMapStorage::new());
        
        let user1 = User::new(1, "old@example.com", false);
        let user2 = User::new(1, "new@example.com", true);
        
        repo.add_user(user1.clone());
        assert_eq!(repo.get_user(1), Some(&user1));
        
        let updated = repo.update_user(user2.clone());
        assert_eq!(updated, Some(user2.clone()));
        assert_eq!(repo.get_user(1), Some(&user2));
    }

    #[test]
    fn test_different_storage_implementations() {
        // Тестируем, что обе реализации Storage работают одинаково
        let mut hashmap_repo = DynamicUserRepository::new(HashMapStorage::new());
        let mut vec_repo = DynamicUserRepository::new(VecStorage::new());
        
        let user = User::new(1, "test@example.com", true);
        
        // Добавляем в оба репозитория
        hashmap_repo.add_user(user.clone());
        vec_repo.add_user(user.clone());
        
        // Проверяем, что оба работают одинаково
        assert_eq!(hashmap_repo.get_user(1), Some(&user));
        assert_eq!(vec_repo.get_user(1), Some(&user));
        
        // Проверяем удаление
        assert_eq!(hashmap_repo.remove_user(1), Some(user.clone()));
        assert_eq!(vec_repo.remove_user(1), Some(user));
    }

    #[test]
    fn test_enum_based_dispatch() {
        let mut repo = EnumUserRepository::new(StorageEnum::new_hashmap());
        
        let user = User::new(1, "test@example.com", true);
        repo.add_user(user.clone());
        
        assert_eq!(repo.get_user(1), Some(&user));
        assert_eq!(repo.get_user(2), None);
        
        let removed = repo.remove_user(1);
        assert_eq!(removed, Some(user));
        assert_eq!(repo.get_user(1), None);
    }

    #[test]
    fn test_enum_based_dispatch_with_vec() {
        let mut repo = EnumUserRepository::new(StorageEnum::new_vec());
        
        let user = User::new(1, "test@example.com", true);
        repo.add_user(user.clone());
        
        assert_eq!(repo.get_user(1), Some(&user));
        assert_eq!(repo.get_user(2), None);
        
        let removed = repo.remove_user(1);
        assert_eq!(removed, Some(user));
        assert_eq!(repo.get_user(1), None);
    }
}
