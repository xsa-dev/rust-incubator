use std::borrow::Cow;

// ============================================================================
// БАЗОВЫЕ СТРУКТУРЫ И ТРЕЙТЫ
// ============================================================================

/// Трейт для команд, которые могут быть обработаны
/// 
/// Этот трейт определяет базовый интерфейс для всех команд в системе.
/// Команды представляют собой действия, которые могут быть выполнены
/// над доменными объектами.
pub trait Command {
    /// Возвращает тип команды для логирования и отладки
    fn command_type(&self) -> &'static str;
}

/// Команда для создания нового пользователя
/// 
/// Эта команда содержит все необходимые данные для создания
/// нового пользователя в системе.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateUser {
    pub email: Cow<'static, str>,
    pub activated: bool,
}

impl CreateUser {
    /// Создает новую команду создания пользователя
    pub fn new(email: impl Into<Cow<'static, str>>, activated: bool) -> Self {
        Self {
            email: email.into(),
            activated,
        }
    }
}

impl Command for CreateUser {
    fn command_type(&self) -> &'static str {
        "CreateUser"
    }
}

/// Структура пользователя
/// 
/// Использует Cow<'static, str> для эффективного хранения строк,
/// что позволяет избежать лишних аллокаций при работе с литералами.
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: u64,
    pub email: Cow<'static, str>,
    pub activated: bool,
}

impl User {
    /// Создает нового пользователя
    pub fn new(id: u64, email: impl Into<Cow<'static, str>>, activated: bool) -> Self {
        Self {
            id,
            email: email.into(),
            activated,
        }
    }
}

/// Ошибки, которые могут возникнуть при работе с пользователями
/// 
/// Этот enum определяет все возможные ошибки, которые могут
/// возникнуть при выполнении операций с пользователями.
#[derive(Debug, Clone, PartialEq)]
pub enum UserError {
    /// Пользователь с таким email уже существует
    UserAlreadyExists(String),
    /// Пользователь не найден
    UserNotFound(u64),
    /// Некорректный email
    InvalidEmail(String),
    /// Внутренняя ошибка системы
    InternalError(String),
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::UserAlreadyExists(email) => {
                write!(f, "Пользователь с email '{}' уже существует", email)
            }
            UserError::UserNotFound(id) => {
                write!(f, "Пользователь с ID {} не найден", id)
            }
            UserError::InvalidEmail(email) => {
                write!(f, "Некорректный email: '{}'", email)
            }
            UserError::InternalError(msg) => {
                write!(f, "Внутренняя ошибка: {}", msg)
            }
        }
    }
}

impl std::error::Error for UserError {}

// ============================================================================
// ТРЕЙТ USER REPOSITORY С ?SIZED BOUND
// ============================================================================

/// Трейт для работы с пользователями в хранилище
/// 
/// Этот трейт определяет интерфейс для работы с пользователями.
/// Трейты по умолчанию ?Sized, что позволяет использовать как обычные типы,
/// так и trait objects (dyn UserRepository).
/// 
/// ?Sized означает "может быть не Sized", что позволяет:
/// - Использовать обычные структуры (которые Sized по умолчанию)
/// - Использовать trait objects (которые не Sized)
/// - Использовать slice types (которые не Sized)
/// 
/// Без ?Sized bound все типы параметров имеют неявную границу Sized,
/// что запрещает использование trait objects.
pub trait UserRepository {
    /// Сохраняет пользователя в хранилище
    /// 
    /// # Аргументы
    /// * `user` - пользователь для сохранения
    /// 
    /// # Возвращает
    /// * `Result<(), UserError>` - результат операции
    fn save_user(&mut self, user: User) -> Result<(), UserError>;
    
    /// Находит пользователя по ID
    /// 
    /// # Аргументы
    /// * `id` - ID пользователя
    /// 
    /// # Возвращает
    /// * `Result<Option<User>, UserError>` - найденный пользователь или None
    fn find_user_by_id(&self, id: u64) -> Result<Option<User>, UserError>;
    
