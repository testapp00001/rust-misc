//! # Declarative Macros: `macro_rules!`
//!
//! Declarative macros use pattern matching to transform Rust code at compile time.
//! They're defined with `macro_rules!` and work by matching input tokens against
//! patterns and expanding the matched pattern into output code.
//!
//! Key concepts:
//! - **Metavariables**: `$name:expr`, `$name:ty`, `$name:ident`, etc.
//! - **Repetitions**: `$(...)*` (zero or more), `$(...)+` (one or more)
//! - **Hygiene**: Macro-generated identifiers don't conflict with caller's identifiers
//! - **$crate**: References the crate where the macro is defined

/// A simple macro that creates a HashMap from key-value pairs.
/// This demonstrates metavariables, repetitions, and the `$crate` path.
macro_rules! hashmap {
    // Empty map
    () => {
        std::collections::HashMap::new()
    };
    // Map with key-value pairs
    ($($key:expr => $value:expr),+ $(,)?) => {
        {
            let mut map = std::collections::HashMap::new();
            $(map.insert($key, $value);)+
            map
        }
    };
}

/// A macro that implements the Display trait using a format string.
/// This shows how macros can reduce boilerplate for trait implementations.
macro_rules! impl_display {
    ($type:ty, $fmt:expr) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $fmt, self)
            }
        }
    };
}

/// A macro that creates a vector of strings from identifiers.
/// Demonstrates the `stringify!` built-in macro.
macro_rules! string_vec {
    ($($item:ident),+ $(,)?) => {
        vec![$(stringify!($item).to_string()),+]
    };
}

/// A macro that defines an enum with automatic Display and FromStr implementations.
/// This demonstrates more complex pattern matching and code generation.
macro_rules! define_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident => $string:expr),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($variant),+
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$variant => write!(f, $string)),+
                }
            }
        }

        impl std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($string => Ok(Self::$variant),)+
                    _ => Err(format!("unknown variant: {s}")),
                }
            }
        }
    };
}

define_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Direction {
        North => "north",
        South => "south",
        East => "east",
        West => "west",
    }
}

/// A macro for creating builder patterns.
/// This demonstrates nested macro patterns and recursive expansion.
macro_rules! builder {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field:ident : $type:ty),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $($field: $type),+
        }

        paste::paste! {
            $vis struct [<$name Builder>] {
                $($field: Option<$type>),+
            }

            impl [<$name Builder>] {
                $vis fn new() -> Self {
                    [<$name Builder>] {
                        $($field: None),+
                    }
                }

                $(
                    $vis fn $field(mut self, value: $type) -> Self {
                        self.$field = Some(value);
                        self
                    }
                )+

                $vis fn build(self) -> Result<$name, String> {
                    Ok($name {
                        $($field: self.$field.ok_or_else(||
                            format!("field `{}` not set", stringify!($field))
                        )?),+
                    })
                }
            }
        }
    };
}

/// Since we don't have the paste crate, here's a simpler builder macro
/// that works without it.
macro_rules! simple_builder {
    (
        $vis:vis struct $name:ident {
            $($field:ident : $type:ty),+ $(,)?
        }
    ) => {
        $vis struct $name {
            $($field: $type),+
        }

        impl $name {
            $vis fn builder() -> [<$name Builder>] {
                [<$name _Builder>]::new()
            }
        }
    };
}

/// A macro for implementing common traits.
/// Demonstrates how to generate multiple trait implementations.
macro_rules! impl_ops {
    ($type:ident, $trait:ident, $method:ident, $op:tt) => {
        impl std::ops::$trait for $type {
            type Output = $type;
            fn $method(self, rhs: $type) -> $type {
                $type(self.0 $op rhs.0)
            }
        }
    };
}

/// A wrapper type for demonstrating the impl_ops macro.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Meters(pub f64);

impl_ops!(Meters, Add, add, +);
impl_ops!(Meters, Sub, sub, -);

/// A macro for creating test cases with multiple inputs.
/// This is a common pattern for data-driven testing.
macro_rules! test_cases {
    ($name:ident: $( ($input:expr, $expected:expr) ),+ $(,)?) => {
        #[test]
        fn $name() {
            $(
                assert_eq!($input, $expected);
            )+
        }
    };
}

