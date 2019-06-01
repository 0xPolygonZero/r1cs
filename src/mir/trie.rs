use field_element::FieldElement;

#[derive(Debug)]
pub struct Trie {
    depth: usize,
    root: Node,
}

type CompressionFunction = fn(FieldElement, FieldElement) -> FieldElement;

impl Trie {
    pub fn new(depth: usize) -> Self {
        Trie { depth, root: Node::Empty }
    }

    pub fn merkle_root(&self, compress: CompressionFunction) -> FieldElement {
        self.root.hash(self.depth, compress)
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
    fn hash(&self, height: usize, compress: CompressionFunction) -> FieldElement {
        match self {
            Node::Leaf => FieldElement::one(),
            Node::Empty => Node::empty_hash(height, compress),
            Node::Intermediate { child_0, child_1 } => {
                compress(child_0.hash(height - 1, compress), child_1.hash(height - 1, compress))
            },
        }
    }

    fn empty_hash(height: usize, compress: CompressionFunction) -> FieldElement {
        if height == 0 {
            FieldElement::zero()
        } else {
            let child_hash = Node::empty_hash(height - 1, compress);
            compress(child_hash.clone(), child_hash)
        }
    }

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