    /// Находит пользователя по email
    /// 
    /// # Аргументы
    /// * `email` - email пользователя
    /// 
    /// # Возвращает
    /// * `Result<Option<User>, UserError>` - найденный пользователь или None
    fn find_user_by_email(&self, email: &str) -> Result<Option<User>, UserError>;
    
    /// Удаляет пользователя по ID
    /// 
    /// # Аргументы
    /// * `id` - ID пользователя
    /// 
    /// # Возвращает
    /// * `Result<Option<User>, UserError>` - удаленный пользователь или None
    fn delete_user(&mut self, id: u64) -> Result<Option<User>, UserError>;
}

// ============================================================================
// COMMAND HANDLER ТРЕЙТ
// ============================================================================

/// Трейт для обработки команд
/// 
/// Этот трейт определяет интерфейс для обработки команд в системе.
/// Использует ?Sized bound для Context, что позволяет использовать
/// как обычные типы, так и trait objects.
/// 
/// Пример использования:
/// ```rust
/// impl CommandHandler<CreateUser> for User {
///     type Context = dyn UserRepository;
///     type Result = Result<(), UserError>;
///     
///     fn handle_command(&self, cmd: &CreateUser, ctx: &Self::Context) -> Self::Result {
///         // Обработка команды создания пользователя
///     }
/// }
/// ```
pub trait CommandHandler<C: Command> {
    /// Тип контекста для обработки команды
    /// 
    /// Может быть как обычным типом, так и trait object благодаря ?Sized bound.
    /// Это позволяет использовать dyn UserRepository в качестве контекста.
    type Context: ?Sized;
    
    /// Тип результата обработки команды
    type Result;
    
    /// Обрабатывает команду в заданном контексте
    /// 
    /// # Аргументы
    /// * `cmd` - команда для обработки
    /// * `ctx` - контекст для обработки команды
    /// 
    /// # Возвращает
    /// * `Self::Result` - результат обработки команды
    fn handle_command(&self, cmd: &C, ctx: &mut Self::Context) -> Self::Result;
}

// ============================================================================
// РЕАЛИЗАЦИЯ COMMAND HANDLER ДЛЯ USER
// ============================================================================

/// Реализация CommandHandler<CreateUser> для User
/// 
/// Эта реализация показывает, как использовать ?Sized bound для
/// работы с trait objects в качестве контекста.
/// 
/// Благодаря ?Sized bound мы можем использовать dyn UserRepository
/// в качестве типа Context, что обеспечивает гибкость в выборе
/// конкретной реализации репозитория во время выполнения.
impl CommandHandler<CreateUser> for User {
    /// Используем dyn UserRepository как контекст
    /// 
    /// ?Sized bound позволяет использовать trait objects, которые
    /// не имеют фиксированного размера во время компиляции.
    type Context = dyn UserRepository;
    
    /// Результат обработки команды
    type Result = Result<(), UserError>;
    
    /// Обрабатывает команду создания пользователя
    /// 
    /// # Аргументы
    /// * `cmd` - команда создания пользователя
    /// * `ctx` - репозиторий пользователей (может быть любая реализация)
    /// 
    /// # Возвращает
    /// * `Result<(), UserError>` - результат операции
    /// 
    /// # Логика
    /// 1. Проверяем, что пользователь с таким email не существует
    /// 2. Создаем нового пользователя с уникальным ID
    /// 3. Сохраняем пользователя в репозитории
    fn handle_command(&self, cmd: &CreateUser, ctx: &mut Self::Context) -> Self::Result {
        // Проверяем, что пользователь с таким email не существует
        if let Ok(Some(_)) = ctx.find_user_by_email(&cmd.email) {
            return Err(UserError::UserAlreadyExists(cmd.email.to_string()));
        }
        
        // Валидируем email (простая проверка)
        if !cmd.email.contains('@') {
            return Err(UserError::InvalidEmail(cmd.email.to_string()));
        }
        
        // Создаем нового пользователя
        // В реальной системе ID генерировался бы по-другому
        let new_user = User::new(
            self.id + 1, // Простая логика генерации ID
            cmd.email.clone(),
            cmd.activated,
        );
        
        // Сохраняем пользователя в репозитории
        ctx.save_user(new_user)?;
        
        Ok(())
    }
}

