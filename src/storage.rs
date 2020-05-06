//! Holds the core [`Storage`](Storage) trait.

use super::AsBytes;
use super::errors::{ReadError, WriteError};

use core::fmt::Debug;

use generic_array::{GenericArray, ArrayLength};
use typenum::marker_traits::Unsigned;

/// The core [`Storage`] trait. Offers sector based writes and word based reads.
///
/// ## Assumptions
///
/// In designing a storage trait, we went for a (reasonable) lowest common
/// denominator which we think (perhaps incorrectly so) is the interface
/// provided by Flash memory. In other words, we expect that implementors of
/// this trait are using storage mediums that can offer reads at a `Word` level
/// granularity and writes at a _sector_ level granularity.
///
/// ## Extensions
///
/// We realize that other some storage mediums can do _more_ than this; for
/// example EEPROM frequently allows for `Word` level writes in addition to
/// reads.
///
/// For such storage mediums and use cases, we offer the _[extensions]_, traits
/// that extend the core [`Storage`] trait with additional functionality.
///
/// [`Storage`]: Storage
/// [extensions]: super::extensions
pub trait Storage {
    /// This is really the 'read' level granularity.
    type Word: AsBytes;

    /// This is really the 'write' level granularity.
    ///
    /// This is in units of words; we expect that implementations will offer
    /// sector sizes that are a multiple of their word size.
    ///
    /// This is an associated type instead of an associated constant because
    /// we're still waiting on real const generics; this is required to be a
    /// [typenum](typenum) [unsigned number type][unsigned]. See [`ArrayLength`]
    /// for more information.
    ///
    /// [`ArrayLength`]: generic_array::ArrayLength
    /// [unsigned]: typenum::marker_traits::Unsigned
    #[allow(non_camel_case_types)]
    type SECTOR_SIZE: ArrayLength<Self::Word>;

    /// Extra errors specific to this implementation that can occur when
    /// reading data.
    type ReadErr: Debug;

    /// Extra errors specific to this implementation that can occur when
    /// writing data.
    type WriteErr: Debug;

    /// In units of sectors.
    fn capacity(&self) -> usize;

    /// In units of words.
    fn capacity_in_words(&self) -> usize {
        self.capacity() * Self::SECTOR_SIZE::to_usize()
    }

    /// In units of bytes.
    fn capacity_in_bytes(&self) -> usize {
        self.capacity_in_words() * <Self::Word as AsBytes>::NUM_BYTES
    }


    /// Implementations may return `ReadError::Uninitialized` for memory
    /// locations that have not been written to at their discretion.
    ///
    /// `offset` must be in [0, `self.capacity_in_words()`) for this to succeed.
    fn read_word(&self, word_offset: usize)
        -> Result<Self::Word, ReadError<Self::ReadErr>>;

    /// Reads in some chunk of data. There is no guarantee that the requested
    /// chunk is aligned to a sector or smaller than a sector.
    ///
    /// `offset` must be in [0, `self.capacity_in_words()`) for this to succeed.
    ///
    /// This function should never panic but can return errors for the
    /// appropriate cases (i.e. out of range).
    ///
    /// Implementors should try to leave `buffer` unaltered when errors happen
    /// wherever possible (i.e. check that `offset` is in range _before_
    /// starting to modify `buffer`).
    ///
    /// This function has a naïve default implementation; implementors that can
    /// provide a more performant way to read in more than a word at a time
    /// should override this.
    #[inline]
    fn read_words(
        &mut self,
        word_offset: usize,
        buffer: &mut [Self::Word],
    ) -> Result<(), ReadError<Self::ReadErr>> {
        let max_offset = word_offset + (buffer.len() - 1);
        if  max_offset >= self.capacity_in_words() {
            return Err(ReadError::OutOfRange {
                requested_offset: max_offset,
                max_offset: self.capacity_in_words(),
            });
        }

        for (idx, word) in buffer.iter_mut().enumerate() {
            *word = self.read_word(word_offset + idx)?;
        }

        Ok(())
    }

    /// Reads in an entire sector.
    ///
    /// This has a default implementation that just calls `read_words`;
    /// implementations that are able to do better for their specific medium
    /// should provide their own (better) implementation.
    ///
    /// Alternatively, if an implementation detects when calls to `read_words`
    /// can take advantage of entire sector reads, there is no need to override
    /// this function; the default implementation will also benefit from this.
    #[inline]
    fn read_sector(
        &mut self,
        sector_idx: usize,
        buffer: &mut GenericArray<Self::Word, Self::SECTOR_SIZE>,
    ) -> Result<(), ReadError<Self::ReadErr>> {
        self.read_words(
            sector_idx * Self::SECTOR_SIZE::to_usize(),
            buffer.as_mut_slice()
        )
    }

    /// Writes out an entire sector.
    ///
    /// `sector_idx` must be in [0, `self.capacity()`) for this to succeed.
    ///
    /// This function should never panic but can return errors for the
    /// appropriate cases (i.e. out of range).
    ///
    /// Implementors should try to leave the actual storage unaltered when
    /// errors happen wherever possible (i.e. check that `sector_idx` is in
    /// range _before_ starting to modify anything; strive to be _atomic_).
    ///
    /// This function has a naïve default implementation; implementors that can
    /// provide a more performant way to read in more than a word at a time
    /// should override this.
    fn write_sector(
        &mut self,
        sector_idx: usize,
        words: &GenericArray<Self::Word, Self::SECTOR_SIZE>,
    ) -> Result<(), WriteError<Self::WriteErr>>;

    // TODO!
    // provide a default impl that does error checking and then calls
    // write_sector_with_words
    // /// `sector_idx` must be [0, `self.capacity()`) for this to succeed.
    // ///
    // /// Returns a slice pointing to the remaining bytes that were not written.
    // fn write_sector_with_bytes(
    //     &mut self,
    //     sector_idx: usize,
    //     bytes: &[u8]
    // ) -> Result<&[u8], WriteError<Self::WriteErr>>;

    // TODO!
    // have write_sector have a default impl that calls this.
    // this should do error checking and what not (i.e. add the standard docs)
    // fn write_sector_with_words(
    //     &mut self,
    //     sector_idx: usize,
    //     sectors: &[Self::Word]
    // ) -> Result<(), WriteError<Self::WriteErr>>;

    // TODO!
    // type EraseErr;
    // turn Eraseable into a marker trait, move its function over to here.
    // provide a default impl that just calls write_sector on an array of
    // zeros for all sectors
    // add a note cautioning that the default impl may allocate lots of
    // stack space (1 whole sector's worth).
}
