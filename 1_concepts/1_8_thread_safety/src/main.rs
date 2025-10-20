use std::marker::PhantomData;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::cell::RefCell;
use std::rc::Rc;

/// OnlySync - Sync, но !Send
/// 
/// Этот тип может быть безопасно разделен между потоками (Sync),
/// но не может быть перемещен между потоками (!Send).
/// 
/// Реализация использует Arc<RefCell<T>>, который является Sync,
/// но не Send, так как RefCell не является Send.
#[derive(Debug, Clone)]
pub struct OnlySync<T> {
    /// Arc<RefCell<T>> является Sync, но не Send
    /// Arc позволяет множественное владение между потоками (Sync)
    /// RefCell не может быть отправлен между потоками (!Send)
    data: Arc<RefCell<T>>,
    /// PhantomData для дополнительной информации о типе
    _phantom: PhantomData<T>,
}

impl<T> OnlySync<T> {
    /// Создает новый экземпляр OnlySync
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(RefCell::new(data)),
            _phantom: PhantomData,
        }
    }
    
    /// Получает неизменяемую ссылку на данные
    pub fn get(&self) -> std::cell::Ref<'_, T> {
        self.data.borrow()
    }
    
    /// Получает изменяемую ссылку на данные
    pub fn get_mut(&self) -> std::cell::RefMut<'_, T> {
        self.data.borrow_mut()
    }
    
    /// Получает количество ссылок
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }
}

/// OnlySend - Send, но !Sync
/// 
/// Этот тип может быть перемещен между потоками (Send),
/// но не может быть безопасно разделен между потоками (!Sync).
/// 
/// Реализация использует RefCell<T>, который является Send,
/// но не Sync, так как RefCell не является Sync.
#[derive(Debug)]
pub struct OnlySend<T> {
    /// RefCell<T> является Send, но не Sync
    /// RefCell обеспечивает внутреннюю мутабельность, но не может быть разделен между потоками
    data: RefCell<T>,
    /// PhantomData для дополнительной информации о типе
    _phantom: PhantomData<T>,
}

impl<T> OnlySend<T> {
    /// Создает новый экземпляр OnlySend
    pub fn new(data: T) -> Self {
        Self {
            data: RefCell::new(data),
            _phantom: PhantomData,
        }
    }
    
    /// Получает неизменяемую ссылку на данные
    pub fn get(&self) -> std::cell::Ref<'_, T> {
        self.data.borrow()
    }
    
    /// Получает изменяемую ссылку на данные
    pub fn get_mut(&self) -> std::cell::RefMut<'_, T> {
        self.data.borrow_mut()
    }
}

/// SyncAndSend - и Sync, и Send
/// 
/// Этот тип может быть как перемещен между потоками (Send),
/// так и безопасно разделен между потоками (Sync).
/// 
/// Реализация использует Arc<Mutex<T>>, который является и Send, и Sync.
#[derive(Debug, Clone)]
pub struct SyncAndSend<T> {
    /// Arc<Mutex<T>> является и Send, и Sync
    /// Arc обеспечивает атомарное подсчет ссылок для множественного владения
    /// Mutex обеспечивает внутреннюю мутабельность с блокировкой
    data: Arc<Mutex<T>>,
    /// PhantomData для дополнительной информации о типе
    _phantom: PhantomData<T>,
}

impl<T> SyncAndSend<T> {
    /// Создает новый экземпляр SyncAndSend
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
            _phantom: PhantomData,
        }
    }
    
    /// Получает неизменяемую ссылку на данные
    pub fn get(&self) -> std::sync::MutexGuard<'_, T> {
        self.data.lock().unwrap()
    }
    
    /// Получает количество ссылок
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }
}

/// NotSyncNotSend - !Sync и !Send
/// 
/// Этот тип не может быть ни перемещен между потоками (!Send),
/// ни безопасно разделен между потоками (!Sync).
/// 
/// Реализация использует Rc<RefCell<T>>, который не является ни Send, ни Sync.
#[derive(Debug, Clone)]
pub struct NotSyncNotSend<T> {
    /// Rc<RefCell<T>> не является ни Send, ни Sync
    /// Rc не является Send (не может быть перемещен между потоками)
    /// RefCell не является Sync (не может быть разделен между потоками)
    data: Rc<RefCell<T>>,
    /// PhantomData для дополнительной информации о типе
    _phantom: PhantomData<T>,
}

impl<T> NotSyncNotSend<T> {
    /// Создает новый экземпляр NotSyncNotSend
    pub fn new(data: T) -> Self {
        Self {
            data: Rc::new(RefCell::new(data)),
            _phantom: PhantomData,
        }
    }
    
