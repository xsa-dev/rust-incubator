//! Дерево с разделяемыми узлами: Rc<RefCell<Node>> + Weak для родителя, чтобы избежать циклов

use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,              // слабая ссылка на родителя — не увеличивает счётчик
    children: RefCell<Vec<Rc<Node>>>,         // сильные ссылки на детей
}

impl Node {
    fn new(value: i32) -> Rc<Node> {
        Rc::new(Node {
            value,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
        })
    }

    fn add_child(parent: &Rc<Node>, child: Rc<Node>) {
        // установить parent у ребёнка (weak, чтобы не образовывать цикл Rc)
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        // добавить ребёнка в список
        parent.children.borrow_mut().push(child);
    }
}

fn main() {
    let root = Node::new(1);
    let left = Node::new(2);
    let right = Node::new(3);

    Node::add_child(&root, Rc::clone(&left));
    Node::add_child(&root, Rc::clone(&right));

    println!("root strong = {}, weak = {}", Rc::strong_count(&root), Rc::weak_count(&root));
    println!("left strong = {}, weak = {}", Rc::strong_count(&left), Rc::weak_count(&left));

    // Достаём родителя через Weak::upgrade()
    if let Some(parent) = left.parent.borrow().upgrade() {
        println!("Родитель left: {}", parent.value);
    } else {
        println!("У left нет родителя");
    }

    // Посмотрим на детей root
    let children_vals: Vec<_> = root.children
        .borrow()
        .iter()
        .map(|c| c.value)
        .collect();
    println!("Дети root: {children_vals:?}");

    // При выходе из main все Rc считаются и корректно освобождаются.
}