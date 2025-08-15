use std::fmt;
use std::ops::BitAnd;
use std::ops::BitAndAssign;
use std::ops::BitOr;
use std::ops::BitOrAssign;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Not;

use serde::Deserializer;
use serde::de;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;

#[derive(Default, Debug, PartialEq, PartialOrd, Eq, Hash, Clone, Copy)]
pub struct Mask(pub u128);

impl Serialize for Mask {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Mask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MaskVisitor;

        impl<'de> de::Visitor<'de> for MaskVisitor {
            type Value = Mask;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an unsigned 128-bit integer that can be used as a Mask")
            }

            fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Mask(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse::<u128>().map(Mask).map_err(E::custom)
            }
        }

        deserializer.deserialize_any(MaskVisitor)
    }
}

impl Deref for Mask {
    type Target = u128;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Mask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u128> for Mask {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl BitOr for Mask {
    type Output = Mask;

    fn bitor(self, rhs: Self) -> Self::Output {
        Mask(self.0 | rhs.0)
    }
}

impl BitOrAssign for Mask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Not for Mask {
    type Output = Self;

    fn not(self) -> Self::Output {
        Mask(!self.0)
    }
}

impl BitAnd for Mask {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Mask(self.0 & rhs.0)
    }
}

impl BitAndAssign for Mask {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}
