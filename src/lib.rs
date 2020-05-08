//! Some traits for various storage mediums.
//!
//! Useful for embedded applications.

#![forbid(
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused_allocation,
    unused_lifetimes,
    unused_comparisons,
    unused_parens,
    while_true
)]
#![deny(
    unused,
    bad_style,
    missing_debug_implementations,
    intra_doc_link_resolution_failure,
    // missing_docs, // TODO!
    unsafe_code,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    rust_2018_idioms
)]
#![doc(test(attr(deny(warnings))))]
#![doc(html_logo_url = "")] // TODO!

// Mark the crate as no_std if the `no_std` feature is enabled.
#![cfg_attr(feature = "no_std", no_std)]

macro_rules! using_std { ($($i:item)*) => ($(#[cfg(not(feature = "no_std"))]$i)*) }

#[allow(unused_extern_crates)]
extern crate core; // makes rls actually look into the standard library (hack)


mod bytes;
pub use bytes::*;

mod storage;
pub use storage::*;

mod extensions;
pub use extensions::*;

pub mod errors;

// TODO: move to its own file
using_std! {
    use std::convert::TryInto;
    use std::fs::{File, OpenOptions};
    use std::io::{Result as IoResult, ErrorKind, Error, Read, Write, Seek, SeekFrom};
    use std::marker::PhantomData;
    use std::path::Path;

    use generic_array::{ArrayLength, GenericArray};

    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    pub struct FileBackedStorage<
        Word = u8,
        SECTOR_SIZE = typenum::consts::U512,
    >
    where
        Word: AsBytes,
        SECTOR_SIZE: ArrayLength<Word>,
    {
        file: File,
        // This is a field so that we don't have to resort to interior
        // mutability to get the length out of a `File`. Also so we don't have
        // to deal with the possibility of the length of the `File` changing
        // underneath us and becoming not a multiple of the sector size.
        size_in_sectors: usize,
        _s: PhantomData<(Word, SECTOR_SIZE)>,
    }

    impl<W: AsBytes, S: ArrayLength<W>> FileBackedStorage<W, S> {
        // Fails if the file already exists.
        pub fn new<P: AsRef<Path>>(path: P, size_in_sectors: usize) -> IoResult<Self> {
            let mut opts = OpenOptions::new();

            let file = opts
                .read(true)
                .write(true)
                .create_new(true)
                .open(path)?;

            file.set_len(
                S::to_u64()
                    .checked_mul(size_in_sectors.try_into().unwrap())
                    .unwrap()
            )?;

            Ok(Self {
                file,
                size_in_sectors,
                _s: PhantomData,
            })
        }

        // Errors if the file does not have a size that's a multiple of the
        // sector size.
        pub fn from_file<P: AsRef<Path>>(path: P) -> IoResult<Self> {
            let mut opts = OpenOptions::new();

            let file = opts
                .read(true)
                .write(true)
                .open(path)?;

            let len: usize = file.metadata()?.len().try_into().unwrap();

            if let Some(0) = len.checked_rem(S::to_usize()) {
                Ok(Self {
                    file,
                    size_in_sectors: (len.checked_div(S::to_usize()).unwrap()),
                    _s: PhantomData,
                })
            } else {
                Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "File length ({}) is not a multiple of the sector size ({}).",
                        len,
                        S::to_usize(),
                    ),
                ))
            }
        }

        // Necessary when the provided files are weird (i.e. block files).
        pub fn from_file_with_explicit_size<P: AsRef<Path>>(path: P, size_in_sectors: usize) -> IoResult<Self> {
            let mut f = Self::from_file(path)?;
            f.size_in_sectors = size_in_sectors;

            Ok(f)
        }
    }

    impl<W: AsBytes, S: ArrayLength<W>> Storage for FileBackedStorage<W, S> {
        type Word = W;
        type SECTOR_SIZE = S;

        type ReadErr = Error;
        type WriteErr = Error;

        fn capacity(&self) -> usize {
            self.size_in_sectors
        }

        fn read_sector(
            &mut self,
            sector_idx: usize,
            buffer: &mut GenericArray<W, S>
        ) -> Result<(), errors::ReadError<Error>> {
            if sector_idx >= self.size_in_sectors {
                return Err(errors::ReadError::OutOfRange {
                    requested_offset: sector_idx,
                    max_offset: self.size_in_sectors
                });
            }

            // Move into place:
            let _ = self.file.seek(SeekFrom::Start(
                sector_idx.checked_mul(S::to_usize()).unwrap().try_into().unwrap()
            ))?;

            // Do the read.
            // TODO: do better than this; we should be able to find a way to
            // do this without the intermediate buffer. Probably using unsafe.
            let sector_size_in_bytes = S::to_usize() * W::NUM_BYTES;
            let mut buf: Vec<u8> = Vec::with_capacity(sector_size_in_bytes);
            buf.resize(sector_size_in_bytes, 0);

            assert_eq!(sector_size_in_bytes, self.file.read(&mut buf)?);

            // Copy into the actual buffer...
            let mut buf = buf.as_slice();
            for idx in 0..(S::to_usize()) {
                let (word, remaining) = AsBytes::from(buf).unwrap();

                buffer.as_mut_slice()[idx] = word;
                buf = remaining;
            }

            Ok(())
        }

        fn write_sector(
            &mut self,
            sector_idx: usize,
            words: &GenericArray<Self::Word, Self::SECTOR_SIZE>,
        ) -> Result<(), errors::WriteError<Self::WriteErr>> {
            if sector_idx >= self.size_in_sectors {
                return Err(errors::WriteError::OutOfRange {
                    requested_offset: sector_idx,
                    max_offset: self.size_in_sectors,
                });
            }

            // Move into place:
            let _ = self.file.seek(SeekFrom::Start(
                sector_idx.checked_mul(S::to_usize()).unwrap().try_into().unwrap()
            ))?;

            // Do the write.
            // TODO: do better than this; we should be able to find a way to
            // do this without the intermediate buffer. Probably using unsafe.
            let sector_size_in_bytes = S::to_usize() * W::NUM_BYTES;
            let mut buf: Vec<u8> = Vec::with_capacity(sector_size_in_bytes);

            for word in words.iter() {
                for bytes in word.to().as_ref().iter() {
                    buf.push(*bytes);
                }
            }

            assert_eq!(sector_size_in_bytes, buf.len());

            // Actually do the write:
            assert_eq!(sector_size_in_bytes, self.file.write(&buf)?);

            Ok(())
        }
    }

    // TODO!
    // impl<W: AsBytes, S: ArrayLength<W>> WordReadable for FileBackedStorage<W, S> {

    // }
}
