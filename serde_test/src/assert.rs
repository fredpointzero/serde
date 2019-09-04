use serde::{Deserialize, Serialize};

use de::Deserializer;
use ser::Serializer;
use token::Token;

use std::fmt::Debug;

/// Runs both `assert_ser_tokens` and `assert_de_tokens`.
///
/// ```edition2018
/// # use serde::{Serialize, Deserialize};
/// # use serde_test::{assert_tokens, Token};
/// #
/// # fn main() {
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct S {
///     a: u8,
///     b: u8,
/// }
///
/// let s = S { a: 0, b: 0 };
/// assert_tokens(&s, &[
///     Token::Struct { name: "S", len: 2 },
///     Token::Str("a"),
///     Token::U8(0),
///     Token::Str("b"),
///     Token::U8(0),
///     Token::StructEnd,
/// ]);
/// # }
/// ```
pub fn assert_tokens<'de, T>(value: &T, tokens: &'de [Token])
where
    T: Serialize + Deserialize<'de> + PartialEq + Debug,
{
    assert_ser_tokens(value, tokens);
    assert_de_tokens(value, tokens);
}

/// Asserts that `value` serializes to the given `tokens`.
///
/// ```edition2018
/// # use serde::{Serialize, Deserialize};
/// # use serde_test::{assert_ser_tokens, Token};
/// #
/// # fn main() {
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct S {
///     a: u8,
///     b: u8,
/// }
///
/// let s = S { a: 0, b: 0 };
/// assert_ser_tokens(&s, &[
///     Token::Struct { name: "S", len: 2 },
///     Token::Str("a"),
///     Token::U8(0),
///     Token::Str("b"),
///     Token::U8(0),
///     Token::StructEnd,
/// ]);
/// # }
/// ```
pub fn assert_ser_tokens<T>(value: &T, tokens: &[Token])
where
    T: Serialize,
{
    let mut ser = Serializer::new(tokens);
    match value.serialize(&mut ser) {
        Ok(_) => {}
        Err(err) => panic!("value failed to serialize: {}", err),
    }

    if ser.remaining() > 0 {
        panic!("{} remaining tokens", ser.remaining());
    }
}

/// Asserts that `value` serializes to the given `tokens`, and then yields
/// `error`.
///
/// ```edition2018
/// use std::sync::{Arc, Mutex};
/// use std::thread;
///
/// use serde::Serialize;
/// use serde_test::{assert_ser_tokens_error, Token};
///
/// #[derive(Serialize)]
/// struct Example {
///     lock: Arc<Mutex<u32>>,
/// }
///
/// fn main() {
///     let example = Example { lock: Arc::new(Mutex::new(0)) };
///     let lock = example.lock.clone();
///
///     let _ = thread::spawn(move || {
///         // This thread will acquire the mutex first, unwrapping the result
///         // of `lock` because the lock has not been poisoned.
///         let _guard = lock.lock().unwrap();
///
///         // This panic while holding the lock (`_guard` is in scope) will
///         // poison the mutex.
///         panic!()
///     }).join();
///
///     let expected = &[
///         Token::Struct { name: "Example", len: 1 },
///         Token::Str("lock"),
///     ];
///     let error = "lock poison error while serializing";
///     assert_ser_tokens_error(&example, expected, error);
/// }
/// ```
pub fn assert_ser_tokens_error<T>(value: &T, tokens: &[Token], error: &str)
where
    T: Serialize,
{
    let mut ser = Serializer::new(tokens);
    match value.serialize(&mut ser) {
        Ok(_) => panic!("value serialized successfully"),
        Err(e) => assert_eq!(e, *error),
    }

    if ser.remaining() > 0 {
        panic!("{} remaining tokens", ser.remaining());
    }
}

/// Asserts that the given `tokens` deserialize into `value`.
///
/// ```edition2018
/// # use serde::{Serialize, Deserialize};
/// # use serde_test::{assert_de_tokens, Token};
/// #
/// # fn main() {
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct S {
///     a: u8,
///     b: u8,
/// }
///
/// let s = S { a: 0, b: 0 };
/// assert_de_tokens(&s, &[
///     Token::Struct { name: "S", len: 2 },
///     Token::Str("a"),
///     Token::U8(0),
///     Token::Str("b"),
///     Token::U8(0),
///     Token::StructEnd,
/// ]);
/// # }
/// ```
pub fn assert_de_tokens<'de, T>(value: &T, tokens: &'de [Token])
where
    T: Deserialize<'de> + PartialEq + Debug,
{
    let in_place = internal_assert_de_tokens(value, Deserializer::new(tokens));
    internal_assert_de_in_place_tokens(value, in_place, Deserializer::new(tokens));
}

/// Asserts that the given `tokens` deserialize into `value`, using a specific version map
///
/// Only tests deserialize and not deserialize in place
///
/// ```edition2018
/// # use serde::{Serialize, Deserialize};
/// # use serde_test::{assert_de_tokens, Token};
/// #
/// # fn main() {
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct S {
///     a: u8,
///     b: u8,
/// }
///
/// let s = S { a: 0, b: 0 };
/// assert_de_tokens_versions(&s, &[
///     Token::Struct { name: "S", len: 2 },
///     Token::Str("a"),
///     Token::U8(0),
///     Token::Str("b"),
///     Token::U8(0),
///     Token::StructEnd,
/// ],
///     None);
/// # }
/// ```
#[cfg(feature = "versioning")]
pub fn assert_de_tokens_versions<'de, T>(
    value: &T,
    tokens: &'de [Token],
    version_map: Option<&'de serde::de::VersionMap>,
) where
    T: Deserialize<'de> + PartialEq + Debug,
{
    internal_assert_de_tokens(value, Deserializer::with_versions(tokens, version_map));
}

fn internal_assert_de_tokens<'de, T>(value: &T, mut de: Deserializer<'de>) -> T
where
    T: Deserialize<'de> + PartialEq + Debug,
{
    let result = match T::deserialize(&mut de) {
        Ok(v) => {
            assert_eq!(v, *value);
            v
        }
        Err(e) => panic!("tokens failed to deserialize: {}", e),
    };
    if de.remaining() > 0 {
        panic!("{} remaining tokens", de.remaining());
    }

    result
}

