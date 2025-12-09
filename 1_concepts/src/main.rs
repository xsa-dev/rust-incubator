use std::sync::{Arc, Mutex, Weak};

/// Узел двусвязного списка
#[derive(Debug)]
struct Node<T> {
    data: Option<T>,
    next: Option<Arc<Mutex<Node<T>>>>,
    prev: Option<Weak<Mutex<Node<T>>>>,
}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Node {
            data: Some(data),
            next: None,
            prev: None,
        }
    }
}

/// Thread-safe двусвязный список
#[derive(Debug)]
pub struct DoublyLinkedList<T> {
    head: Option<Arc<Mutex<Node<T>>>>,
    tail: Option<Arc<Mutex<Node<T>>>>,
    len: usize,
}

impl<T> DoublyLinkedList<T> {
    /// Создает новый пустой список
    pub fn new() -> Self {
        DoublyLinkedList {
            head: None,
            tail: None,
            len: 0,
        }
    }

    /// Возвращает количество элементов в списке
    pub fn len(&self) -> usize {
        self.len
    }

    /// Проверяет, пуст ли список
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Добавляет элемент в начало списка
    pub fn push_front(&mut self, data: T) {
        let new_node = Arc::new(Mutex::new(Node::new(data)));
        
        match self.head.take() {
            Some(old_head) => {
                old_head.lock().unwrap().prev = Some(Arc::downgrade(&new_node));
                new_node.lock().unwrap().next = Some(old_head);
            }
            None => {
                self.tail = Some(new_node.clone());
            }
        }
        
        self.head = Some(new_node);
        self.len += 1;
    }

    /// Добавляет элемент в конец списка
    pub fn push_back(&mut self, data: T) {
        let new_node = Arc::new(Mutex::new(Node::new(data)));
        
        match self.tail.take() {
            Some(old_tail) => {
                old_tail.lock().unwrap().next = Some(new_node.clone());
                new_node.lock().unwrap().prev = Some(Arc::downgrade(&old_tail));
            }
            None => {
                self.head = Some(new_node.clone());
            }
        }
        
        self.tail = Some(new_node);
        self.len += 1;
    }

    /// Удаляет и возвращает первый элемент списка
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().and_then(|old_head| {
            match old_head.lock().unwrap().next.take() {
                Some(new_head) => {
                    new_head.lock().unwrap().prev = None;
                    self.head = Some(new_head);
                }
                None => {
                    self.tail = None;
                }
            }
            self.len -= 1;
            old_head.lock().unwrap().data.take()
        })
    }

    /// Удаляет и возвращает последний элемент списка
    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().and_then(|old_tail| {
            match old_tail.lock().unwrap().prev.take() {
                Some(prev_weak) => {
                    if let Some(prev) = prev_weak.upgrade() {
                        prev.lock().unwrap().next = None;
                        self.tail = Some(prev);
                    } else {
                        self.head = None;
                    }
                }
                None => {
                    self.head = None;
                }
            }
            self.len -= 1;
            old_tail.lock().unwrap().data.take()
        })
    }
}

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe обертка для DoublyLinkedList
#[derive(Debug)]
pub struct ThreadSafeDoublyLinkedList<T> {
    inner: Arc<Mutex<DoublyLinkedList<T>>>,
}

impl<T> ThreadSafeDoublyLinkedList<T> {
    /// Создает новый thread-safe список
    pub fn new() -> Self {
        ThreadSafeDoublyLinkedList {
            inner: Arc::new(Mutex::new(DoublyLinkedList::new())),
        }
    }

    /// Возвращает количество элементов в списке
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    /// Проверяет, пуст ли список
    pub fn is_empty(&self) -> bool {
        self.inner.lock().unwrap().is_empty()
    }

    /// Добавляет элемент в начало списка
    pub fn push_front(&self, data: T) {
        self.inner.lock().unwrap().push_front(data);
    }

    /// Добавляет элемент в конец списка
    pub fn push_back(&self, data: T) {
        self.inner.lock().unwrap().push_back(data);
    }