    /// Получает неизменяемую ссылку на данные
    pub fn get(&self) -> std::cell::Ref<'_, T> {
        self.data.borrow()
    }
    
    /// Получает изменяемую ссылку на данные
    pub fn get_mut(&self) -> std::cell::RefMut<'_, T> {
        self.data.borrow_mut()
    }
    
    /// Получает количество ссылок
    pub fn strong_count(&self) -> usize {
        Rc::strong_count(&self.data)
    }
}

/// Демонстрация работы с OnlySync
fn demonstrate_only_sync() {
    println!("=== Демонстрация OnlySync (Sync, но !Send) ===");
    
    let only_sync = OnlySync::new(42);
    println!("Создан OnlySync с значением: {}", only_sync.get());
    
    // OnlySync является Sync, поэтому можно создать несколько ссылок
    let _only_sync_clone = only_sync.clone();
    println!("Клонирован OnlySync, количество ссылок: {}", only_sync.strong_count());
    
    // Попытка отправить OnlySync в другой поток приведет к ошибке компиляции
    // Это демонстрирует, что OnlySync не является Send
    println!("OnlySync не может быть отправлен в другой поток (не Send)");
    
    // OnlySync является Sync, поэтому можно безопасно использовать из разных потоков
    // через Arc, но сам OnlySync не может быть перемещен между потоками
    println!("OnlySync является Sync - может быть разделен между потоками");
    println!("OnlySync не является Send - не может быть перемещен между потоками\n");
}

/// Демонстрация работы с OnlySend
fn demonstrate_only_send() {
    println!("=== Демонстрация OnlySend (Send, но !Sync) ===");
    
    let only_send = OnlySend::new(42);
    println!("Создан OnlySend с значением: {}", only_send.get());
    
    // OnlySend является Send, поэтому можно отправить в другой поток
    let handle = thread::spawn(move || {
        println!("В другом потоке: {}", only_send.get());
    });
    
    handle.join().unwrap();
    println!("OnlySend успешно отправлен в другой поток\n");
}

/// Демонстрация работы с SyncAndSend
fn demonstrate_sync_and_send() {
    println!("=== Демонстрация SyncAndSend (и Sync, и Send) ===");
    
    let sync_and_send = SyncAndSend::new(42);
    println!("Создан SyncAndSend с значением: {}", sync_and_send.get());
    
    // SyncAndSend является и Sync, и Send
    let sync_and_send_clone = sync_and_send.clone();
    println!("Клонирован SyncAndSend, количество ссылок: {}", sync_and_send.strong_count());
    
    // Можно отправить в другой поток
    let handle = thread::spawn(move || {
        println!("В другом потоке: {}", sync_and_send_clone.get());
    });
    
    handle.join().unwrap();
    println!("SyncAndSend успешно использован из разных потоков\n");
}

/// Демонстрация работы с NotSyncNotSend
fn demonstrate_not_sync_not_send() {
    println!("=== Демонстрация NotSyncNotSend (!Sync и !Send) ===");
    
    let not_sync_not_send = NotSyncNotSend::new(42);
    println!("Создан NotSyncNotSend с значением: {}", not_sync_not_send.get());
    
    // NotSyncNotSend не является ни Sync, ни Send
    let _not_sync_not_send_clone = not_sync_not_send.clone();
    println!("Клонирован NotSyncNotSend, количество ссылок: {}", not_sync_not_send.strong_count());
    
    // Попытка отправить NotSyncNotSend в другой поток приведет к ошибке компиляции
    // Это демонстрирует, что NotSyncNotSend не является Send
    println!("NotSyncNotSend не может быть отправлен в другой поток (не Send)");
    println!("NotSyncNotSend не может быть разделен между потоками (не Sync)\n");
}

/// Демонстрация fearless concurrency
fn demonstrate_fearless_concurrency() {
    println!("=== Демонстрация Fearless Concurrency ===");
    
    // Создаем данные, которые будут разделены между потоками
    let shared_data = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    // Создаем несколько потоков, которые будут изменять общие данные
    for i in 0..5 {
        let shared_data = Arc::clone(&shared_data);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                let mut data = shared_data.lock().unwrap();
                *data += 1;
            }
            println!("Поток {} завершил работу", i);
        });
        handles.push(handle);
    }
    
    // Ждем завершения всех потоков
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Финальное значение: {}", shared_data.lock().unwrap());
    println!("Все потоки завершили работу без data races!\n");
}

