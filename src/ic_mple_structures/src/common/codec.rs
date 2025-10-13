use std::borrow::Cow;

use ic_stable_structures::Storable;

/// A codec for a data type.
pub trait Codec<D>: Storable {
    /// Dencodes a `Source` value into a `Destination` value.
    fn decode(source: Self) -> D;

    /// Encodes a `Destination` value into a `Source` value.
    fn encode(dest: D) -> Self;
}

/// A codec for a data type.
pub trait RefCodec<D: Clone>: Storable {
    /// Dencodes a `Source` value into a `Destination` value.
    fn decode_ref<'a>(source: &'a Self) -> Cow<'a, D>;

    /// Encodes a `Destination` value into a `Source` value.
    fn encode(dest: D) -> Self;
}

impl<D: Storable> Codec<D> for D {
    fn decode(source: D) -> D {
        source
    }

    fn encode(dest: D) -> D {
        dest
    }
}

impl<D: Storable + Clone> RefCodec<D> for D {
    fn decode_ref<'a>(source: &'a D) -> Cow<'a, D> {
        Cow::Borrowed(source)
    }

    fn encode(dest: D) -> D {
        dest
    }
}
