use std::rc::Rc;
use std::cell::RefCell;

/// A global stack that can be shared and mutated through multiple references.
/// 
/// This implementation uses `Rc<RefCell<Vec<T>>>` to achieve:
/// - Shared ownership: multiple owners can hold references to the same stack
/// - Interior mutability: the stack can be mutated through shared references
/// - Clone semantics: cloning only creates a new reference, not a copy of data
#[derive(Clone)]
pub struct GlobalStack<T> {
    data: Rc<RefCell<Vec<T>>>,
}

impl<T> GlobalStack<T> {
    /// Creates a new empty GlobalStack.
    pub fn new() -> Self {
        Self {
            data: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Pushes an element onto the stack.
    pub fn push(&self, item: T) {
        self.data.borrow_mut().push(item);
    }

    /// Pops an element from the stack.
    /// Returns `None` if the stack is empty.
    pub fn pop(&self) -> Option<T> {
        self.data.borrow_mut().pop()
    }

    /// Returns the number of elements in the stack.
    pub fn len(&self) -> usize {
        self.data.borrow().len()
    }

    /// Returns `true` if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.data.borrow().is_empty()
    }

    /// Returns a reference to the top element without removing it.
    /// Returns `None` if the stack is empty.
    pub fn peek(&self) -> Option<std::cell::Ref<'_, T>> {
        if self.is_empty() {
            None
        } else {
            Some(std::cell::Ref::map(self.data.borrow(), |v| &v[v.len() - 1]))
        }
    }
}

impl<T> Default for GlobalStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    // Test basic functionality
    let stack1 = GlobalStack::new();
    
    // Push some elements
    stack1.push(1);
    stack1.push(2);
    stack1.push(3);
    
    println!("Stack1 length: {}", stack1.len()); // Should be 3
    
    // Clone the stack - this creates a new reference to the same data
    let stack2 = stack1.clone();
    
    // Both stacks point to the same data
    println!("Stack2 length: {}", stack2.len()); // Should be 3
    
    // Mutate through stack2
    stack2.push(4);
    
    // Both stacks see the same change
    println!("Stack1 length after stack2.push(4): {}", stack1.len()); // Should be 4
    println!("Stack2 length after stack2.push(4): {}", stack2.len()); // Should be 4
    
    // Pop from stack1
    if let Some(value) = stack1.pop() {
        println!("Popped from stack1: {}", value); // Should be 4
    }
    
    // Both stacks see the same change
    println!("Stack1 length after pop: {}", stack1.len()); // Should be 3
    println!("Stack2 length after pop: {}", stack2.len()); // Should be 3
    
    // Test peek functionality
    if let Some(top) = stack1.peek() {
        println!("Top element: {}", *top); // Should be 3
    }
    
    // Test with strings
    let string_stack = GlobalStack::new();
    string_stack.push("Hello".to_string());
    string_stack.push("World".to_string());
    
    let string_stack2 = string_stack.clone();
    string_stack2.push("Rust".to_string());
    
    println!("String stack length: {}", string_stack.len()); // Should be 3
    println!("String stack2 length: {}", string_stack2.len()); // Should be 3
    
    // Pop all elements
    while let Some(s) = string_stack.pop() {
        println!("Popped: {}", s);
    }
    
    println!("Final string stack length: {}", string_stack.len()); // Should be 0
    println!("Final string stack2 length: {}", string_stack2.len()); // Should be 0
}