/// Демонстрация различных типов синхронизации
fn demonstrate_synchronization_types() {
    println!("=== Демонстрация различных типов синхронизации ===");
    
    // Mutex для эксклюзивного доступа
    let mutex_data = Arc::new(Mutex::new(0));
    let mutex_clone = Arc::clone(&mutex_data);
    
    // RwLock для множественного чтения или эксклюзивной записи
    let rwlock_data = Arc::new(RwLock::new(0));
    let rwlock_clone = Arc::clone(&rwlock_data);
    
    let handle1 = thread::spawn(move || {
        for _ in 0..1000 {
            let mut data = mutex_clone.lock().unwrap();
            *data += 1;
        }
    });
    
    let handle2 = thread::spawn(move || {
        for _ in 0..1000 {
            let mut data = rwlock_clone.write().unwrap();
            *data += 1;
        }
    });
    
    handle1.join().unwrap();
    handle2.join().unwrap();
    
    println!("Mutex результат: {}", mutex_data.lock().unwrap());
    println!("RwLock результат: {}", rwlock_data.read().unwrap());
    println!("Оба типа синхронизации работают корректно!\n");
}

fn main() {
    println!("=== Демонстрация Thread Safety в Rust ===\n");
    
    // Демонстрация различных типов Send/Sync
    demonstrate_only_sync();
    demonstrate_only_send();
    demonstrate_sync_and_send();
    demonstrate_not_sync_not_send();
    
    // Демонстрация fearless concurrency
    demonstrate_fearless_concurrency();
    
    // Демонстрация различных типов синхронизации
    demonstrate_synchronization_types();
    
    println!("=== Объяснение Send и Sync ===");
    println!("Send: Тип может быть безопасно перемещен между потоками");
    println!("Sync: Тип может быть безопасно разделен между потоками");
    println!("\nОсновные принципы:");
    println!("1. Send + Sync = можно использовать в многопоточном коде");
    println!("2. Send + !Sync = можно перемещать, но не разделять");
    println!("3. !Send + Sync = можно разделять, но не перемещать");
    println!("4. !Send + !Sync = нельзя использовать в многопоточном коде");
    println!("\nFearless Concurrency означает, что компилятор Rust");
    println!("гарантирует отсутствие data races на этапе компиляции!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    
    #[test]
    fn test_only_sync_creation() {
        let only_sync = OnlySync::new(42);
        assert_eq!(*only_sync.get(), 42);
        assert_eq!(only_sync.strong_count(), 1);
    }
    
    #[test]
    fn test_only_sync_cloning() {
        let only_sync = OnlySync::new(42);
        let clone = only_sync.clone();
        assert_eq!(only_sync.strong_count(), 2);
        assert_eq!(*clone.get(), 42);
    }
    
    #[test]
    fn test_only_send_creation() {
        let only_send = OnlySend::new(42);
        assert_eq!(*only_send.get(), 42);
    }
    
    #[test]
    fn test_only_send_send() {
        let only_send = OnlySend::new(42);
        let handle = thread::spawn(move || {
            assert_eq!(*only_send.get(), 42);
        });
        handle.join().unwrap();
    }
    
    #[test]
    fn test_sync_and_send_creation() {
        let sync_and_send = SyncAndSend::new(42);
        assert_eq!(*sync_and_send.get(), 42);
        assert_eq!(sync_and_send.strong_count(), 1);
    }
    
    #[test]
    fn test_sync_and_send_cloning() {
        let sync_and_send = SyncAndSend::new(42);
        let clone = sync_and_send.clone();
        assert_eq!(sync_and_send.strong_count(), 2);
        assert_eq!(*clone.get(), 42);
    }
    
    #[test]
    fn test_sync_and_send_send() {
        let sync_and_send = SyncAndSend::new(42);
        let handle = thread::spawn(move || {
            assert_eq!(*sync_and_send.get(), 42);
        });
        handle.join().unwrap();
    }
    
    #[test]
    fn test_not_sync_not_send_creation() {
        let not_sync_not_send = NotSyncNotSend::new(42);
        assert_eq!(*not_sync_not_send.get(), 42);
        assert_eq!(not_sync_not_send.strong_count(), 1);
    }
    
    #[test]
    fn test_not_sync_not_send_cloning() {
        let not_sync_not_send = NotSyncNotSend::new(42);
        let clone = not_sync_not_send.clone();
        assert_eq!(not_sync_not_send.strong_count(), 2);
        assert_eq!(*clone.get(), 42);
    }
    
    #[test]
    fn test_thread_safety_with_arc() {
        let data = Arc::new(Mutex::new(0));
        let mut handles = vec![];
        
        for _ in 0..10 {
            let data = Arc::clone(&data);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let mut data = data.lock().unwrap();
                    *data += 1;
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert_eq!(*data.lock().unwrap(), 1000);
    }
    
    #[test]
    fn test_mutation_safety() {
        let only_sync = OnlySync::new(0);
        let clone = only_sync.clone();
        
        // Изменяем данные через одну ссылку
        *only_sync.get_mut() = 42;
        
        // Проверяем, что изменения видны через другую ссылку
        assert_eq!(*clone.get(), 42);
    }
}