use std::borrow::Cow;

use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};

use crate::common::codec::{Codec, RefCodec};

#[derive(Clone, CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum VersionedUser {
    V1(UserV1),
    V2(UserV2),
}

impl Storable for VersionedUser {

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

pub struct UserCodec;

impl Codec<VersionedUser, UserV2> for UserCodec {

    fn decode(&self, source: VersionedUser) -> UserV2 {
        match source {
            VersionedUser::V1(user_v1) => UserV2 {
                name: user_v1.0,
                age: None
            },
            VersionedUser::V2(user_v2) => user_v2,
        }
    }

    fn encode(&self, dest: UserV2) -> VersionedUser {
        VersionedUser::V2(dest)
    }
}

impl RefCodec<VersionedUser, UserV2> for UserCodec {

    fn decode_ref<'a>(&self, source: &'a VersionedUser) -> std::borrow::Cow<'a, UserV2> {
        match source {
            VersionedUser::V1(user_v1) => Cow::Owned(UserV2 {
                name: user_v1.0.clone(),
                age: None
            }),
            VersionedUser::V2(user_v2) => Cow::Borrowed(user_v2),
        }
    }

    fn encode(&self, dest: UserV2) -> VersionedUser {
        VersionedUser::V2(dest)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, CandidType, Deserialize)]
pub struct StringValue(pub String);

impl Storable for StringValue {
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

pub fn str_val(len: usize) -> StringValue {
    let mut s = String::with_capacity(len);
    s.extend((0..len).map(|_| 'Q'));
    StringValue(s)
}

/// New type pattern used to implement `Storable` trait for all arrays.
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct Array<const N: usize>(pub [u8; N]);

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
