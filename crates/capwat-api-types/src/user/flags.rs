use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct UserFlags: u64 {
        const ADMINISTRATOR = 1 << 1;
    }
}

crate::should_impl_primitive_traits!(UserFlags);

impl UserFlags {
    #[must_use]
    pub const fn is_admin(&self) -> bool {
        self.contains(Self::ADMINISTRATOR)
    }
}

impl<'de> serde::de::Deserialize<'de> for UserFlags {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = UserFlags;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("Capwat JWT UserFlagss")
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(UserFlags::from_bits_truncate(v))
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse::<u64>()
                    .map_err(serde::de::Error::custom)
                    .and_then(|v| self.visit_u64(v))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

impl serde::Serialize for UserFlags {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}
