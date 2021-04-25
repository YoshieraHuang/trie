use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_set::Iter;
use std::hash::Hash;

#[derive(Default, Debug)]
pub struct Node<V> {
    // 子结点
    children: HashMap<&'static str, Box<Node<V>>>,
    // 订阅了单层wildcard对应的node
    onode: Option<Box<Node<V>>>,
    // 订阅了多层wildcard对应的组
    m_value_set: HashSet<V>,
    // 当前结点对应的值
    value_set: HashSet<V>,
}

impl<V: Default + Clone + Eq + Hash> Node<V> {
    // 生成一个不带object的新节点
    pub fn new() -> Self {
        return Node{
            children: HashMap::new(),
            value_set: HashSet::new(),
            onode: None,
            m_value_set: HashSet::new(),
        }
    }

    // 添加一个value
    pub fn add(&mut self, value: V) -> bool {
        self.value_set.insert(value)
    }

    // 返回当前的values
    pub fn values(&self) -> Iter<'_, V>{
        self.value_set.iter()
    }

    // 移除一个value
    pub fn remove(&mut self, vaue: &V) -> bool {
        self.value_set.remove(vaue)
    }

    // 不存在value
    pub fn is_empty_values(&self) -> bool {
        self.value_set.is_empty()
    }

    // 移除所有的value，如果当前有值，则返回true。如果本身没有值，则返回false
    pub fn remove_all(&mut self) -> bool {
        if self.is_empty_values() {
            false
        } else {
            self.value_set.clear();
            true
        }
    }

    // 所有子节点的不可变引用
    pub fn child_nodes(&self) -> impl Iterator<Item=&Node<V>> {
        self.children.values().map(|n| n.as_ref())
    }

    // 所有子节点的可变引用
    pub fn child_mut_nodes(&mut self) -> impl Iterator<Item=&mut Node<V>> {
        self.children.values_mut().map(|n| n.as_mut())
    }

    // 返回单层wildcard对应的node的可变引用，如果已经有node，则返回，如果没有对应node，则创建并返回
    pub fn mut_owc_node(&mut self) -> &mut Node<V> {
        // 如果是None则插入新的值，并返回对应的引用
        self.onode.get_or_insert(Box::new(Node::new()))
    }

    // 返回单层wildcard对应的node的不可变引用，如果已经有node，则返回，如果没有对应node，则创建并返回
    pub fn owc_node(&self) -> Option<&Node<V>> {
        self.onode.as_ref().map(|n| (*n).as_ref())
    }

    // 向多层wildcard组中插入值
    pub fn mwc_add(&mut self, value: V) -> bool {
        self.m_value_set.insert(value)
    }

    // 从多层wildcard组中移除值
    pub fn mwc_remove(&mut self, value: &V) -> bool {
        self.m_value_set.remove(value)
    }

    // 返回多层wildcard组中所有的值的引用
    pub fn mwc_values(&self) -> Iter<'_, V> {
        self.m_value_set.iter()
    }

    // 多层wildcard组是否是空的
    pub fn is_mwc_empty_values(&self) -> bool {
        self.m_value_set.is_empty()
    }

    // 移除多层wildcard组中所有的值
    pub fn mwc_remove_all(&mut self) -> bool {
        if self.is_mwc_empty_values() {
            false
        } else {
            self.m_value_set.clear();
            true
        }
    }

    pub fn get_or_insert(&mut self, token: &'static str) -> &mut Node<V> {
        self.children.entry(token).or_insert(Box::new(Node::new()))
    }

    pub fn get_mut_child_node(&mut self, token: &'static str) -> Option<&mut Node<V>> {
        self.children.get_mut(token).map(|n| (*n).as_mut())
    }

    pub fn get_child_node(&self, token: &'static str) -> Option<&Node<V>> {
        self.children.get(token).map(|n| (*n).as_ref())
    }
}