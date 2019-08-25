use crate::gadget_builder::GadgetBuilder;
use crate::field::Field;
use crate::expression::Expression;

/// A symmetric-key block cipher.
pub trait BlockCipher<F: Field> {
    fn encrypt(&self, builder: &mut GadgetBuilder<F>, key: &Expression<F>, input: &Expression<F>)
               -> Expression<F>;

    fn decrypt(&self, builder: &mut GadgetBuilder<F>, key: &Expression<F>, output: &Expression<F>)
               -> Expression<F>;
}

/// A function which compresses two field elements into one, and is intended to be one-way.
pub trait CompressionFunction<F: Field> {
    fn compress(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>, y: &Expression<F>)
                -> Expression<F>;
}

/// A permutation of single field elements.
pub trait Permutation<F: Field> {
    fn permute(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>) -> Expression<F>;
}

/// A permutation of multiple field elements.
pub trait MultiPermutation<F: Field> {
    // TODO figure out a good interface. Const generics would be nice...
}

/// A function which hashes a sequence of field elements, outputting a single field element.
pub trait HashFunction<F: Field> {
    fn hash(&self, builder: &mut GadgetBuilder<F>, blocks: &[Expression<F>]) -> Expression<F>;
}