use core::fmt::Debug;

#[derive(Debug)]
pub enum ReadError<E: Debug> {
    Uninitialized,
    OutOfRange { got: usize, max: usize },
    Other(E),
}

#[derive(Debug)]
pub enum WriteError<E: Debug> {
    NotEnoughBytes { got: usize, needed: usize },
    OutOfRange { got: usize, max: usize },
    Other(E),
}

/// We assume that a storage medium can offer reads at a `Word` level
/// granularity and writes at a _sector_ level granularity.
pub trait Storage {
    /// This is really the 'read' level granularity.
    type Word: FromBytes;

    /// This is really the 'write' level granularity.
    /// In units of words.
    // type SectorSize:

    const SECTOR_SIZE: usize;

    type ReadErr: Debug;
    type WriteErr: Debug;

    /// In units of sectors.
    fn capacity(&self) -> usize;
    fn capacity_in_words(&self) -> usize { self.capacity() * Self::SECTOR_SIZE }
    fn capacity_in_bytes(&self) -> usize { self.capacity_in_words() * <Self::Word as FromBytes>::NUM_BYTES }

    /// Implementations may return `ReadError::Uninitialized` for memory
    /// locations that have not been written to at their discretion.
    ///
    /// `addr` must be [0, `self.capacity_in_words()`) for this to succeed.
    #[inline] // <-- just documentation, doesn't actually do anything
    fn read_word(&self, addr: usize) -> Result<Self::Word, ReadError<Self::ReadErr>>;

    // TODO: add an API with generic_array that's a little bit better.

    /// `addr` must be [0, `self.capacity_in_words()`) for this to succeed.
    ///
    /// Returns a slice pointing to the remaining words that were not written.
    #[inline]
    fn write_sector(
        &mut self,
        addr: usize,
        words: &[Self::Word],
    ) -> Result<&[Self::Word], WriteError<Self::WriteErr>> {
        // if words.len() < Self::SECTOR_SIZE
        unimplemented!()
    }

    /// `addr` must be [0, `self.capacity_in_words()`) for this to succeed.
    ///
    /// Returns a slice pointing to the remaining bytes that were not written.
    #[inline] // <-- just documentation, doesn't actually do anything
    fn write_sector_with_bytes(
        &mut self,
        addr: usize,
        bytes: &[u8]
    ) -> Result<&[u8], WriteError<Self::WriteErr>>;
}
