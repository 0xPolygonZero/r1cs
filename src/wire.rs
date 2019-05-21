#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Wire {
    pub index: u32,
}

impl Wire {
    pub const ONE: Wire = Wire { index: 0 };
}