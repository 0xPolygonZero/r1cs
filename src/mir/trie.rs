#[derive(Debug)]
pub struct Trie {
    depth: usize,
    root: Node,
}

impl Trie {
    pub fn new(depth: usize) -> Self {
        Trie { depth, root: Node::Empty }
    }

    pub fn contains(&self, value: &[bool]) -> bool {
        assert_eq!(value.len(), self.depth);
        self.root.contains(value)
    }

    pub fn insert(&mut self, value: &[bool]) {
        assert_eq!(value.len(), self.depth);
        self.root.insert(value);
    }
}

#[derive(Debug)]
enum Node {
    Leaf,
    Empty,
    Intermediate {
        child_0: Box<Node>,
        child_1: Box<Node>,
    },
}

impl Node {
    fn contains(&self, value: &[bool]) -> bool {
        match self {
            Node::Leaf => {
                assert!(value.is_empty());
                true
            },
            Node::Empty => false,
            Node::Intermediate { child_0, child_1 } => {
                let first = value[0];
                let rest = &value[1..];
                let child = if first { child_1 } else { child_0 };
                child.contains(rest)
            }
        }
    }

    fn insert(&mut self, value: &[bool]) {
        match self {
            Node::Leaf => {
                panic!("Collision!");
            },
            Node::Empty => {
                if value.is_empty() {
                    *self = Node::Leaf;
                } else {
                    *self = Node::Intermediate {
                        child_0: Box::new(Node::Empty),
                        child_1: Box::new(Node::Empty),
                    };
                    self.insert(value);
                }
            },
            Node::Intermediate { child_0, child_1 } => {
                let first = value[0];
                let rest = &value[1..];
                let mut child = if first { child_1 } else { child_0 };
                child.insert(rest);
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use mir::trie::Trie;

    #[test]
    fn insert() {
        let mut trie = Trie::new(4);
        trie.insert(&[false, true, false, true]);
        assert!(trie.contains(&[false, true, false, true]));
        assert!(!trie.contains(&[true, true, false, true]));
        assert!(!trie.contains(&[false, true, false, false]));
    }

    #[test]
    #[should_panic]
    fn insert_too_small() {
        let mut trie = Trie::new(4);
        trie.insert(&[false, true, true]);
    }

    #[test]
    #[should_panic]
    fn assert_too_large() {
        let mut trie = Trie::new(4);
        trie.insert(&[false, true, false, true, false]);
    }
}