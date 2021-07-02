use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_set::Iter;
use std::hash::Hash;

/// trie树结点
#[derive(Default, Debug)]
pub struct Node<V> {
    // 子结点
    children: HashMap<&'static str, Box<Node<V>>>,
    // 订阅了单层wildcard对应的node
    o_node: Option<Box<Node<V>>>,
    // 订阅了多层wildcard对应的组
    m_value_set: HashSet<V>,
    // 当前结点对应的值
    value_set: HashSet<V>,
}

impl<V:Eq + Hash> Node<V> {
    /// 生成一个新节点
    pub(crate) fn new() -> Self {
        return Node{
            children: HashMap::new(),
            value_set: HashSet::new(),
            o_node: None,
            m_value_set: HashSet::new(),
        }
    }

    /// 添加一个value
    pub(crate) fn add(&mut self, value: V) -> bool {
        self.value_set.insert(value)
    }

    /// 返回当前的values
    pub(crate) fn values(&self) -> Iter<'_, V>{
        self.value_set.iter()
    }

    /// 移除一个value
    pub(crate) fn remove(&mut self, value: &V) -> bool {
        self.value_set.remove(value)
    }

    /// 不存在value
    pub(crate) fn is_empty(&self) -> bool {
        self.value_set.is_empty()
    }

    /// 移除所有的value，如果当前有值，则返回true。如果本身没有值，则返回false
    pub(crate) fn remove_all(&mut self) -> bool {
        if self.is_empty() {
            false
        } else {
            self.value_set.clear();
            true
        }
    }

    /// 所有子节点的不可变引用
    #[allow(dead_code)]
    fn child_nodes(&self) -> impl Iterator<Item=&Node<V>> {
        self.children.values().map(|n| n.as_ref())
    }

    /// 所有子节点的可变引用
    #[allow(dead_code)]
    fn child_nodes_mut(&mut self) -> impl Iterator<Item=&mut Node<V>> {
        self.children.values_mut().map(|n| n.as_mut())
    }
    
    /// 返回单层wildcard对应的node的不可变引用，如果已经有node，则返回，如果没有对应node，则创建并返回
    pub(crate) fn owc_node(&self) -> Option<&Node<V>> {
        self.o_node.as_ref().map(|n| (*n).as_ref())
    }

    /// 返回单层wildcard对应的node的可变引用，如果已经有node，则返回，如果没有对应node，则创建并返回
    pub(crate) fn owc_node_mut(&mut self) -> &mut Node<V> {
        // 如果是None则插入新的值，并返回对应的引用
        self.o_node.get_or_insert(Box::new(Node::new()))
    }

    /// 向多层wildcard组中插入值
    pub(crate) fn mwc_add(&mut self, value: V) -> bool {
        self.m_value_set.insert(value)
    }

    /// 从多层wildcard组中移除值
    pub(crate) fn mwc_remove(&mut self, value: &V) -> bool {
        self.m_value_set.remove(value)
    }

    /// 返回多层wildcard组中所有的值的引用
    pub(crate) fn mwc_values(&self) -> Iter<'_, V> {
        self.m_value_set.iter()
    }

    /// 多层wildcard组是否是空的
    pub(crate) fn is_mwc_empty(&self) -> bool {
        self.m_value_set.is_empty()
    }

    /// 移除多层wildcard组中所有的值
    pub(crate) fn mwc_remove_all(&mut self) -> bool {
        if self.is_mwc_empty() {
            false
        } else {
            self.m_value_set.clear();
            true
        }
    }

    /// 获得一个token对应的子节点。如果不存在，则创建
    pub(crate) fn get_child_node_mut_or_insert(&mut self, token: &'static str) -> &mut Node<V> {
        self.children.entry(token).or_insert(Box::new(Node::new()))
    }

    /// 返回token对应的子节点的可变引用
    pub(crate) fn get_child_node_mut(&mut self, token: &'static str) -> Option<&mut Node<V>> {
        self.children.get_mut(token).map(|n| (*n).as_mut())
    }

    /// 返回token对应的子节点的不可变引用
    pub(crate) fn get_child_node(&self, token: &'static str) -> Option<&Node<V>> {
        self.children.get(token).map(|n| (*n).as_ref())
    }
}