fn internal_assert_de_in_place_tokens<'de, T>(value: &T, mut in_place: T, mut de: Deserializer<'de>)
where
    T: Deserialize<'de> + PartialEq + Debug,
{
    // Do the same thing for deserialize_in_place. This isn't *great* because a
    // no-op impl of deserialize_in_place can technically succeed here. Still,
    // this should catch a lot of junk.
    match T::deserialize_in_place(&mut de, &mut in_place) {
        Ok(()) => {
            assert_eq!(in_place, *value);
        }
        Err(e) => panic!("tokens failed to deserialize_in_place: {}", e),
    }
    if de.remaining() > 0 {
        panic!("{} remaining tokens", de.remaining());
    }
}

/// Asserts that the given `tokens` yield `error` when deserializing.
///
/// ```edition2018
/// # use serde::{Serialize, Deserialize};
/// # use serde_test::{assert_de_tokens_error, Token};
/// #
/// # fn main() {
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// #[serde(deny_unknown_fields)]
/// struct S {
///     a: u8,
///     b: u8,
/// }
///
/// assert_de_tokens_error::<S>(
///     &[
///         Token::Struct { name: "S", len: 2 },
///         Token::Str("x"),
///     ],
///     "unknown field `x`, expected `a` or `b`",
/// );
/// # }
/// ```
pub fn assert_de_tokens_error<'de, T>(tokens: &'de [Token], error: &str)
where
    T: Deserialize<'de>,
{
    let mut de = Deserializer::new(tokens);
    match T::deserialize(&mut de) {
        Ok(_) => panic!("tokens deserialized successfully"),
        Err(e) => assert_eq!(e, *error),
    }

    // There may be one token left if a peek caused the error
    de.next_token_opt();

    if de.remaining() > 0 {
        panic!("{} remaining tokens", de.remaining());
    }
}
