use std::borrow::Cow;

/// A codec for a data type.
pub trait Codec<S, D: Clone> {
    /// Dencodes a `Source` value into a `Destination` value.
    fn decode(&self, source: S) -> D;

    /// Encodes a `Destination` value into a `Source` value.
    fn encode(&self, dest: D) -> S;
}

/// A codec for a data type.
pub trait RefCodec<S, D: Clone> {
    /// Dencodes a `Source` value into a `Destination` value.
    fn decode_ref<'a>(&self, source: &'a S) -> Cow<'a, D>;

    /// Encodes a `Destination` value into a `Source` value.
    fn encode(&self, dest: D) -> S;
}

/// Default NoOps codec.
pub struct DefaultCodec<D> {
    phantom: std::marker::PhantomData<D>,
}

impl<D> Default for DefaultCodec<D> {
    fn default() -> Self {
        Self {
            phantom: std::marker::PhantomData,
        }
    }
}

impl<D: Clone> Codec<D, D> for DefaultCodec<D> {
    fn decode(&self, source: D) -> D {
        source
    }

    fn encode(&self, dest: D) -> D {
        dest
    }
}

impl<D: Clone> RefCodec<D, D> for DefaultCodec<D> {
    fn decode_ref<'a>(&self, source: &'a D) -> Cow<'a, D> {
        Cow::Borrowed(source)
    }

    fn encode(&self, dest: D) -> D {
        dest
    }
}
