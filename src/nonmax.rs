use creusot_std::prelude::*;
use std::num::NonZeroU16;

#[derive(
    Debug,
    creusot_std::prelude::Clone,
    Copy,
    creusot_std::prelude::Default,
    creusot_std::prelude::PartialEq,
    Eq,
)]
pub struct NonMaxU16(u16);

impl TryFrom<u16> for NonMaxU16 {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value == u16::MAX {
            Err(())
        } else {
            Ok(NonMaxU16(value))
        }
    }
}

impl From<NonMaxU16> for u16 {
    fn from(value: NonMaxU16) -> Self {
        value.0
    }
}

impl std::fmt::Display for NonMaxU16 {
    #[trusted]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DeepModel for NonMaxU16 {
    type DeepModelTy = <u16 as DeepModel>::DeepModelTy;

    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        self.0.deep_model()
    }
}

#[derive(
    Debug,
    creusot_std::prelude::Clone,
    Copy,
    creusot_std::prelude::Default,
    creusot_std::prelude::PartialEq,
    Eq,
)]
pub struct NonMaxU32(u32);

impl TryFrom<u32> for NonMaxU32 {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == u32::MAX {
            Err(())
        } else {
            Ok(NonMaxU32(value))
        }
    }
}

impl From<NonMaxU32> for u32 {
    fn from(value: NonMaxU32) -> Self {
        value.0
    }
}

impl std::fmt::Display for NonMaxU32 {
    #[trusted]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DeepModel for NonMaxU32 {
    type DeepModelTy = <u32 as DeepModel>::DeepModelTy;

    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        self.0.deep_model()
    }
}
