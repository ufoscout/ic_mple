use std::borrow::Cow;

use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{Storable, storable::Bound};

use crate::common::codec::Codec;

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
        Decode!(bytes.as_ref(), UserV2).unwrap()
    }
}

pub struct UserCodec;

impl Codec<VersionedUser, UserV2> for UserCodec {

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