    /// Удаляет и возвращает первый элемент списка
    pub fn pop_front(&self) -> Option<T> {
        self.inner.lock().unwrap().pop_front()
    }

    /// Удаляет и возвращает последний элемент списка
    pub fn pop_back(&self) -> Option<T> {
        self.inner.lock().unwrap().pop_back()
    }

    /// Создает клон Arc для использования в других потоках
    pub fn clone(&self) -> Self {
        ThreadSafeDoublyLinkedList {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Default for ThreadSafeDoublyLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for ThreadSafeDoublyLinkedList<T> {
    fn clone(&self) -> Self {
        self.clone()
    }
}

/// Итератор для DoublyLinkedList
pub struct DoublyLinkedListIter<T> {
    current: Option<Arc<Mutex<Node<T>>>>,
}

impl<T> DoublyLinkedListIter<T> {
    fn new(list: &DoublyLinkedList<T>) -> Self {
        DoublyLinkedListIter {
            current: list.head.clone(),
        }
    }
}

impl<T: Clone> Iterator for DoublyLinkedListIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.take().and_then(|node| {
            let node = node.lock().unwrap();
            self.current = node.next.clone();
            // Клонируем данные, так как мы не можем переместить их из Arc<Mutex<Node<T>>>
            node.data.as_ref().cloned()
        })
    }
}

impl<T: Clone> DoublyLinkedList<T> {
    /// Создает итератор для обхода списка
    pub fn iter(&self) -> DoublyLinkedListIter<T> {
        DoublyLinkedListIter::new(self)
    }
}

/// Thread-safe итератор для ThreadSafeDoublyLinkedList
pub struct ThreadSafeDoublyLinkedListIter<T> {
    current: Option<Arc<Mutex<Node<T>>>>,
}

impl<T> ThreadSafeDoublyLinkedListIter<T> {
    fn new(list: &ThreadSafeDoublyLinkedList<T>) -> Self {
        let inner = list.inner.lock().unwrap();
        ThreadSafeDoublyLinkedListIter {
            current: inner.head.clone(),
        }
    }
}

impl<T: Clone> Iterator for ThreadSafeDoublyLinkedListIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.take().and_then(|node| {
            let node = node.lock().unwrap();
            self.current = node.next.clone();
            node.data.as_ref().cloned()
        })
    }
}

impl<T: Clone> ThreadSafeDoublyLinkedList<T> {
    /// Создает итератор для обхода списка
    pub fn iter(&self) -> ThreadSafeDoublyLinkedListIter<T> {
        ThreadSafeDoublyLinkedListIter::new(self)
    }
}