// ============================================================================
// MOCK РЕАЛИЗАЦИЯ USER REPOSITORY ДЛЯ ТЕСТОВ
// ============================================================================

/// Mock реализация UserRepository для тестирования
/// 
/// Эта реализация используется в тестах для имитации работы
/// с хранилищем пользователей. Хранит данные в памяти в HashMap.
/// 
/// Показывает, как обычная структура (которая Sized) может
/// использоваться в качестве Context благодаря ?Sized bound.
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct MockUserRepository {
    users: HashMap<u64, User>,
    email_to_id: HashMap<String, u64>,
    next_id: u64,
}

impl MockUserRepository {
    /// Создает новый mock репозиторий
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            email_to_id: HashMap::new(),
            next_id: 1,
        }
    }
    
    /// Добавляет пользователя в mock репозиторий (для тестов)
    pub fn add_user(&mut self, user: User) {
        let id = user.id;
        let email = user.email.to_string();
        self.users.insert(id, user);
        self.email_to_id.insert(email, id);
        self.next_id = self.next_id.max(id + 1);
    }
    
    /// Получает всех пользователей (для тестов)
    pub fn get_all_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }
}

impl UserRepository for MockUserRepository {
    fn save_user(&mut self, user: User) -> Result<(), UserError> {
        let id = user.id;
        let email = user.email.to_string();
        
        // Проверяем, что пользователь с таким ID не существует
        if self.users.contains_key(&id) {
            return Err(UserError::UserAlreadyExists(format!("ID {}", id)));
        }
        
        // Проверяем, что пользователь с таким email не существует
        if self.email_to_id.contains_key(&email) {
            return Err(UserError::UserAlreadyExists(email));
        }
        
        // Сохраняем пользователя
        self.users.insert(id, user);
        self.email_to_id.insert(email, id);
        self.next_id = self.next_id.max(id + 1);
        
        Ok(())
    }
    
    fn find_user_by_id(&self, id: u64) -> Result<Option<User>, UserError> {
        Ok(self.users.get(&id).cloned())
    }
    
    fn find_user_by_email(&self, email: &str) -> Result<Option<User>, UserError> {
        if let Some(&id) = self.email_to_id.get(email) {
            Ok(self.users.get(&id).cloned())
        } else {
            Ok(None)
        }
    }
    
