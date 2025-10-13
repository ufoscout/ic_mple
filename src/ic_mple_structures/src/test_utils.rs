use std::borrow::Cow;

use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{Storable, storable::Bound};

use crate::common::{Bounded, Codec, RefCodec};

#[derive(Clone, CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum UserCodec {
    V1(UserV1),
    V2(UserV2),
}

impl Storable for UserCodec {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Encode!(&self).unwrap().into()
    }

    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(&bytes, Self).unwrap()
    }
}

#[derive(Clone, CandidType, Deserialize, PartialEq, Eq, Debug)]
pub struct UserV1(pub String);

impl Storable for UserV1 {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        Encode!(&self).unwrap().into()
    }

    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

#[derive(Clone, CandidType, Deserialize, PartialEq, Eq, Debug)]
pub struct UserV2 {
    pub name: String,
    pub age: Option<u8>,
}

impl Storable for UserV2 {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        Encode!(&self).unwrap().into()
    }

    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Codec<UserV2> for UserCodec {
    fn decode(source: UserCodec) -> UserV2 {
        match source {
            UserCodec::V1(user_v1) => UserV2 {
                name: user_v1.0,
                age: None,
            },
            UserCodec::V2(user_v2) => user_v2,
        }
    }

    fn encode(dest: UserV2) -> UserCodec {
        UserCodec::V2(dest)
    }
}

impl RefCodec<UserV2> for UserCodec {
    fn decode_ref<'a>(source: &'a UserCodec) -> std::borrow::Cow<'a, UserV2> {
        match source {
            UserCodec::V1(user_v1) => Cow::Owned(UserV2 {
                name: user_v1.0.clone(),
                age: None,
            }),
            UserCodec::V2(user_v2) => Cow::Borrowed(user_v2),
        }
    }

    fn encode(dest: UserV2) -> UserCodec {
        UserCodec::V2(dest)
    }
}

/// New type pattern used to implement `Storable` trait for all arrays.
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct Array<const N: usize>(pub [u8; N]);

/// Implement `Bounded` for all arrays.
impl<const N: usize> Bounded for Array<N> {
    const MIN: Array<N> = Array([0; N]);
    const MAX: Array<N> = Array([u8::MAX; N]);
}

impl<const N: usize> Storable for Array<N> {
    const BOUND: Bound = Bound::Bounded {
        max_size: N as u32,
        is_fixed_size: true,
    };

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(self.0.to_vec())
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        let mut buf = [0u8; N];
        buf.copy_from_slice(&bytes);
        Array(buf)
    }

    fn into_bytes(self) -> Vec<u8> {
        self.0.to_vec()
    }
}
