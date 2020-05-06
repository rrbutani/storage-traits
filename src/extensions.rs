
use super::{Storage, errors::EraseError};

use core::fmt::Debug;

// TODO!

/// For storage mediums that can reliably provide word level writes (like
/// EEPROM, for example).
pub trait WordWritable: Storage {
    // #[inline] // <-- just documentation, doesn't actually do anything
    fn write_word(&mut self, addr: usize, word: Self::Word) -> Result<(), ()>;
}

// TODO!

// pub struct ErasedPageToken<F: Flash + ?Sized> {
//     /// The index of the sector that this token is for.
//     sector_idx: usize,
//     /// A record of the sectors in this page.
//     record: [u8; <F as Storage>::SECTOR_SIZE],
//     _f: core::marker::PhantomData<F>,
// }

// TODO!

/// An extension to the `Storage` trait that allows for complicated access
/// schemes that wish to write to erased pages gradually. Tracking of these
/// pages is done at runtime.
///
/// Attempts to be less unsafe (i.e. there's no danger of writing to words that
/// have yet to be cleared or have already been written to since being cleared;
/// you are prevented from doing this).
///
/// This does, however, assume that there are not multiple instances of the
/// type this is implemented on *with overlapping address ranges*.
pub trait Flash: Storage {
    // // Errors if `sector_idx` is not in [0, `self.capacity()`).
    // fn erase_sector(&mut self, sector_idx: usize) -> Result<ErasedPageToken<Self>, ()>;

    // /// `addr` must be [0, `self.capacity_in_words()`) for this to succeed.
    // #[allow(unsafe_code)]
    // unsafe fn write_word(&mut self, addr: usize, word: Self::Word) -> Result<(), ()>;

}


/// Should only be implemented by types that have a fast way to erase
/// themselves.
pub trait Eraseable: Storage {
    type EraseErr: Debug;

    /// Erases the entirety of the storage medium/partition/section that this
    /// instance corresponds to.
    fn erase(&mut self) -> Result<(), EraseError<<Self as Storage>::WriteErr, Self::EraseErr>>;
}
