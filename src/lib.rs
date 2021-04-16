mod node;

use node::Node;
use std::hash::Hash;

#[derive(Default)]
pub struct Trie<V> {
    root: Box<Node<V>>,
    split_char: char,
}

impl<V: Default + Clone + Eq + Hash> Trie<V> {
    // 初始化
    pub fn new_with_split_char(split_char: char) -> Trie<V> {
        Trie {
            root: Default::default(),
            split_char: split_char,
        }
    }

    // 添加键值对，键已经被分割
    fn insert_by_keys(&mut self, keys: impl Iterator<Item=&'static str>, value: V){
        // node中有key，则用key对应的node，node中无key，则生成node并添加到树中
        keys.fold(&mut self.root,
                |node, key| node.children.entry(key).or_insert(Box::new(Node::new())))
            // 找到之后就把value给放进去
            .add_value(value);
    }

    // 添加键值对
    pub fn insert(&mut self, key: &'static str, value: V) {
        let keys = key.split(self.split_char);
        self.insert_by_keys(keys, value);
    }

    // 查找一个键，返回所有值，键已经被分割
    fn find_by_keys(&self, keys: impl Iterator<Item=&'static str>) -> Option<impl Iterator<Item=&V>> {
        // 迭代key来获得最终node，如果没有key则为None
        keys.fold(Some(&self.root),
                |node, key| node.and_then(|n| n.children.get(key)))
            // 如果node中没有values，也算作没有结果，需要返回None
            .and_then(|n| if n.has_no_values() { None } else { Some(n) })
            // 返回values这个迭代器
            .map(|n| n.values())
    }

    // 返回键对应的所有值，如果不存在键，则返回None
    pub fn find(&self, key: &'static str) -> Option<impl Iterator<Item=&V>> {
        let keys = key.split(self.split_char);
        self.find_by_keys(keys)
    }

    // 移除keys对应的组中的value值。如果存在keys组并且其中有value值，返回true。
    // 如果不存在keys组或者keys组中没有value值，返回false
    fn remove_by_keys(&mut self, keys: impl Iterator<Item=&'static str>, value: &V) -> bool {
        // 找到对应的node，如果没有key，则为None
        keys.fold(Some(&mut self.root),
                |node, key| node.and_then(|n| n.children.get_mut(key)))
            // 把最后node中的value给remove掉
            .map(|n| n.remove_value(value))
            // 如果node为None返回false
            .unwrap_or(false)
    }

    // 移除key对应的组中的value值。如果存在key组并且其中有value值，返回true。
    // 如果不存在key组或者keys组中没有value值，返回false
    pub fn remove(&mut self, key: &'static str, value: &V) -> bool {
        let keys = key.split(self.split_char);
        self.remove_by_keys(keys, value)
    }

    // 移除keys对应的组中的所有value。如果存在keys则返回true，如果不存在则返回false
    fn remove_all_by_keys(&mut self, keys: impl Iterator<Item=&'static str>) -> bool {
        keys.fold(Some(&mut self.root),
                |node, key| node.and_then(|n| n.children.get_mut(key)))
            .map(|n| n.remove_all())
            .unwrap_or(false)
    }

    // 移除key对应的组中的所有value。如果存在keys则返回true，如果不存在则返回false
    pub fn remove_all(&mut self, key: &'static str) -> bool {
        let keys = key.split(self.split_char);
        self.remove_all_by_keys(keys)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::collections::HashSet;

    // 两个迭代器中的元素在忽略顺序的情况下是否一一相等
    fn vec_eq<V: Hash + Eq>(vec1: impl Iterator<Item=V>, vec2: Vec<V>) -> bool{
        let set1: HashSet<V> = vec1.collect();
        let set2: HashSet<V> = vec2.into_iter().collect();
        set1 == set2
    }

    #[test]
    fn trie() {
        let mut tree = Trie::new_with_split_char('.');
        tree.insert("a", 10);
        tree.insert("a.b", 5);
        tree.insert("a", 8);
        tree.insert("a.b.c", 12);
        assert!(vec_eq(tree.find("a").unwrap(), vec![&10, &8]));
        assert!(vec_eq(tree.find("a.b").unwrap(), vec![&5,]));
        assert!(vec_eq(tree.find("a.b.c").unwrap(), vec![&12,]));
        assert!(tree.find("b").is_none());
        assert!(tree.find("c").is_none());
        assert!(tree.remove("a", &10));
        assert!(tree.remove("b", &1) == false);
        assert!(tree.remove("a.b", &5));
        assert!(tree.remove("a", &5) == false);
        assert!(vec_eq(tree.find("a").unwrap(), vec![&8, ]));
        assert!(tree.find("a.b").is_none());
        assert!(tree.remove("a", &5) == false);
        assert!(vec_eq(tree.find("a.b.c").unwrap(), vec![&12, ]));
        tree.insert("a.b.c", 15);
        tree.insert("a.b.c", 17);
        assert!(tree.remove_all("a.b.c"));
        assert!(tree.find("a.b.c").is_none());
        assert!(tree.remove_all("a"));
        assert!(tree.remove_all("a.b") == false);
        assert!(tree.remove_all("xyz") == false);
    }
}