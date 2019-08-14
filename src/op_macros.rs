macro_rules! impl_uniop_val_to_ref {
    ($type:ty, $trait:ty, $method:ident) => {
        impl $trait for $type {
            fn $method(self) -> $type {
                (&self).$method()
            }
        }
    }
}

macro_rules! impl_binop_all {
    ($type:ty, $trait:ty, $method:ident) => {
        impl_binop_val_val_to_ref_ref!($type, $trait, $method);
        impl_binop_val_ref_to_ref_ref!($type, $trait, $method);
        impl_binop_ref_val_to_ref_ref!($type, $trait, $method);
    }
}

macro_rules! impl_binop_val_val_to_ref_ref {
    ($type:ty, $trait:ty, $method:ident) => {
        impl $trait for $type {
            type Output = $type;

            fn $method(self, rhs: $type) -> $type {
                (&self).$method(&rhs)
            }
        }
    };
}

macro_rules! impl_binop_val_ref_to_ref_ref {
    ($type:ty, $trait:ty, $method:ident) => {
        impl $trait for $type {
            type Output = $type;

            fn $method(self, rhs: &$type) -> $type {
                (&self).$method(rhs)
            }
        }
    };
}

macro_rules! impl_binop_ref_val_to_ref_ref {
    ($type:ty, $trait:ty, $method:ident) => {
        impl $trait for &$type {
            type Output = $type;

            fn $method(self, rhs: $type) -> $type {
                self.$method(&rhs)
            }
        }
    };
}

