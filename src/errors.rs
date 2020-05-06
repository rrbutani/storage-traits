//! Errors related to using storage mediums.

use core::fmt::Debug;

// #[derive(Debug)]
// pub enum ReadError<E: Debug> {
//     Uninitialized,
//     OutOfRange { got: usize, max: usize },
//     Other(E),
// }

// #[derive(Debug)]
// pub enum WriteError<E: Debug> {
//     NotEnoughBytes { got: usize, needed: usize },
//     OutOfRange { got: usize, max: usize },
//     Other(E),
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum WriteError<E: Debug> {
    /// For calls to `write_bytes` or `write_sector` that fall outside of the
    /// partition's space. The requested_offset (/ the sector size) must be
    /// greater than `Storage::SECTOR_SIZE` (i.e. out of range).
    OutOfRange { requested_offset: usize, max_offset: usize },

    // TODO
    InvalidNumberOfBytes { bytes_given: usize, bytes_in_a_sector: usize },
    // TODO
    InvalidNumberOfWords { words_given: usize, words_in_a_sector: usize },

    Other(E),
}

impl<E: Debug> From<E> for WriteError<E> {
    fn from(other: E) -> Self {
        WriteError::Other(other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// A non-exhaustive list of errors that can occur when reading data from a
/// storage medium.
///
/// Like [`WriteError`](WriteError), this has a catch-all
/// [`Other`](ReadError::Other) variant that can be used to represent
/// implementation specific errors that we didn't catch here.
///
/// Additionally, we've marked this enum as `#[non_exhaustive]` since it's
/// probable that there are some common errors that would be good to add to this
/// that we've missed.
///
/// The units of `offset` depend on the context (as in, who is returning the
/// error), but usually offset will be in units of words.
pub enum ReadError<E: Debug> {
    /// For when requested data has not been written to before.
    ///
    /// Implementations can choose to simply return 0s instead of returning this
    /// error in such cases.
    Uninitialized { offset: usize },
    /// For when calls to a read function fall outside of the medium's space.
    /// The `requested_offset` must be greater than the storage's capacity (i.e.
    /// out of range).
    OutOfRange { requested_offset: usize, max_offset: usize },
    /// Catch-all variant for implementation specific errors.
    Other(E),
}

impl<E: Debug> From<E> for ReadError<E> {
    fn from(other: E) -> Self {
        ReadError::Other(other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EraseError<W: Debug, E: Debug> {
    ErrorInIndividualErase(WriteError<W>),
    Other(E),
}

impl<W: Debug, E: Debug> From<E> for EraseError<W, E> {
    fn from(other: E) -> Self {
        EraseError::Other(other)
    }
}

using_std! {
    use std::fmt;

    macro_rules! display_using_debug {
        ($ty:ty) => { impl<T: fmt::Debug> Display for $ty<T> {
            fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                Debug::fmt(fmt)
            }
        }};
    }

    macro_rules! err {
        ($ty:ty) => {
            display_using_debug!($ty);

            impl<T: Debug> std::error::Error for $ty<T> { }
        };
    }

    err!(WriteError);
    err!(ReadError);
    err!(EraseError);
}