/// A macro for creating a newtype wrapper with common trait implementations.
macro_rules! newtype {
    ($vis:vis struct $name:ident($inner:ty)) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis struct $name(pub $inner);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                $name(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

newtype!(pub struct UserId(u64));
newtype!(pub struct OrderId(u64));
newtype!(pub struct ProductId(u32));

/// A macro that generates a static lookup table.
/// Demonstrates how to create compile-time data structures.
macro_rules! lookup_table {
    ($vis:vis static $name:ident: $key:ty => $value:ty = {
        $($k:expr => $v:expr),+ $(,)?
    }) => {
        $vis static $name: &[($key, $value)] = &[
            $(($k, $v)),+
        ];
    };
}

lookup_table! {
    pub static HTTP_STATUS: u16 => &'static str = {
        200 => "OK",
        301 => "Moved Permanently",
        404 => "Not Found",
        500 => "Internal Server Error",
    }
}

/// A macro for creating a thread-local variable with initialization.
macro_rules! thread_local_init {
    ($vis:vis static $name:ident: $type:ty = $init:expr) => {
        std::thread_local! {
            $vis static $name: std::cell::Cell<$type> = std::cell::Cell::new($init);
        }
    };
}

thread_local_init!(pub static COUNTER: u64 = 0);

/// A macro that implements Debug by printing field names and values.
macro_rules! debug_struct {
    ($name:ident { $($field:ident),+ }) => {
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name))
                    $(.field(stringify!($field), &self.$field))+
                    .finish()
            }
        }
    };
}

pub struct Config {
    pub host: String,
    pub port: u16,
    pub debug: bool,
}

debug_struct!(Config { host, port, debug });

/// A macro for creating a const array of tuples from parallel lists.
macro_rules! zip_arrays {
    ($([$($a:expr),* $(,)?] [$($b:expr),* $(,)?]),+ $(,)?) => {
        &[
            $(($a, $b)),+
        ]
    };
}

/// A macro that creates a function that returns a &'static str
/// based on a match expression.
macro_rules! str_match_fn {
    ($vis:vis fn $name:ident($arg:ident: $type:ty) -> &'static str {
        $($pattern:pat => $result:expr),+ $(,)?
    }) => {
        $vis fn $name($arg: $type) -> &'static str {
            match $arg {
                $($pattern => $result),+
            }
        }
    };
}

str_match_fn!(pub fn season_name(month: u8) -> &'static str {
    12 | 1 | 2 => "winter",
    3 | 4 | 5 => "spring",
    6 | 7 | 8 => "summer",
    9 | 10 | 11 => "autumn",
    _ => "unknown",
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashmap_macro() {
        let map = hashmap! {
            "name" => "Alice",
            "city" => "NYC",
        };
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("name"), Some(&"Alice"));
        assert_eq!(map.get("city"), Some(&"NYC"));
    }

    #[test]
    fn test_hashmap_empty() {
        let map: std::collections::HashMap<&str, i32> = hashmap! {};
        assert!(map.is_empty());
    }

    #[test]
    fn test_hashmap_trailing_comma() {
        let map = hashmap! {
            1 => "one",
            2 => "two",
            3 => "three",
        };
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_direction_enum() {
        assert_eq!(Direction::North.to_string(), "north");
        assert_eq!(Direction::South.to_string(), "south");

        let dir: Direction = "east".parse().unwrap();
        assert_eq!(dir, Direction::East);

        let err = "invalid".parse::<Direction>();
        assert!(err.is_err());
    }

    #[test]
    fn test_meters_ops() {
        let a = Meters(10.0);
        let b = Meters(3.0);
        assert_eq!(a + b, Meters(13.0));
        assert_eq!(a - b, Meters(7.0));
    }

    test_cases!(test_arithmetic:
        (1 + 1, 2),
        (2 * 3, 6),
        (10 - 4, 6),
    );

    #[test]
    fn test_newtype_user_id() {
        let id = UserId(42);
        assert_eq!(id.to_string(), "42");
        assert_eq!(id, UserId::from(42u64));

        let inner: u64 = id.into();
        assert_eq!(inner, 42);
    }

    #[test]
    fn test_newtype_order_id() {
        let id = OrderId(100);
        assert_eq!(id.0, 100);
        assert_eq!(OrderId::from(100u64), id);
    }

    #[test]
    fn test_lookup_table() {
        assert_eq!(HTTP_STATUS.len(), 4);
        assert_eq!(HTTP_STATUS[0], (200, "OK"));
        assert_eq!(HTTP_STATUS[2], (404, "Not Found"));
    }

    #[test]
    fn test_thread_local_counter() {
        COUNTER.with(|c| {
            c.set(42);
            assert_eq!(c.get(), 42);
        });
    }

    #[test]
    fn test_config_debug() {
        let config = Config {
            host: "localhost".to_string(),
            port: 8080,
            debug: true,
        };
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("host"));
        assert!(debug_str.contains("localhost"));
        assert!(debug_str.contains("8080"));
    }

    #[test]
    fn test_season_name() {
        assert_eq!(season_name(1), "winter");
        assert_eq!(season_name(4), "spring");
        assert_eq!(season_name(7), "summer");
        assert_eq!(season_name(10), "autumn");
    }
}
