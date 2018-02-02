use bytes::Bytes;
use ip_addr::IpCidr;
use lex::{Lex, LexResult};
use lex::expect;

use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};
use std::net::IpAddr;

fn lex_rhs_values<'i, T: Lex<'i>>(input: &'i str) -> LexResult<Vec<T>> {
    let mut input = expect(input, "{")?.trim_left();
    let mut res = Vec::new();
    loop {
        let (item, rest) = T::lex(input)?;
        res.push(item);
        input = rest.trim_left();
        if let Ok(input) = expect(input, "}") {
            return Ok((res, input.trim_left()));
        }
    }
}

macro_rules! declare_types {
    (@enum $(# $attrs:tt)* $name:ident { $($(# $variant_attrs:tt)* $variant:ident ( $ty:ty ) , )* }) => {
        $(# $attrs)*
        pub enum $name<'a> {
            $($(# $variant_attrs)* $variant($ty),)*
        }

        impl<'a> GetType for $name<'a> {
            fn get_type(&self) -> Type {
                match *self {
                    $($name::$variant(_) => Type::$variant,)*
                }
            }
        }

        impl<'a> Debug for $name<'a> {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                match *self {
                    $($name::$variant(ref inner) => Debug::fmt(inner, f),)*
                }
            }
        }
    };

    ($($name:ident ( $lhs_ty:ty | $(# $rhs_attrs:tt)* $rhs_ty:ty ) , )*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        pub enum Type {
            $($name,)*
        }

        pub trait GetType {
            fn get_type(&self) -> Type;
        }

        impl GetType for Type {
            fn get_type(&self) -> Type {
                *self
            }
        }

        declare_types!(@enum #[repr(u8)] #[derive(Clone)] LhsValue {
            $($name($lhs_ty),)*
        });

        declare_types!(@enum #[derive(Serialize, Deserialize)] RhsValue {
            $($(# $rhs_attrs)* $name($rhs_ty),)*
        });

        declare_types!(@enum #[derive(Serialize, Deserialize)] RhsValues {
            $($(# $rhs_attrs)* $name(Vec<$rhs_ty>),)*
        });

        impl<'a> PartialOrd<RhsValue<'a>> for LhsValue<'a> {
            fn partial_cmp(&self, other: &RhsValue) -> Option<Ordering> {
                match (self, other) {
                    $((&LhsValue::$name(ref lhs), &RhsValue::$name(ref rhs)) => {
                        lhs.partial_cmp(rhs)
                    },)*
                    _ => None,
                }
            }
        }

        impl<'a> PartialEq<RhsValue<'a>> for LhsValue<'a> {
            fn eq(&self, other: &RhsValue) -> bool {
                match (self, other) {
                    $((&LhsValue::$name(ref lhs), &RhsValue::$name(ref rhs)) => lhs == rhs,)*
                    _ => false,
                }
            }
        }

        impl<'a> RhsValue<'a> {
            pub fn lex(input: &str, ty: Type) -> LexResult<Self> {
                Ok(match ty {
                    $(Type::$name => {
                        let (value, input) = <$rhs_ty>::lex(input)?;
                        (RhsValue::$name(value), input)
                    })*
                })
            }
        }

        impl<'a> RhsValues<'a> {
            pub fn lex(input: &str, ty: Type) -> LexResult<Self> {
                Ok(match ty {
                    $(Type::$name => {
                        let (value, input) = lex_rhs_values(input)?;
                        (RhsValues::$name(value), input)
                    })*
                })
            }
        }
    };
}

declare_types!(
    Ip(IpAddr | IpCidr),
    Bytes(Bytes<'a> | #[serde(borrow)] Bytes<'a>),
    Unsigned(u64 | u64),
);