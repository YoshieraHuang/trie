mod node;
mod error;

use node::Node;
use std::hash::Hash;
use error::{Result, Error};
// use std::iter::Iterator;

#[derive(Default)]
pub struct Trie<V> {
    // 根结点
    root: Box<Node<V>>,
    // 分割key的各个token的字符
    sc: char,
    // 替代一个token的wildcard字符串
    owc: &'static str,
    // 替代多个token的wildcard字符串
    mwc: &'static str,
}

impl<V: Default + Clone + Eq + Hash> Trie<V> {
    // 初始化
    pub fn new(sc: char, owc: &'static str, mwc: &'static str) -> Trie<V> {
        Trie {
            root: Default::default(),
            sc: sc,
            owc: owc,
            mwc: mwc,
        }
    }

    // 从给出的key中得到各级token，如果key为空，则会返回error
    fn tokens_from_key(key: &'static str, sc: char) -> Result<impl Iterator<Item=&'static str>> {
        if key.len() == 0 {
            return Err(Error::EmptyToken(key.to_owned()))
        }

        Ok(key.split(sc))
    }


    // 添加键值对
    pub fn insert(&mut self, key: &'static str, value: V) -> Result<()> {
        let (node, hasmwc) = self.must_find_mut_node(key)?;
        // 找到之后就把value给放进去，如果存在mwc则放在mwc里面去
        if hasmwc { node.mwc_add(value); } else { node.add(value); }
        Ok(())
    }

    // 返回键对应的所有值的迭代器，如果不存在键，则返回空迭代器
    pub fn find(&self, key: &'static str) -> Result<impl Iterator<Item=&V>> {
        let sc = self.sc;
        let owc = self.owc;
        let mwc = self.mwc;
        let mut hasmwc = false;
        // 保存结果
        let mut values: Vec<&V> = Vec::new();
        // 迭代key来获得最终node
        let mut nodes = Self::tokens_from_key(key, sc)?
            // 待处理的nodes
            .try_fold(vec![self.root.as_ref(), ],
                |nodes, token| {
                    Self::check_token(token, key, hasmwc)?;
                    
                    // 如果是空node，那就不用查找了
                    if nodes.len() == 0 {
                        return Ok(nodes);
                    }
                    
                    // 把当前的nodes中的所有的多层wildcard的组给放到values中
                    for node in nodes.iter() {
                        for m_value in node.mwc_values() {
                            values.push(m_value);
                        }
                    }
                    
                    if token == owc {
                        // 如果是单层，就要把当前node的所有children都带进去
                        let mut next_nodes: Vec<&Node<V>> = Vec::new();
                        for node in nodes.into_iter() {
                            // 先把订阅单层的那些放进去
                            if let Some(ref o_node) = node.onode {
                                // 只用给引用
                                next_nodes.push(o_node);
                            }
                            // 把所有子节点的引用都放进去
                            for child_node in node.children.values() {
                                next_nodes.push(child_node);
                            }
                        }
                        Ok(next_nodes)
                    // 如果是多层的
                    } else if token == mwc {
                        hasmwc = true;
                        Ok(nodes)
                    } else {
                        // 普通token，直接去children里面取就好了
                        let mut next_nodes: Vec<&Node<V>> = Vec::new();
                        for node in nodes.into_iter() {
                            if let Some(n) = node.children.get(token) {
                                next_nodes.push(n);
                            }
                        }
                        Ok(next_nodes)
                    }
                })?;
        let mut i = 0;
        while nodes.len() > i {
            let node = nodes[i];
            // 把当前node的所有value都放到values中去
            values.extend(node.values());
            // 如果有hasmwc，则要放入mwc值并迭代子节点
            if hasmwc {
                values.extend(node.mwc_values());
                if let Some(ref onode) = node.onode {
                    nodes.push(onode);
                }

                for child_node in node.child_nodes() {
                    nodes.push(child_node);
                }
            }
            i += 1;
        }

        // 如果有多层wildcard，需要把所有的values都
        Ok(values.into_iter())
    }

    // 移除key对应的组中的value值。如果存在key组并且其中有value值，返回true。
    // 如果不存在key组或者keys组中没有value值，返回false
    pub fn remove(&mut self, key: &'static str, value: &V) -> Result<bool> {
        let (node, hasmwc) = self.find_mut_node(key)?;

        Ok(node.map(|n|
                if hasmwc {
                    n.mwc_remove(value)
                } else {
                    n.remove(value)
                }
            )
            .unwrap_or(false)
        )
    }

    // 移除key对应的组中的所有value。如果存在keys则返回true，如果不存在则返回false
    pub fn remove_all(&mut self, key: &'static str) -> Result<bool> {
        let (node, hasmwc) = self.find_mut_node(key)?;
    
        Ok(node.map(|n|
                if hasmwc {
                    n.mwc_remove_all()
                } else {
                    n.remove_all()
                }
            )
            .unwrap_or(false)
        )
    }

    // 找到key对应的node，返回其引用，如果没有，则返回None
    fn find_node(&self, key: &'static str) -> Result<(Option<&Node<V>>, bool)> {
        let sc = self.sc;
        let mwc = self.mwc;
        let owc = self.owc;
        let mut hasmwc = false;
        // 将key分成token
        let node = Self::tokens_from_key(key, sc)?
            // 查找token对应的node，如果没有token就返回None
            .try_fold(Some(& *self.root),
                |node, token| {
                    Self::check_token(token, key, hasmwc)?;
                    Ok(node.and_then(|n| {
                        match token {
                            // 是mwc，直接返回当前的node
                            _ if token == mwc => { hasmwc = true; Some(n) },
                            // 是owc，则返回owc面对的owc
                            _ if token == owc => { n.owc_node() },
                            // 找到token对应的子结点，如果没有则返回None
                            _ => { n.get_child_node(token) }
                        }
                    }))
                })?;
        Ok((node, hasmwc))
    }

    // 是否有对应的key存在
    pub fn exist(&self, key: &'static str) -> Result<bool> {
        let sc = self.sc;
        let owc = self.owc;
        let mwc = self.mwc;
        let mut hasmwc = false;
        let mut nodes = vec![self.root.as_ref(), ];
        for token in Self::tokens_from_key(key, sc)? {
            Self::check_token(token, key, hasmwc)?;
            // 没有node，就不会存在key
            if nodes.len() == 0 {
                return Ok(false);
            }
            
            // 多层wildcard中有值，返回true
            if nodes.iter().any(|n| !n.is_mwc_empty_values()) {
                return Ok(true)
            }
            
            // 如果有多层wildcard，就要要在
            if token == mwc {
                hasmwc = true;
                continue;
            }

            let mut next_nodes: Vec<&Node<V>> = Vec::new();
            if token == owc {
                for node in nodes.into_iter() {
                    if let Some(ref onode) = node.onode {
                        next_nodes.push(onode);
                    }
                    for child_node in node.child_nodes() {
                        next_nodes.push(child_node);
                    }
                }
            } else {
                for node in nodes.into_iter() {
                    if let Some(n) = node.children.get(token) {
                        next_nodes.push(n);
                    }
                }
            }
            nodes = next_nodes;
        }

        // nodes中任意一个node含有value就认为是存在的
        if nodes.iter().any(|n| !n.is_empty_values()) {
            Ok(true)
        } else {
            if hasmwc {
                let mut i = 0;
                while nodes.len() > i {
                    let node = nodes[i];
                    // 如果有values，认为存在
                    if !node.is_empty_values() || !node.is_mwc_empty_values() {
                        return Ok(true)
                    }

                    // 如果有hasmwc，则要迭代子节点
                    if let Some(ref onode) = node.onode {
                        nodes.push(onode);
                    }
    
                    for child_node in node.child_nodes() {
                        nodes.push(child_node);
                    }
                    i += 1;
                }
                Ok(false)
            } else {
                Ok(false)
            }
        }
    }

    // 找到key对应的node，返回其引用。如果没有对应node存在，则创建
    fn must_find_mut_node(&mut self, key: &'static str) -> Result<(&mut Node<V>, bool)> {
        let sc = self.sc;
        let owc = self.owc;
        let mwc = self.mwc;
        // 是否遇到过了mwc
        let mut hasmwc = false;
        // 找到对应的node
        let node = Self::tokens_from_key(key, sc)?
            .try_fold(&mut *self.root,
                |node, token| {
                    Self::check_token(token, key, hasmwc)?;
                    Ok(match token {
                        _ if token == mwc => { hasmwc = true; node },
                        _ if token == owc => node.mut_owc_node(),
                        _ => node.get_or_insert(token)
                    })
            }
        )?;
        Ok((node, hasmwc))
    }

    // 找到key对应的node，返回其可变引用
    fn find_mut_node(&mut self, key: &'static str) -> Result<(Option<&mut Node<V>>, bool)> {
        let sc = self.sc;
        let mwc = self.mwc;
        let owc = self.owc;
        let mut hasmwc = false;
        // 将key分成token
        let node = Self::tokens_from_key(key, sc)?
            // 查找token对应的node，如果没有token就返回None
            .try_fold(Some(&mut *self.root),
                |node, token| {
                    Self::check_token(token, key, hasmwc)?;
                    Ok(node.and_then(|n| {
                        match token {
                            // 是mwc，直接返回当前的node
                            _ if token == mwc => { hasmwc = true; Some(n) },
                            // 是owc，则返回owc面对的owc
                            _ if token == owc => { Some(n.mut_owc_node()) },
                            // 找到token对应的子结点，如果没有则返回None
                            _ => { n.get_mut_child_node(token) }
                        }
                    }))
                })?;
        Ok((node, hasmwc))
    }

    fn check_token(token: &'static str, key: &'static str, hasmwc: bool) -> Result<()> {
        // token长度为0，返回错误
        if token.len() == 0 {
            Err(Error::EmptyToken(key.to_owned()))
        } else if hasmwc {
        // 过了mwc还出现了token，返回错误
            Err(Error::TokenAfterMwc(key.to_owned()))
        } else {
            Ok(())
        }
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

    fn is_empty<V: Hash + Eq>(iter: impl Iterator<Item=V>) -> bool {
        iter.collect::<Vec<V>>().len() == 0
    }

    #[test]
    fn basic_trie() {
        let mut trie = Trie::new('.', "*", ">");
        assert!(trie.insert("a", 10).is_ok());
        assert!(trie.insert("a", 10).is_ok());
        assert!(trie.insert("a.b", 5).is_ok());
        assert!(trie.insert("a", 8).is_ok());
        assert!(trie.insert("a.b.c", 12).is_ok());
        assert!(vec_eq(trie.find("a").unwrap(), vec![&10, &8]));
        assert!(vec_eq(trie.find("a.b").unwrap(), vec![&5,]));
        assert!(vec_eq(trie.find("a.b.c").unwrap(), vec![&12,]));
        assert!(is_empty(trie.find("b").unwrap()));
        assert!(is_empty(trie.find("c").unwrap()));
        assert!(trie.remove("a", &10).unwrap());
        assert!(trie.remove("b", &1).unwrap() == false);
        assert!(trie.remove("a.b", &5).unwrap());
        assert!(trie.remove("a", &5).unwrap() == false);
        assert!(vec_eq(trie.find("a").unwrap(), vec![&8, ]));
        assert!(is_empty(trie.find("a.b").unwrap()));
        assert!(trie.remove("a", &5).unwrap() == false);
        assert!(vec_eq(trie.find("a.b.c").unwrap(), vec![&12, ]));
        assert!(trie.insert("a.b.c", 15).is_ok());
        assert!(trie.insert("a.b.c", 17).is_ok());
        assert!(trie.remove_all("a.b.c").unwrap());
        assert!(is_empty(trie.find("a.b.c").unwrap()));
        assert!(trie.remove_all("a").unwrap());
        assert!(trie.remove_all("a.b").unwrap() == false);
        assert!(trie.remove_all("xyz").unwrap() == false);
    }

    #[test]
    fn illegal_subject() {
        let mut trie = Trie::new('.', "*", ">");
        assert_eq!(trie.insert("", 0).err(), Some(Error::EmptyToken("".to_owned())));
        assert_eq!(trie.find("").err(), Some(Error::EmptyToken("".to_owned())));
        assert_eq!(trie.find("a..").err(), Some(Error::EmptyToken("a..".to_owned())));
        assert_eq!(trie.find("...").err(), Some(Error::EmptyToken("...".to_owned())));
        assert!(is_empty(trie.find(">").unwrap()));
        assert_eq!(trie.find(">.").err(), Some(Error::EmptyToken(">.".to_owned())));
        assert_eq!(trie.find(">.a").err(), Some(Error::TokenAfterMwc(">.a".to_owned())));
    }

    #[test]
    fn wc_trie() {
        let mut trie = Trie::new('.', "*", ">");
        assert!(is_empty(trie.find("a").unwrap()));
        assert!(is_empty(trie.find("b").unwrap()));
        assert!(is_empty(trie.find("a.b").unwrap()));
        assert!(is_empty(trie.find("*").unwrap()));
        assert!(is_empty(trie.find("*.b").unwrap()));
        assert!(is_empty(trie.find(">").unwrap()));
        assert!(trie.insert("a", 8).is_ok());
        assert!(is_empty(trie.find("a.b").unwrap()));
        assert!(is_empty(trie.find("b").unwrap()));
        assert!(vec_eq(trie.find("a").unwrap(), vec![&8]));
        assert!(vec_eq(trie.find("*").unwrap(), vec![&8]));
        assert!(vec_eq(trie.find(">").unwrap(), vec![&8]));
        assert!(is_empty(trie.find("*.b").unwrap()));
        assert!(trie.insert("a.b", 9).is_ok());
        assert!(vec_eq(trie.find("a").unwrap(), vec![&8]));
        assert!(vec_eq(trie.find("*").unwrap(), vec![&8]));
        assert!(vec_eq(trie.find("a.b").unwrap(), vec![&9]));
        assert!(vec_eq(trie.find("*.b").unwrap(), vec![&9]));
        assert!(vec_eq(trie.find("a.*").unwrap(), vec![&9]));
        assert!(vec_eq(trie.find(">").unwrap(), vec![&8, &9]));
        assert!(trie.insert("a.c", 1).is_ok());
        assert!(vec_eq(trie.find("a").unwrap(), vec![&8]));
        assert!(vec_eq(trie.find("*").unwrap(), vec![&8]));
        assert!(vec_eq(trie.find("a.b").unwrap(), vec![&9]));
        assert!(vec_eq(trie.find("*.b").unwrap(), vec![&9]));
        assert!(vec_eq(trie.find("a.*").unwrap(), vec![&9, &1]));
        assert!(vec_eq(trie.find(">").unwrap(), vec![&8, &9, &1]));
    }
}