    fn delete_user(&mut self, id: u64) -> Result<Option<User>, UserError> {
        if let Some(user) = self.users.remove(&id) {
            let email = user.email.to_string();
            self.email_to_id.remove(&email);
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
}

// ============================================================================
// ОСНОВНАЯ ФУНКЦИЯ И ДЕМОНСТРАЦИЯ
// ============================================================================

fn main() {
    println!("=== Демонстрация ?Sized trait bound в Rust ===\n");
    
    // Создаем тестового пользователя
    let user = User::new(1, "admin@example.com", true);
    println!("Создан пользователь: {:?}", user);
    println!();
    
    // Создаем команду создания нового пользователя
    let create_cmd = CreateUser::new("newuser@example.com", false);
    println!("Создана команда: {:?}", create_cmd);
    println!();
    
    // Создаем mock репозиторий
    let mut mock_repo = MockUserRepository::new();
    
    // Добавляем существующего пользователя в репозиторий
    mock_repo.add_user(user.clone());
    println!("Добавлен пользователь в mock репозиторий");
    println!();
    
    // Демонстрируем работу CommandHandler с ?Sized bound
    println!("=== ДЕМОНСТРАЦИЯ ?SIZED BOUND ===");
    println!("CommandHandler<CreateUser> использует dyn UserRepository как Context");
    println!("Это возможно благодаря ?Sized bound в определении трейта\n");
    
    // Обрабатываем команду создания пользователя
    match user.handle_command(&create_cmd, &mut mock_repo) {
        Ok(()) => {
            println!("✓ Команда успешно обработана!");
            
            // Проверяем, что пользователь был создан
            if let Ok(Some(created_user)) = mock_repo.find_user_by_email("newuser@example.com") {
                println!("✓ Новый пользователь создан: {:?}", created_user);
            }
        }
        Err(e) => {
            println!("✗ Ошибка при обработке команды: {}", e);
        }
    }
    
    println!();
    
    // Демонстрируем обработку ошибки (пользователь уже существует)
    println!("=== ДЕМОНСТРАЦИЯ ОБРАБОТКИ ОШИБОК ===");
    let duplicate_cmd = CreateUser::new("admin@example.com", true);
    println!("Пытаемся создать пользователя с существующим email: {:?}", duplicate_cmd);
    
    match user.handle_command(&duplicate_cmd, &mut mock_repo) {
        Ok(()) => {
            println!("✓ Команда успешно обработана!");
        }
        Err(e) => {
            println!("✗ Ошибка при обработке команды: {}", e);
        }
    }
    
    println!();
    
    // Демонстрируем обработку некорректного email
    println!("=== ДЕМОНСТРАЦИЯ ВАЛИДАЦИИ ===");
    let invalid_cmd = CreateUser::new("invalid-email", true);
    println!("Пытаемся создать пользователя с некорректным email: {:?}", invalid_cmd);
    
    match user.handle_command(&invalid_cmd, &mut mock_repo) {
        Ok(()) => {
            println!("✓ Команда успешно обработана!");
        }
        Err(e) => {
            println!("✗ Ошибка при обработке команды: {}", e);
        }
    }
    
    println!();
    
    // Показываем все пользователей в репозитории
    println!("=== ТЕКУЩЕЕ СОСТОЯНИЕ РЕПОЗИТОРИЯ ===");
    let all_users = mock_repo.get_all_users();
    println!("Всего пользователей: {}", all_users.len());
    for user in all_users {
        println!("  {:?}", user);
    }
    
    println!();
    
    // Объясняем преимущества ?Sized bound
    println!("=== ПРЕИМУЩЕСТВА ?SIZED BOUND ===");
    println!("✓ Позволяет использовать trait objects (dyn UserRepository)");
    println!("✓ Обеспечивает гибкость в выборе реализации во время выполнения");
    println!("✓ Сохраняет совместимость с обычными типами (Sized)");
    println!("✓ Улучшает API и эргономику кода");
    println!("✓ Позволяет создавать более универсальные generic функции");
    println!();
    
    println!("=== КЛЮЧЕВЫЕ ПОНЯТИЯ ===");
    println!("• Sized trait - маркерный трейт для типов с известным размером");
    println!("• ?Sized bound - снимает неявную границу Sized");
    println!("• Trait objects - типы, которые стираются во время компиляции");
    println!("• Dynamic dispatch - разрешение вызовов во время выполнения");
    println!("• Object safety - требования для создания trait objects");
    
    println!();
    
    // Демонстрируем разницу между обычными типами и trait objects
    demonstrate_sized_vs_unsized();
}

// ============================================================================
// ДОПОЛНИТЕЛЬНАЯ ДЕМОНСТРАЦИЯ: SIZED VS UNSIZED
// ============================================================================

/// Демонстрирует разницу между использованием обычных типов (Sized)
/// и trait objects (Unsized) в контексте ?Sized bound
fn demonstrate_sized_vs_unsized() {
    println!("=== ДЕМОНСТРАЦИЯ: SIZED VS UNSIZED ===");
    println!();
    
    // Создаем пользователя и команду
    let user = User::new(1, "demo@example.com", true);
    let create_cmd = CreateUser::new("demo_user@example.com", false);
    
    // ========================================================================
    // ИСПОЛЬЗОВАНИЕ ОБЫЧНОГО ТИПА (SIZED)
    // ========================================================================
    
    println!("1. ИСПОЛЬЗОВАНИЕ ОБЫЧНОГО ТИПА (MockUserRepository):");
    println!("   - Тип известен во время компиляции");
    println!("   - Статическая диспетчеризация");
    println!("   - Нет накладных расходов на vtable lookup");
    println!();
    
    let mut mock_repo = MockUserRepository::new();
    mock_repo.add_user(user.clone());
    
    match user.handle_command(&create_cmd, &mut mock_repo) {
        Ok(()) => println!("   ✓ Команда успешно обработана с MockUserRepository"),
        Err(e) => println!("   ✗ Ошибка: {}", e),
    }
    
    println!();
    
    // ========================================================================
    // ИСПОЛЬЗОВАНИЕ TRAIT OBJECT (UNSIZED)
    // ========================================================================
    
    println!("2. ИСПОЛЬЗОВАНИЕ TRAIT OBJECT (dyn UserRepository):");
    println!("   - Тип стирается во время компиляции");
    println!("   - Динамическая диспетчеризация через vtable");
    println!("   - Небольшие накладные расходы на vtable lookup");
    println!("   - Гибкость в выборе реализации во время выполнения");
    println!();
    
    // Создаем trait object
    let mut mock_repo_for_trait = MockUserRepository::new();
    mock_repo_for_trait.add_user(user.clone());
    let mut trait_object_repo: Box<dyn UserRepository> = Box::new(mock_repo_for_trait);
    
    match user.handle_command(&create_cmd, &mut *trait_object_repo) {
        Ok(()) => println!("   ✓ Команда успешно обработана с dyn UserRepository"),
        Err(e) => println!("   ✗ Ошибка: {}", e),
    }
    
    println!();
    
    // ========================================================================
    // ДЕМОНСТРАЦИЯ ГИБКОСТИ TRAIT OBJECTS
    // ========================================================================
    
    println!("3. ГИБКОСТЬ TRAIT OBJECTS:");
    println!("   - Можно легко переключаться между реализациями");
    println!("   - Подходит для гетерогенных коллекций");
    println!("   - Позволяет создавать более универсальные функции");
    println!();
    
    // Демонстрируем, что можно использовать разные реализации
    let new_cmd = CreateUser::new("flexible@example.com", true);
    let mut another_mock_repo = MockUserRepository::new();
    another_mock_repo.add_user(user.clone());
    
    match user.handle_command(&new_cmd, &mut another_mock_repo) {
        Ok(()) => println!("   ✓ Команда обработана с другой реализацией репозитория"),
        Err(e) => println!("   ✗ Ошибка: {}", e),
    }
    
    println!();
    
    // ========================================================================
    // ОБЪЯСНЕНИЕ ?SIZED BOUND
    // ========================================================================
    
    println!("4. ЗАЧЕМ НУЖЕН ?SIZED BOUND:");
    println!("   - Без ?Sized: все типы параметров имеют неявную границу Sized");
    println!("   - С ?Sized: можно использовать как Sized, так и Unsized типы");
    println!("   - Позволяет создавать более гибкие API");
    println!("   - Обеспечивает совместимость с trait objects");
    println!();
    
    println!("ПРИМЕР БЕЗ ?SIZED BOUND:");
    println!("  trait BadExample {{");
    println!("      fn method(&self);");
    println!("  }}");
    println!("  // Context: T - только Sized типы (обычные структуры)");
    println!("  // НЕ МОЖЕТ: dyn BadExample, [T], str");
    println!();
    
    println!("ПРИМЕР С ?SIZED BOUND:");
    println!("  trait GoodExample {{");
    println!("      fn method(&self);");
    println!("  }}");
    println!("  // Context: T: ?Sized - и Sized, и Unsized типы");
    println!("  // МОЖЕТ: MockUserRepository, dyn GoodExample, [T], str");
    println!();
    
    println!("ВЫВОД:");
    println!("  ?Sized bound - это ключ к созданию гибких и универсальных API");
    println!("  в Rust, которые работают как с обычными типами, так и с trait objects");
}

// ============================================================================
// ТЕСТЫ
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_handler_success() {
        // Создаем пользователя и команду
        let user = User::new(1, "admin@example.com", true);
        let create_cmd = CreateUser::new("newuser@example.com", false);
        
        // Создаем mock репозиторий
        let mut mock_repo = MockUserRepository::new();
        mock_repo.add_user(user.clone());
        
        // Обрабатываем команду
        let result = user.handle_command(&create_cmd, &mut mock_repo);
        
        // Проверяем результат
        assert!(result.is_ok());
        
        // Проверяем, что пользователь был создан
        let created_user = mock_repo.find_user_by_email("newuser@example.com").unwrap();
        assert!(created_user.is_some());
        
        let created_user = created_user.unwrap();
        assert_eq!(created_user.email, "newuser@example.com");
        assert_eq!(created_user.activated, false);
    }

    #[test]
    fn test_command_handler_user_already_exists() {
        // Создаем пользователя и команду с существующим email
        let user = User::new(1, "admin@example.com", true);
        let create_cmd = CreateUser::new("admin@example.com", false);
        
        // Создаем mock репозиторий
        let mut mock_repo = MockUserRepository::new();
        mock_repo.add_user(user.clone());
        
        // Обрабатываем команду
        let result = user.handle_command(&create_cmd, &mut mock_repo);
        
        // Проверяем, что получили ошибку
        assert!(result.is_err());
        
        match result.unwrap_err() {
            UserError::UserAlreadyExists(email) => {
                assert_eq!(email, "admin@example.com");
            }
            _ => panic!("Ожидалась ошибка UserAlreadyExists"),
        }
    }

    #[test]
    fn test_command_handler_invalid_email() {
        // Создаем пользователя и команду с некорректным email
        let user = User::new(1, "admin@example.com", true);
        let create_cmd = CreateUser::new("invalid-email", false);
        
        // Создаем mock репозиторий
        let mut mock_repo = MockUserRepository::new();
        mock_repo.add_user(user.clone());
        
        // Обрабатываем команду
        let result = user.handle_command(&create_cmd, &mut mock_repo);
        
        // Проверяем, что получили ошибку
        assert!(result.is_err());
        
        match result.unwrap_err() {
            UserError::InvalidEmail(email) => {
                assert_eq!(email, "invalid-email");
            }
            _ => panic!("Ожидалась ошибка InvalidEmail"),
        }
    }

    #[test]
    fn test_mock_repository_operations() {
        let mut repo = MockUserRepository::new();
        
        // Создаем пользователя
        let user = User::new(1, "test@example.com", true);
        
        // Сохраняем пользователя
        assert!(repo.save_user(user.clone()).is_ok());
        
        // Находим пользователя по ID
        let found_user = repo.find_user_by_id(1).unwrap();
        assert_eq!(found_user, Some(user.clone()));
        
        // Находим пользователя по email
        let found_user = repo.find_user_by_email("test@example.com").unwrap();
        assert_eq!(found_user, Some(user.clone()));
        
        // Удаляем пользователя
        let deleted_user = repo.delete_user(1).unwrap();
        assert_eq!(deleted_user, Some(user));
        
        // Проверяем, что пользователь удален
        let found_user = repo.find_user_by_id(1).unwrap();
        assert_eq!(found_user, None);
    }

    #[test]
    fn test_mock_repository_duplicate_user() {
        let mut repo = MockUserRepository::new();
        
        // Создаем первого пользователя
        let user1 = User::new(1, "test@example.com", true);
        assert!(repo.save_user(user1).is_ok());
        
        // Пытаемся создать пользователя с тем же email
        let user2 = User::new(2, "test@example.com", false);
        let result = repo.save_user(user2);
        
        // Проверяем, что получили ошибку
        assert!(result.is_err());
        
        match result.unwrap_err() {
            UserError::UserAlreadyExists(email) => {
                assert_eq!(email, "test@example.com");
            }
            _ => panic!("Ожидалась ошибка UserAlreadyExists"),
        }
    }

    #[test]
    fn test_command_trait() {
        let cmd = CreateUser::new("test@example.com", true);
        assert_eq!(cmd.command_type(), "CreateUser");
    }

    #[test]
    fn test_user_error_display() {
        let error = UserError::UserAlreadyExists("test@example.com".to_string());
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("test@example.com"));
        assert!(error_msg.contains("уже существует"));
    }
}
