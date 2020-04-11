//! Home of the [`AsBytes`](AsBytes) trait.

// We really want const generics here so we can ask for `Self::NUM_BYTES` in
// methods, but alas.
/// Types that implement this can be constructed from a slice of [`u8`]s.
///
/// [`u8`]: u8
pub trait AsBytes: Sized {
    /// The number of bytes the implementing type needs to construct itself.
    const NUM_BYTES: usize = core::mem::size_of::<Self>();

    /// The array of bytes that the implementing type can be turned into.
    ///
    /// This type should always be an array like this: `[u8; Self::NUM_BYTES]`
    /// but unfortunately we can't actually write or use that type until const
    /// generics land, so we do this instead.
    ///
    /// We have no way to enforce that the number of bytes here matches the
    /// constant in this trait, so it's on implementors to not do bad things!
    type To: AsRef<[u8]> + AsMut<[u8]>;

    /// Constructs an instance of `Self` from a slice of bytes, returning the
    /// constructed instance (and the remaining bytes) on success.
    ///
    /// This function is _fallible_; if there aren't enough bytes or if anything
    /// goes wrong, this function will return [`None`](Option::None).
    /// Implementors should take care not to panic in this function.
    fn from(bytes: &[u8]) -> Option<(Self, &[u8])>;

    /// Go the other way; a type to its bytes. This is infallible.
    ///
    /// Implementors are allowed to pick between big and little endian but must
    /// be internally consistent. That is, `to` and `from` should  form an
    /// infallible roundtrip.
    ///
    /// As in, this should work:
    /// ```rust
    /// assert_eq!(AsBytes::from(AsBytes::to(234u64).as_ref()), Some((234u64, &[])));
    /// ```
    fn to(s: &Self) -> Self::To;
}

macro_rules! impl_from_bytes {
    ($($ty:ty)*) => {$(
        impl AsBytes for $ty {
            type To = [u8; core::mem::size_of::<Self>()];

            fn from(bytes: &[u8]) -> Option<(Self, &[u8])> {
                if bytes.len() < Self::NUM_BYTES {
                    None
                } else {
                    let (bytes, rest) = bytes.split_at(Self::NUM_BYTES);

                    // Lean on the const generics inside std:
                    use core::convert::TryFrom as TF;
                    let val = Self::from_le_bytes(TF::try_from(bytes).unwrap());

                    Some((val, rest))
                }
            }

            fn to(s: &Self) -> Self::To {
                s.to_le_bytes()
            }
        }
    )*};
}

impl_from_bytes! { u8 u16 u32 u64 u128 usize }


