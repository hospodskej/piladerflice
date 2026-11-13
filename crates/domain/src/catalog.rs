//! The fixed vocabulary of the catalogue. Stored as text in the database and
//! parsed into these enums at the edge, so the rest of the code can't hold an
//! unknown category.


macro_rules! text_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum $name { $($variant),+ }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            pub fn as_str(self) -> &'static str {
                match self { $($name::$variant => $text),+ }
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(&self, s: S) -> ::std::result::Result<S::Ok, S::Error> {
                s.serialize_str(self.as_str())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(d: D) -> ::std::result::Result<Self, D::Error> {
                let text = <::std::borrow::Cow<'de, str> as ::serde::Deserialize>::deserialize(d)?;
                text.parse().map_err(::serde::de::Error::custom)
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = $crate::catalog::UnknownValue;
            fn from_str(s: &str) -> ::std::result::Result<Self, $crate::catalog::UnknownValue> {
                match s {
                    $($text => Ok($name::$variant),)+
                    _ => Err($crate::catalog::UnknownValue { kind: stringify!($name), value: s.to_owned() }),
                }
            }
        }
    };
}
pub(crate) use text_enum;

/// A text that isn't one of the known values.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown {kind}: {value:?}")]
pub struct UnknownValue {
    pub kind: &'static str,
    pub value: String,
}

text_enum! {
    /// What the sawmill sells; each has its own VAT rule.
    Category {
        Firewood => "palivove",
        Lumber => "rezivo",
        Residues => "zbytky",
        Stone => "kamenivo",
    }
}

impl Category {
    /// The name used in the admin (Czech only).
    pub fn admin_label(self) -> &'static str {
        match self {
            Category::Firewood => "Palivové dřevo",
            Category::Lumber => "Stavební řezivo",
            Category::Residues => "Výrobní zbytky (štěpka, piliny, odkory, hobliny)",
            Category::Stone => "Okrasné kamenivo",
        }
    }
}

text_enum! {
    /// Which product page layout a catalogue product uses.
    Template {
        Firewood => "firewood",
        Lumber => "lumber",
        SimpleVariant => "simple_variant",
    }
}

text_enum! {
    /// Firewood hardness, for the e-shop filter.
    Hardness {
        Hard => "hard",
        Soft => "soft",
    }
}

text_enum! {
    /// How firewood is delivered.
    VariantGroup {
        Container => "kontejner",
        Crates => "bedny",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_text() {
        for c in Category::ALL {
            assert_eq!(c.as_str().parse::<Category>().unwrap(), *c);
        }
        assert!("nabytek".parse::<Category>().is_err());
        assert_eq!(serde_json::to_string(&Template::SimpleVariant).unwrap(), "\"simple_variant\"");
        assert_eq!(serde_json::to_string(&Category::Firewood).unwrap(), "\"palivove\"");
        assert_eq!(serde_json::from_str::<Category>("\"rezivo\"").unwrap(), Category::Lumber);
    }
}
