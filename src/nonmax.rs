use creusot_std::prelude::*;

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

    #[requires(value@ != u16::MAX@)]
    #[ensures(match(result) { Ok(result) => value@ == result@, Err(_) => false })]
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value == u16::MAX {
            Err(())
        } else {
            Ok(NonMaxU16(value))
        }
    }
}

impl From<NonMaxU16> for u16 {
    #[ensures(result@ == value@)]
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

impl View for NonMaxU16 {
    type ViewTy = Int;

    #[logic]
    fn view(self) -> Int {
        self.0.view()
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

    #[requires(value@ != u32::MAX@)]
    #[ensures(match(result) { Ok(result) => value@ == result@, Err(_) => false })]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == u32::MAX {
            Err(())
        } else {
            Ok(NonMaxU32(value))
        }
    }
}

impl From<NonMaxU32> for u32 {
    #[ensures(result@ == value@)]
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

impl View for NonMaxU32 {
    type ViewTy = Int;

    #[logic]
    fn view(self) -> Int {
        self.0.view()
    }
}
