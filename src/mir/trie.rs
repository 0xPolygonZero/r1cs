use field_element::FieldElement;

/// A Merkle trie for storing a set of binary values. Each instance has a fixed height, `bits`; all
/// inserted values must have that exact bit length.
#[derive(Debug)]
pub struct Trie {
    bits: usize,
    root: Node,
}

type CompressionFunction = fn(FieldElement, FieldElement) -> FieldElement;

impl Trie {
    pub fn new(bits: usize) -> Self {
        Trie { bits, root: Node::Empty }
    }

    /// Merkle roots are computed in the following way. A leaf is assigned a value 1 if its position
    /// in the tree (i.e., its pattern of left and right branches) corresponds to a member of the
    /// set, otherwise it is assigned 0. A non-leaf node is assigned compress(left, right), where
    /// left and right correspond to the node's children.
    ///
    /// If a node is empty (i.e., the set contains no values prefixed with the node's position), it
    /// is assigned a value as if it had two empty nodes as children, even though no such children
    /// are stored in memory. This simplifies certain authenticated operations. For example, to
    /// prove that a set S does not contain a value x, we can prove inclusion of a leaf node whose
    /// position is x and whose value is zero, even though such a node is not stored in memory.
    pub fn merkle_root(&self, compress: CompressionFunction) -> FieldElement {
        self.root.hash(self.bits, compress)
    }

    pub fn contains(&self, value: &[bool]) -> bool {
        assert_eq!(value.len(), self.bits);
        self.root.contains(value)
    }

    pub fn insert(&mut self, value: &[bool]) {
        assert_eq!(value.len(), self.bits);
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
    fn hash(&self, bits_left: usize, compress: CompressionFunction) -> FieldElement {
        match self {
            Node::Leaf => FieldElement::one(),
            Node::Empty => Node::empty_hash(bits_left, compress),
            Node::Intermediate { child_0, child_1 } => {
                compress(child_0.hash(bits_left - 1, compress), child_1.hash(bits_left - 1, compress))
            },
        }
    }

    fn empty_hash(bits_left: usize, compress: CompressionFunction) -> FieldElement {
        if bits_left == 0 {
            FieldElement::zero()
        } else {
            let child_hash = Node::empty_hash(bits_left - 1, compress);
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
    use field_element::FieldElement;

    #[test]
    fn one_bit() {
        let mut trie = Trie::new(1);
        assert!(!trie.contains(&[false]));
        assert!(!trie.contains(&[true]));
        // The root should be hash(0, 0) = 4.
        assert_eq!(FieldElement::from(4), trie.merkle_root(test_compress));

        trie.insert(&[false]);
        assert!(trie.contains(&[false]));
        assert!(!trie.contains(&[true]));
        // The root should be hash(1, 0) = 5.
        assert_eq!(FieldElement::from(5), trie.merkle_root(test_compress));

        trie.insert(&[true]);
        assert!(trie.contains(&[false]));
        assert!(trie.contains(&[true]));
        // The root should be hash(1, 1) = 7.
        assert_eq!(FieldElement::from(7), trie.merkle_root(test_compress));
    }

    #[test]
    fn three_bits() {
        let mut trie = Trie::new(3);
        // hash(0, 0) = 4; hash(4, 4) = 16; hash(16, 16) = 52.
        assert_eq!(FieldElement::from(52), trie.merkle_root(test_compress));

        trie.insert(&[false, true, true]);
        assert!(trie.contains(&[false, true, true]));
        assert!(!trie.contains(&[true, true, false]));
        assert!(!trie.contains(&[false, false, false]));
        assert!(!trie.contains(&[false, true, false]));

        // Leaf is 1; first parent is hash(0, 1) = 6; next parent is hash(4, 6) = 20; root is
        // hash(20, 16) = 56.
        assert_eq!(FieldElement::from(56), trie.merkle_root(test_compress));
    }

    #[test]
    #[should_panic]
    fn insert_too_small() {
        let mut trie = Trie::new(4);
        trie.insert(&[false, true, true]);
    }

    #[test]
    #[should_panic]
    fn insert_too_large() {
        let mut trie = Trie::new(4);
        trie.insert(&[false, true, false, true, false]);
    }

    // A dummy compression function which returns x + (y + 1)*2 + 2.
    fn test_compress(x: FieldElement, y: FieldElement) -> FieldElement {
        x + (y + 1.into()) * 2u128 + 2.into()
    }
}