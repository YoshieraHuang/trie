mod node;
pub mod token;

pub use token::{Token, Tokens};
use node::Node;
use std::hash::Hash;
use lru_map::LRUMap;

#[derive(Default)]
pub struct Trie<'a, V, const N: usize> {
    // 查询结果的缓存
    cache: LRUMap<Vec<&'a str>, Vec<V>, N>,
    // 根结点
    root: Box<Node<'a, V>>,
}

impl<'a, V, const N: usize> Trie<'a, V, N>
where
    V: Eq + Hash + Clone
{
    /// 初始化
    pub fn new() -> Trie<'a, V, N> {
        Trie {
            cache: LRUMap::default(),
            root: Box::new(Node::new()),
        }
    }

    /// 添加键值对
    pub fn insert(&mut self, tokens: &Tokens<'a>, value: V) {
        // 查找对应的节点
        let (node, is_mwc) = self.must_find_node_mut(tokens);
        // 找到之后就把value给放进去，如果存在mwc则放在mwc里面去
        if is_mwc {
            node.mwc_add(value.clone());
        } else {
            node.add(value.clone());
        }

        // 删除与当前tokens匹配的缓存结果，因为已经过期
        self.cache.remove(|keys| tokens.match_keys(keys));
    }

    /// 返回能与keys匹配的所有值的迭代器，如果不存在键，返回空迭代器
    pub fn find(&mut self, keys: impl AsRef<[&'a str]>) -> Vec<V> {
        let keys = keys.as_ref().to_vec();
        // 先查找cache，如果命中就返回
        if let Some(res) = self.cache.get(&keys) {
            return (*res).clone();
        }

        // 保存结果
        let mut values: Vec<V> = Vec::new();
        // 迭代key来获得最终node
        let nodes = keys.iter()
            // 待处理的nodes
            .try_fold(vec![self.root.as_ref(), ],
                |nodes, token| {
                    // 如果是空node，那就不用查找了
                    if nodes.len() == 0 {
                        return Err(());
                    }
                    
                    let mut next_nodes: Vec<&Node<V>> = Vec::new();
                    for node in nodes.into_iter() {
                        // 多层wildcard必然满足tokens的需求，所以直接添加到values中
                        values.extend(node.mwc_values_owned());
                        // 符合当前token的node可以是token对应的，也可以是owc对应的
                        next_nodes.extend(node.owc_node());
                        if let Some(n) = node.get_child_node(token) {
                            next_nodes.push(n);
                        }
                    }
                    Ok(next_nodes)
                }).unwrap_or(vec![]);
        // 先迭代mwc中的结果
        values.extend(nodes.into_iter().flat_map(|n| n.values_owned()));
        self.cache.put(keys, values.clone());
        values
    }

    /// 移除tokens对应的组中的value值。如果存在tokens组并且其中有value值，返回true。
    /// 如果不存在tokens组或者tokens组中没有value值，返回false
    pub fn remove(&mut self, tokens: &Tokens<'a>, value: &V) -> bool {
        self.cache.remove(|keys| tokens.match_keys(keys));
        match self.find_node_mut(tokens) {
            None => false,
            Some((node, hasmwc)) => {
                if hasmwc {
                    node.mwc_remove(value)
                } else {
                    node.remove(value)
                }
            }
        }
    }

    /// 移除key对应的组中的所有value。如果存在keys则返回true，如果不存在则返回false
    pub fn remove_all(&mut self, tokens: &Tokens<'a>) -> bool {
        self.cache.remove(|keys| tokens.match_keys(keys));
        match self.find_node_mut(tokens) {
            None => false,
            Some((node, hasmwc)) => 
                if hasmwc {
                    node.mwc_remove_all()
                } else {
                    node.remove_all()
                }
        }
    }

    /// 找到key对应的node，返回其引用，如果没有，则返回None
    #[allow(dead_code)]
    fn find_node(&self, tokens: &Tokens<'a>) -> (Option<&Node<V>>, bool) {
        let mut hasmwc = false;
        let value = tokens.0.iter()
            // 查找token对应的node，如果没有token就返回None
            .fold(Some(& *self.root),
                |node, token| {
                    node.and_then(|n| {
                        match token {
                            Token::MultiWildcard => {
                                hasmwc = true;
                                Some(n)
                            },
                            Token::OneWildcard => {
                                n.owc_node()
                            },
                            Token::Normal(s) => {
                                n.get_child_node(s)
                            }
                        }
                    })
                });
        (value, hasmwc)
    }

    // 是否有与keys匹配的值存在，包含带有wildcard的
    pub fn exist(&self, keys: impl AsRef<[&'a str]>) -> bool {
        // 迭代key来获得最终node
        // 其中try_fold里面的Result没有错误的含义，只是用来使用Err来短路迭代
        let nodes = keys.as_ref().iter()
            // 待处理的nodes
            .try_fold(vec![self.root.as_ref(), ],
                |nodes, token| {
                    // 如果是空node，那就不用查找了
                    if nodes.len() == 0 {
                        return Err(false);
                    }
                    let mut next_nodes: Vec<&Node<V>> = Vec::new();
                    for node in nodes.into_iter() {
                        // 存在mwc的结果则肯定有匹配值
                        if !node.is_mwc_empty() { return Err(true); }
                        // 符合当前token的node可以是token对应的，也可以是owc对应的
                        next_nodes.extend(node.owc_node());
                        if let Some(n) = node.get_child_node(token) {
                            next_nodes.push(n);
                        }
                    }
                    Ok(next_nodes)
                }
            );
        match nodes {
            // 短路，直接输出内部包含值
            Err(v) => { return v; },
            // 没有短路，查找匹配的nodes中是否有值
            Ok(ns) => {
                for n in ns.into_iter() {
                    if !n.is_empty() { return true; }
                }
                return false;
            }
        }
    }

    // 找到key对应的node，返回其可变引用。如果没有对应node存在，则创建
    fn must_find_node_mut(&mut self, tokens: &Tokens<'a>) -> (&mut Node<'a, V>, bool) {
        // 是否遇到过了mwc
        let mut hasmwc = false;
        // 找到对应的node
        let node = tokens.0.iter()
            .fold(&mut *self.root,
                |node, token| {
                    match token {
                        Token::MultiWildcard => {
                            hasmwc = true;
                            node
                        },
                        Token::OneWildcard => node.owc_node_mut(),
                        Token::Normal(s) => node.get_child_node_mut_or_insert(s)
                    }
            }
        );
        (node, hasmwc)
    }

    // 找到key对应的node，返回其可变引用。如果没有，则返回None
    fn find_node_mut(&mut self, tokens: &Tokens<'a>) -> Option<(&mut Node<'a, V>, bool)> {
        let mut hasmwc = false;
        tokens.0.iter()
            // 查找token对应的node，如果没有token就返回None
            .try_fold(&mut *self.root,
                |node, token| {
                    match token {
                        Token::MultiWildcard => {
                            hasmwc = true;
                            Some(node)
                        },
                        Token::OneWildcard => {
                            Some(node.owc_node_mut())
                        },
                        Token::Normal(s) => {
                            node.get_child_node_mut(s)
                        }
                    }
                }
            )
            .map(|node| (node, hasmwc))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::token::*;
    use std::collections::HashSet;

    // 两个迭代器中的元素在忽略顺序的情况下是否一一相等
    fn vec_eq<V: Hash + Eq>(vec1: Vec<V>, vec2: Vec<V>) -> bool{
        let set1: HashSet<V> = vec1.into_iter().collect();
        let set2: HashSet<V> = vec2.into_iter().collect();
        set1 == set2
    }

    #[test]
    fn test_basic_trie() -> Result<(), CommonTokenError> {
        let mut trie = Trie::<_, 10>::new();
        let parser = CommonTokenParser::new('.', "*", ">");
        trie.insert(&parser.parse_tokens("a")?, 1);
        trie.insert(&parser.parse_tokens("a")?, 2);
        trie.insert(&parser.parse_tokens("")?, 3);
        trie.insert(&parser.parse_tokens("a.b")?, 5);
        trie.insert(&parser.parse_tokens(".")?, 6);
        trie.insert(&parser.parse_tokens("a")?, 8);
        trie.insert(&parser.parse_tokens("a.b.c")?, 12);
        assert!(vec_eq(trie.find(&["a"]), vec![1, 2, 8]));
        assert!(vec_eq(trie.find(&[""]), vec![3, ]));
        assert!(vec_eq(trie.find(&["a", "b"]), vec![5, ]));
        assert!(vec_eq(trie.find(&["", ""]), vec![6, ]));
        assert!(vec_eq(trie.find(&["a", "b", "c"]), vec![12,]));
        assert_eq!(trie.find(vec!["b"]).len(), 0);
        assert_eq!(trie.find(vec!["c"]).len(), 0);
        assert_eq!(trie.remove(&parser.parse_tokens("a")?, &1), true);
        assert_eq!(trie.remove(&parser.parse_tokens("a")?, &1), false);
        assert_eq!(trie.remove(&parser.parse_tokens("a.b")?, &5), true);
        assert_eq!(trie.remove(&parser.parse_tokens("a")?, &5), false);
        assert!(vec_eq(trie.find(vec!["a"]), vec![2, 8, ]));
        assert_eq!(trie.find(vec!["a", "b"]).len(), 0);
        assert!(vec_eq(trie.find(vec!["a", "b", "c"]), vec![12, ]));
        assert_eq!(trie.remove(&parser.parse_tokens("a.b")?, &5), false);
        trie.insert(&parser.parse_tokens("a.b.c")?, 15);
        trie.insert(&parser.parse_tokens("a.b.c")?, 17);
        assert_eq!(trie.remove_all(&parser.parse_tokens("a.b.c")?), true);
        assert_eq!(trie.find(vec!["a", "b", "c"]).len(), 0);
        assert_eq!(trie.remove_all(&parser.parse_tokens("a")?), true);
        assert_eq!(trie.remove_all(&parser.parse_tokens("a.b")?), false);
        assert_eq!(trie.remove_all(&parser.parse_tokens("x.y.z")?), false);
        Ok(())
    }

    #[test]
    fn test_trie_with_wildcard() -> Result<(), CommonTokenError> {
        let mut trie = Trie::<_, 10>::new();
        let parser = CommonTokenParser::new('.', "*", ">");
        trie.insert(&parser.parse_tokens("a")?, 1);
        trie.insert(&parser.parse_tokens("a.b")?, 2);
        trie.insert(&parser.parse_tokens("")?, 3);
        trie.insert(&parser.parse_tokens("*")?, 4);
        trie.insert(&parser.parse_tokens(">")?, 5);
        trie.insert(&parser.parse_tokens("*.c")?, 6);
        trie.insert(&parser.parse_tokens("a.*.c")?, 7);
        trie.insert(&parser.parse_tokens("a.>")?, 8);

        assert!(vec_eq(trie.find(vec!["a"]), vec![1, 4, 5]));
        assert!(vec_eq(trie.find(vec!["b"]), vec![4, 5]));
        assert!(vec_eq(trie.find(vec!["a", "b"]), vec![2, 5, 8]));
        assert!(vec_eq(trie.find(vec!["a", "c"]), vec![5, 6, 8]));
        assert!(vec_eq(trie.find(vec!["a", "b", "c"]), vec![5, 7, 8]));
        Ok(())
    }
}