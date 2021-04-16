use std::collections::{HashMap, HashSet};
use std::collections::hash_set::Iter;
use std::hash::Hash;

#[derive(Default)]
pub struct Node<V> {
    // 子结点
    pub children: HashMap<&'static str, Box<Node<V>>>,
    // 当前结点对应的值
    pub value_set: HashSet<V>,
}

impl<V: Default + Clone + Eq + Hash> Node<V> {
    // 生成一个不带object的新节点
    pub fn new() -> Self {
        return Node{
            children: HashMap::new(),
            value_set: HashSet::new(),
        }
    }

    // 添加一个value
    pub fn add_value(&mut self, object: V) -> bool {
        self.value_set.insert(object)
    }

    // 返回当前的objects
    pub fn values(&self) -> Iter<'_, V>{
        self.value_set.iter()
    }

    // 移除一个object
    pub fn remove_value(&mut self, object: &V) -> bool {
        self.value_set.remove(object)
    }

    // 不存在value
    pub fn has_no_values(&self) -> bool {
        self.value_set.is_empty()
    }

    // 移除所有的value，如果当前有值，则返回true。如果本身没有值，则返回false
    pub fn remove_all(&mut self) -> bool {
        if self.value_set.is_empty() {
            false
        } else {
            self.value_set.clear();
            true
        }
    }
}