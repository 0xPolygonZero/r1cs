use itertools::Itertools;

use crate::{Element, Expression, Field, GadgetBuilder, WireValues};

/// A symmetric-key block cipher.
pub trait BlockCipher<F: Field> {
    /// Encrypt the given input using the given key.
    fn encrypt(&self, builder: &mut GadgetBuilder<F>, key: &Expression<F>, input: &Expression<F>)
               -> Expression<F>;

    /// Decrypt the given output using the given key.
    fn decrypt(&self, builder: &mut GadgetBuilder<F>, key: &Expression<F>, output: &Expression<F>)
               -> Expression<F>;

    /// Like `encrypt`, but actually evaluates the encryption function rather than just adding it
    /// to a `GadgetBuilder`.
    fn encrypt_evaluate(&self, key: &Element<F>, input: &Element<F>) -> Element<F> {
        let mut builder = GadgetBuilder::new();
        let encrypted = self.encrypt(
            &mut builder, &Expression::from(key), &Expression::from(input));
        let mut values = WireValues::new();
        builder.build().execute(&mut values);
        encrypted.evaluate(&values)
    }

    /// Like `decrypt`, but actually evaluates the decryption function rather than just adding it
    /// to a `GadgetBuilder`.
    fn decrypt_evaluate(&self, key: &Element<F>, output: &Element<F>) -> Element<F> {
        let mut builder = GadgetBuilder::new();
        let decrypted = self.decrypt(
            &mut builder, &Expression::from(key), &Expression::from(output));
        let mut values = WireValues::new();
        builder.build().execute(&mut values);
        decrypted.evaluate(&values)
    }
}

/// A function which compresses two field elements into one, and is intended to be one-way.
pub trait CompressionFunction<F: Field> {
    /// Compress two field elements into one.
    fn compress(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>, y: &Expression<F>)
                -> Expression<F>;

    /// Like `compress`, but actually evaluates the compression function rather than just adding it
    /// to a `GadgetBuilder`.
    fn compress_evaluate(&self, x: &Element<F>, y: &Element<F>) -> Element<F> {
        let mut builder = GadgetBuilder::new();
        let compressed = self.compress(&mut builder, &Expression::from(x), &Expression::from(y));
        let mut values = WireValues::new();
        builder.build().execute(&mut values);
        compressed.evaluate(&values)
    }
}

/// A permutation of single field elements.
pub trait Permutation<F: Field> {
    /// Permute the given field element.
    fn permute(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>) -> Expression<F>;

    /// Like `permute`, but actually evaluates the permutation rather than just adding it to a
    /// `GadgetBuilder`.
    fn permute_evaluate(&self, x: &Element<F>) -> Element<F> {
        let mut builder = GadgetBuilder::new();
        let permuted = self.permute(&mut builder, &Expression::from(x));
        let mut values = WireValues::new();
        builder.build().execute(&mut values);
        permuted.evaluate(&values)
    }
}

/// A permutation of multiple field elements.
pub trait MultiPermutation<F: Field> {
    // TODO figure out a good interface. Const generics would be nice...
}

/// A function which hashes a sequence of field elements, outputting a single field element.
pub trait HashFunction<F: Field> {
    fn hash(&self, builder: &mut GadgetBuilder<F>, blocks: &[Expression<F>]) -> Expression<F>;

    /// Like `hash`, but actually evaluates the hash function rather than just adding it to a
    /// `GadgetBuilder`.
    fn hash_evaluate(&self, blocks: &[Element<F>]) -> Element<F> {
        let mut builder = GadgetBuilder::new();
        let block_expressions = blocks.iter().map(Expression::from).collect_vec();
        let hash = self.hash(&mut builder, &block_expressions);
        let mut values = WireValues::new();
        builder.build().execute(&mut values);
        hash.evaluate(&values)
    }
}