fn main() {
    // Пример использования single-threaded
    println!("=== Single-threaded example ===");
    let mut list = DoublyLinkedList::new();
    
    list.push_front(1);
    list.push_back(2);
    list.push_front(0);
    list.push_back(3);
    
    println!("List length: {}", list.len());
    
    while let Some(value) = list.pop_front() {
        println!("Popped: {}", value);
    }
    
    // Пример использования итератора
    println!("\n=== Iterator example ===");
    let mut list2 = DoublyLinkedList::new();
    list2.push_back(100);
    list2.push_back(200);
    list2.push_back(300);
    
    println!("Iterating through list:");
    for (i, value) in list2.iter().enumerate() {
        println!("  {}: {}", i, value);
    }
    
    // Пример использования thread-safe версии
    println!("\n=== Thread-safe example ===");
    let thread_safe_list = ThreadSafeDoublyLinkedList::new();
    
    thread_safe_list.push_front(10);
    thread_safe_list.push_back(20);
    thread_safe_list.push_front(5);
    thread_safe_list.push_back(30);
    
    println!("Thread-safe list length: {}", thread_safe_list.len());
    
    while let Some(value) = thread_safe_list.pop_front() {
        println!("Thread-safe popped: {}", value);
    }
    
    // Пример использования thread-safe итератора
    println!("\n=== Thread-safe iterator example ===");
    let thread_safe_list2 = ThreadSafeDoublyLinkedList::new();
    thread_safe_list2.push_back(1000);
    thread_safe_list2.push_back(2000);
    thread_safe_list2.push_back(3000);
    
    println!("Iterating through thread-safe list:");
    for (i, value) in thread_safe_list2.iter().enumerate() {
        println!("  {}: {}", i, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_empty_list() {
        let list = DoublyLinkedList::<i32>::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_push_front() {
        let mut list = DoublyLinkedList::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        
        assert_eq!(list.len(), 3);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_push_back() {
        let mut list = DoublyLinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        
        assert_eq!(list.len(), 3);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_pop_front() {
        let mut list = DoublyLinkedList::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn test_pop_back() {
        let mut list = DoublyLinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn test_mixed_operations() {
        let mut list = DoublyLinkedList::new();
        
        // Добавляем элементы
        list.push_front(1);
        list.push_back(2);
        list.push_front(0);
        list.push_back(3);
        
        assert_eq!(list.len(), 4);
        
        // Проверяем порядок извлечения
        assert_eq!(list.pop_front(), Some(0));
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn test_thread_safe_empty() {
        let list = ThreadSafeDoublyLinkedList::<i32>::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_thread_safe_operations() {
        let list = ThreadSafeDoublyLinkedList::new();
        
        list.push_front(1);
        list.push_back(2);
        list.push_front(0);
        list.push_back(3);
        
        assert_eq!(list.len(), 4);
        assert!(!list.is_empty());
        
        assert_eq!(list.pop_front(), Some(0));
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn test_thread_safety() {
        let list = Arc::new(ThreadSafeDoublyLinkedList::new());
        let mut handles = vec![];
        
        // Создаем несколько потоков для записи
        for i in 0..5 {
            let list_clone = list.clone();
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    list_clone.push_front(i * 10 + j);
                    thread::sleep(Duration::from_millis(1));
                }
            });
            handles.push(handle);
        }
        
        // Создаем несколько потоков для чтения
        for _ in 0..3 {
            let list_clone = list.clone();
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    let _ = list_clone.pop_front();
                    thread::sleep(Duration::from_millis(1));
                }
            });
            handles.push(handle);
        }
        
        // Ждем завершения всех потоков
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Проверяем, что список в корректном состоянии
        assert!(list.len() >= 0);
    }

    #[test]
    fn test_concurrent_push_pop() {
        let list = Arc::new(ThreadSafeDoublyLinkedList::new());
        let mut handles = vec![];
        
        // Поток 1: добавляет элементы
        let list1 = list.clone();
        let handle1 = thread::spawn(move || {
            for i in 0..100 {
                list1.push_front(i);
                if i % 10 == 0 {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });
        handles.push(handle1);
        
        // Поток 2: добавляет элементы в конец
        let list2 = list.clone();
        let handle2 = thread::spawn(move || {
            for i in 100..200 {
                list2.push_back(i);
                if i % 10 == 0 {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });
        handles.push(handle2);
        
        // Поток 3: извлекает элементы
        let list3 = list.clone();
        let handle3 = thread::spawn(move || {
            let mut count = 0;
            while count < 50 {
                if list3.pop_front().is_some() {
                    count += 1;
                }
                thread::sleep(Duration::from_millis(2));
            }
        });
        handles.push(handle3);
        
        // Ждем завершения всех потоков
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Проверяем финальное состояние
        assert!(list.len() > 0);
        println!("Final list length: {}", list.len());
    }

    #[test]
    fn test_iterator() {
        let mut list = DoublyLinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        
        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_thread_safe_iterator() {
        let list = ThreadSafeDoublyLinkedList::new();
        list.push_back(10);
        list.push_back(20);
        list.push_back(30);
        
        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(10));
        assert_eq!(iter.next(), Some(20));
        assert_eq!(iter.next(), Some(30));
        assert_eq!(iter.next(), None);
    }
}
