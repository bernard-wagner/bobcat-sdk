use bobcat_maths::{I, U};

use bobcat_storage::{Keccak256, keccak256_builder};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

#[cfg(not(feature = "std"))]
mod no_std {
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Error {
        WriteAllEof,
        ReadExactEof,
        InvalidData,
    }

    pub trait Write {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Error>;
        fn flush(&mut self) -> Result<(), Error>;
        fn is_empty(&self) -> bool;

        fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Error> {
            while !buf.is_empty() {
                match self.write(buf) {
                    Ok(0) => return Err(Error::WriteAllEof),
                    Ok(n) if n <= buf.len() => buf = &buf[n..],
                    Ok(_) => return Err(Error::InvalidData),
                    Err(error) => return Err(error),
                }
            }
            Ok(())
        }
    }

    pub trait Read {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>;

        fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), Error> {
            while !buf.is_empty() {
                match self.read(buf) {
                    Ok(0) => break,
                    Ok(n) if n <= buf.len() => buf = &mut buf[n..],
                    Ok(_) => return Err(Error::InvalidData),
                    Err(error) => return Err(error),
                }
            }
            if buf.is_empty() {
                Ok(())
            } else {
                Err(Error::ReadExactEof)
            }
        }
    }

    impl Write for &mut [u8] {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
            let len = core::cmp::min(self.len(), buf.len());
            let target = core::mem::take(self);
            let (written, remaining) = target.split_at_mut(len);
            written.copy_from_slice(&buf[..len]);
            *self = remaining;
            Ok(len)
        }

        fn flush(&mut self) -> Result<(), Error> {
            Ok(())
        }

        fn is_empty(&self) -> bool {
            <[u8]>::is_empty(self)
        }
    }

    #[cfg(feature = "alloc")]
    impl Write for Vec<u8> {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
            self.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<(), Error> {
            Ok(())
        }

        fn is_empty(&self) -> bool {
            Vec::is_empty(self)
        }
    }

    #[cfg(feature = "alloc")]
    impl Write for &mut Vec<u8> {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
            (**self).write(buf)
        }

        fn flush(&mut self) -> Result<(), Error> {
            (**self).flush()
        }

        fn is_empty(&self) -> bool {
            Vec::is_empty(self)
        }
    }

    impl Read for &[u8] {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
            let len = core::cmp::min(self.len(), buf.len());
            buf[..len].copy_from_slice(&self[..len]);
            *self = &self[len..];
            Ok(len)
        }
    }

    impl Read for &mut [u8] {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
            let len = core::cmp::min(self.len(), buf.len());
            let source = core::mem::take(self);
            let (read, remaining) = source.split_at_mut(len);
            buf[..len].copy_from_slice(read);
            *self = remaining;
            Ok(len)
        }
    }
}

#[cfg(not(feature = "std"))]
pub use no_std::{Error, Read, Write};

#[cfg(feature = "std")]
pub use std::io::{Error, Read, Write};

#[doc(hidden)]
#[derive(Clone)]
pub struct SelectorHasher(Keccak256);

impl SelectorHasher {
    pub fn new() -> Self {
        Self(keccak256_builder())
    }

    pub fn update(self, bytes: &[u8]) -> Self {
        Self(self.0.update(bytes))
    }

    pub fn update_usize(self, mut value: usize) -> Self {
        let mut digits = [0u8; 20];
        let mut start = digits.len();
        loop {
            start -= 1;
            digits[start] = b'0' + (value % 10) as u8;
            value /= 10;
            if value == 0 {
                break;
            }
        }
        self.update(&digits[start..])
    }

    pub fn selector(&self) -> [u8; 4] {
        let hash = self.0.finalize();
        [hash[0], hash[1], hash[2], hash[3]]
    }
}

impl Default for SelectorHasher {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EvmCdWriteTarget {
    type Writer<'a>: Write
    where
        Self: 'a;

    fn writer(&mut self) -> Self::Writer<'_>;
}

impl EvmCdWriteTarget for [u8] {
    type Writer<'a> = &'a mut [u8];

    fn writer(&mut self) -> Self::Writer<'_> {
        self
    }
}

impl EvmCdWriteTarget for &mut [u8] {
    type Writer<'a>
        = &'a mut [u8]
    where
        Self: 'a;

    fn writer(&mut self) -> Self::Writer<'_> {
        self
    }
}

impl<const N: usize> EvmCdWriteTarget for [u8; N] {
    type Writer<'a> = &'a mut [u8];

    fn writer(&mut self) -> Self::Writer<'_> {
        self.as_mut_slice()
    }
}

#[cfg(feature = "alloc")]
impl EvmCdWriteTarget for Vec<u8> {
    type Writer<'a> = &'a mut Vec<u8>;

    fn writer(&mut self) -> Self::Writer<'_> {
        self
    }
}

pub trait EvmCdSerialise {
    fn serialise<W: EvmCdWriteTarget + ?Sized>(&self, writer: &mut W) -> Result<(), Error> {
        self.serialise_writer(&mut writer.writer())
    }

    fn serialise_to_array<const N: usize>(&self) -> Result<[u8; N], Error> {
        let mut output = [0; N];
        let mut writer = output.as_mut_slice();
        self.serialise_writer(&mut writer)?;
        if !writer.is_empty() {
            return Err(invalid_data());
        }
        Ok(output)
    }

    #[cfg(feature = "alloc")]
    fn serialise_to_vec(&self) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();
        self.serialise(&mut output)?;
        Ok(output)
    }

    #[doc(hidden)]
    fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error>;

    #[doc(hidden)]
    fn serialise_value<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.serialise_writer(writer)
    }

    #[doc(hidden)]
    fn is_abi_dynamic() -> bool {
        false
    }

    #[doc(hidden)]
    fn abi_head_size() -> usize {
        32
    }

    #[doc(hidden)]
    fn abi_tail_size(&self) -> usize {
        0
    }

    #[doc(hidden)]
    fn serialise_abi_head<W: Write>(
        &self,
        _tail_offset: usize,
        writer: &mut W,
    ) -> Result<(), Error> {
        self.serialise_value(writer)
    }

    #[doc(hidden)]
    fn serialise_abi_tail<W: Write>(&self, _writer: &mut W) -> Result<(), Error> {
        Ok(())
    }

    #[doc(hidden)]
    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher;
}

#[doc(hidden)]
pub enum EvmCdHead<T> {
    Value(T),
    Offset(usize),
}

#[doc(hidden)]
pub struct EvmCdStaticBufferKind;

#[cfg(feature = "alloc")]
#[doc(hidden)]
pub struct EvmCdDynamicBufferKind;

#[doc(hidden)]
pub trait EvmCdBufferKind {
    type Buffer<S>: EvmCdDecodeBuffer;
    type Combined<Rhs: EvmCdBufferKind>: EvmCdBufferKind;
}

impl EvmCdBufferKind for EvmCdStaticBufferKind {
    type Buffer<S> = EvmCdBuffer<S>;
    type Combined<Rhs: EvmCdBufferKind> = Rhs;
}

#[cfg(feature = "alloc")]
impl EvmCdBufferKind for EvmCdDynamicBufferKind {
    type Buffer<S> = EvmCdDynamicBuffer<S>;
    type Combined<Rhs: EvmCdBufferKind> = EvmCdDynamicBufferKind;
}

#[doc(hidden)]
pub trait EvmCdDecodeBuffer: AsRef<[u8]> + AsMut<[u8]> + Sized {
    type Kind: EvmCdBufferKind;

    fn new(len: usize) -> Result<Self, Error>;
}

#[doc(hidden)]
pub struct EvmCdBuffer<S> {
    storage: core::mem::MaybeUninit<S>,
}

impl<S> EvmCdDecodeBuffer for EvmCdBuffer<S> {
    type Kind = EvmCdStaticBufferKind;

    fn new(len: usize) -> Result<Self, Error> {
        if len > size_of::<S>() {
            return Err(invalid_data());
        }
        let mut storage = core::mem::MaybeUninit::<S>::uninit();
        // SAFETY: the storage remains wrapped in MaybeUninit and is only exposed as bytes.
        // Zeroing every byte makes the complete u8 slice valid without constructing an S.
        unsafe {
            storage
                .as_mut_ptr()
                .cast::<u8>()
                .write_bytes(0, size_of::<S>())
        };
        Ok(Self { storage })
    }
}

impl<S> AsRef<[u8]> for EvmCdBuffer<S> {
    fn as_ref(&self) -> &[u8] {
        // SAFETY: new() zero-initializes every byte before constructing Self.
        unsafe { core::slice::from_raw_parts(self.storage.as_ptr().cast::<u8>(), size_of::<S>()) }
    }
}

impl<S> AsMut<[u8]> for EvmCdBuffer<S> {
    fn as_mut(&mut self) -> &mut [u8] {
        // SAFETY: new() zero-initializes every byte before constructing Self.
        unsafe {
            core::slice::from_raw_parts_mut(self.storage.as_mut_ptr().cast::<u8>(), size_of::<S>())
        }
    }
}

#[cfg(feature = "alloc")]
#[doc(hidden)]
pub struct EvmCdDynamicBuffer<S>(pub Vec<u8>, pub core::marker::PhantomData<S>);

#[cfg(feature = "alloc")]
impl<S> EvmCdDecodeBuffer for EvmCdDynamicBuffer<S> {
    type Kind = EvmCdDynamicBufferKind;

    fn new(len: usize) -> Result<Self, Error> {
        if len > MAX_ALLOC_DESERIALISE_LEN {
            return Err(invalid_data());
        }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(len).map_err(|_| invalid_data())?;
        bytes.resize(len, 0);
        Ok(Self(bytes, core::marker::PhantomData))
    }
}

#[cfg(feature = "alloc")]
impl<S> AsRef<[u8]> for EvmCdDynamicBuffer<S> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(feature = "alloc")]
impl<S> AsMut<[u8]> for EvmCdDynamicBuffer<S> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

pub trait EvmCdDeserialise: Sized {
    type Buffer: EvmCdDecodeBuffer;

    fn new_buffer(len: usize) -> Result<Self::Buffer, Error> {
        Self::Buffer::new(len)
    }

    fn deserialise<B>(bytes: &B) -> Result<Self, Error>
    where
        B: AsRef<[u8]> + ?Sized,
    {
        let mut reader = bytes.as_ref();
        Self::deserialise_reader(&mut reader)
    }

    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error>;

    #[doc(hidden)]
    fn deserialise_value<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Self::deserialise_reader(reader)
    }

    #[doc(hidden)]
    fn is_abi_dynamic() -> bool {
        false
    }

    #[doc(hidden)]
    fn abi_head_size() -> usize {
        32
    }

    #[doc(hidden)]
    fn abi_tail_size(&self) -> usize {
        0
    }

    #[doc(hidden)]
    fn deserialise_abi_head<R: Read>(reader: &mut R) -> Result<EvmCdHead<Self>, Error> {
        Ok(EvmCdHead::Value(Self::deserialise_value(reader)?))
    }

    #[doc(hidden)]
    fn deserialise_abi_finish<R: Read>(
        head: EvmCdHead<Self>,
        _expected_tail_offset: usize,
        _reader: &mut R,
    ) -> Result<Self, Error> {
        match head {
            EvmCdHead::Value(value) => Ok(value),
            EvmCdHead::Offset(_) => Err(invalid_data()),
        }
    }

    #[doc(hidden)]
    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher;
}

#[cfg(not(feature = "std"))]
pub fn invalid_data() -> Error {
    Error::InvalidData
}

#[cfg(feature = "std")]
pub fn invalid_data() -> Error {
    Error::new(std::io::ErrorKind::InvalidData, "invalid EVM calldata")
}

fn read_usize_word<R: Read>(reader: &mut R) -> Result<usize, Error> {
    let mut word = [0u8; 32];
    reader.read_exact(&mut word)?;
    if word[..32 - size_of::<usize>()]
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err(invalid_data());
    }
    Ok(usize::from_be_bytes(
        word[32 - size_of::<usize>()..].try_into().unwrap(),
    ))
}

fn write_dynamic_tail<W: Write>(bytes: &[u8], writer: &mut W) -> Result<(), Error> {
    writer.write_all(&U::from_usize(bytes.len()).0)?;
    writer.write_all(bytes)?;
    const ZEROES: [u8; 31] = [0; 31];
    let padding = (32 - bytes.len() % 32) % 32;
    writer.write_all(&ZEROES[..padding])
}

fn write_dynamic_bytes<W: Write>(bytes: &[u8], writer: &mut W) -> Result<(), Error> {
    writer.write_all(&U::from_u32(32).0)?;
    write_dynamic_tail(bytes, writer)
}

fn dynamic_tail_size(len: usize) -> usize {
    32 + len + (32 - len % 32) % 32
}

fn read_dynamic_tail<const CAP: usize, R: Read>(
    reader: &mut R,
) -> Result<([u8; CAP], usize), Error> {
    let len = read_usize_word(reader)?;
    if len > CAP {
        return Err(invalid_data());
    }
    let mut bytes = [0u8; CAP];
    reader.read_exact(&mut bytes[..len])?;
    let padding = (32 - len % 32) % 32;
    let mut padding_bytes = [0u8; 31];
    reader.read_exact(&mut padding_bytes[..padding])?;
    if padding_bytes[..padding].iter().any(|byte| *byte != 0) {
        return Err(invalid_data());
    }
    Ok((bytes, len))
}

fn read_dynamic_bytes<const CAP: usize, R: Read>(
    reader: &mut R,
) -> Result<([u8; CAP], usize), Error> {
    if read_usize_word(reader)? != 32 {
        return Err(invalid_data());
    }
    read_dynamic_tail(reader)
}

macro_rules! fixed_deserialise_buffer {
    ($storage:ty) => {
        type Buffer = EvmCdBuffer<$storage>;
    };
}

impl EvmCdSerialise for U {
    fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.0)
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"uint256")
    }
}

impl EvmCdDeserialise for U {
    fixed_deserialise_buffer!([u8; 32]);

    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        Ok(U(buf))
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"uint256")
    }
}

impl EvmCdSerialise for bool {
    fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        u8::from(*self).serialise_value(writer)
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"bool")
    }
}

impl EvmCdDeserialise for bool {
    fixed_deserialise_buffer!([u8; 32]);

    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        match u8::deserialise_reader(reader)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid_data()),
        }
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"bool")
    }
}

macro_rules! for_ints {
    ($($ty:ty => $abi:literal),+ $(,)?) => {
        $(
            impl EvmCdSerialise for $ty {
                fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
                    writer.write_all(&[0u8; 32 - size_of::<$ty>()])?;
                    writer.write_all(&self.to_be_bytes())
                }

                fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
                    hasher.update($abi)
                }
            }

            impl EvmCdDeserialise for $ty {
                fixed_deserialise_buffer!([u8; 32]);

                fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
                    let U(word) = U::deserialise_reader(reader)?;
                    if word[..32 - size_of::<$ty>()].iter().any(|byte| *byte != 0) {
                        return Err(invalid_data());
                    }
                    Ok(<$ty>::from_be_bytes(
                        word[32 - size_of::<$ty>()..].try_into().unwrap(),
                    ))
                }

                fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
                    hasher.update($abi)
                }
            }
        )+
    };
}

for_ints! {
    u8 => b"uint8",
    u16 => b"uint16",
    u32 => b"uint32",
    u64 => b"uint64",
    u128 => b"uint128",
}

macro_rules! for_signed_ints {
    ($($ty:ty => $abi:literal),+ $(,)?) => {
        $(
            impl EvmCdSerialise for $ty {
                fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
                    let extension = if self.is_negative() { 0xff } else { 0 };
                    writer.write_all(&[extension; 32 - size_of::<$ty>()])?;
                    writer.write_all(&self.to_be_bytes())
                }

                fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
                    hasher.update($abi)
                }
            }

            impl EvmCdDeserialise for $ty {
                fixed_deserialise_buffer!([u8; 32]);

                fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
                    let mut word = [0u8; 32];
                    reader.read_exact(&mut word)?;
                    let value_start = 32 - size_of::<$ty>();
                    let extension = if word[value_start] & 0x80 == 0 { 0 } else { 0xff };
                    if word[..value_start].iter().any(|byte| *byte != extension) {
                        return Err(invalid_data());
                    }
                    Ok(<$ty>::from_be_bytes(word[value_start..].try_into().unwrap()))
                }

                fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
                    hasher.update($abi)
                }
            }
        )+
    };
}

for_signed_ints! {
    i8 => b"int8",
    i16 => b"int16",
    i32 => b"int32",
    i64 => b"int64",
    i128 => b"int128",
}

impl EvmCdSerialise for usize {
    fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        <u32 as EvmCdSerialise>::serialise_value(
            &u32::try_from(*self).map_err(|_| invalid_data())?,
            writer,
        )
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"uint32")
    }
}

impl EvmCdDeserialise for usize {
    fixed_deserialise_buffer!([u8; 32]);

    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(u32::deserialise_reader(reader)? as usize)
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"uint32")
    }
}

macro_rules! evm_cd_uint {
    ($name:ident, $bits:literal, $bytes:literal) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub [u8; $bytes]);

        impl $name {
            pub const fn new(bytes: [u8; $bytes]) -> Self {
                Self(bytes)
            }

            pub const fn into_array(self) -> [u8; $bytes] {
                self.0
            }

            pub const fn as_array(&self) -> &[u8; $bytes] {
                &self.0
            }
        }

        impl From<[u8; $bytes]> for $name {
            fn from(bytes: [u8; $bytes]) -> Self {
                Self::new(bytes)
            }
        }

        impl From<$name> for [u8; $bytes] {
            fn from(value: $name) -> Self {
                value.into_array()
            }
        }

        impl From<$name> for U {
            fn from(value: $name) -> Self {
                let mut word = [0u8; 32];
                word[32 - $bytes..].copy_from_slice(value.as_array());
                Self::from(word)
            }
        }

        impl From<U> for $name {
            fn from(value: U) -> Self {
                let word: [u8; 32] = value.into();
                Self::new(word[32 - $bytes..].try_into().unwrap())
            }
        }

        impl AsRef<[u8; $bytes]> for $name {
            fn as_ref(&self) -> &[u8; $bytes] {
                self.as_array()
            }
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                self.as_array()
            }
        }

        impl EvmCdSerialise for $name {
            fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
                writer.write_all(&[0; 32 - $bytes])?;
                writer.write_all(&self.0)
            }

            fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
                hasher.update(b"uint").update_usize($bits)
            }
        }

        impl EvmCdDeserialise for $name {
            fixed_deserialise_buffer!([u8; 32]);

            fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
                let mut word = [0u8; 32];
                reader.read_exact(&mut word)?;
                if word[..32 - $bytes].iter().any(|byte| *byte != 0) {
                    return Err(invalid_data());
                }
                Ok(Self(word[32 - $bytes..].try_into().unwrap()))
            }

            fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
                hasher.update(b"uint").update_usize($bits)
            }
        }
    };
}

macro_rules! evm_cd_int {
    ($name:ident, $bits:literal, $bytes:literal) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
        pub struct $name(pub [u8; $bytes]);

        impl $name {
            pub const fn new(bytes: [u8; $bytes]) -> Self {
                Self(bytes)
            }

            pub const fn into_array(self) -> [u8; $bytes] {
                self.0
            }

            pub const fn as_array(&self) -> &[u8; $bytes] {
                &self.0
            }
        }

        impl From<[u8; $bytes]> for $name {
            fn from(bytes: [u8; $bytes]) -> Self {
                Self::new(bytes)
            }
        }

        impl From<$name> for [u8; $bytes] {
            fn from(value: $name) -> Self {
                value.into_array()
            }
        }

        impl From<$name> for I {
            fn from(value: $name) -> Self {
                let extension = if value.0[0] & 0x80 == 0 { 0 } else { 0xff };
                let mut word = [extension; 32];
                word[32 - $bytes..].copy_from_slice(value.as_array());
                Self::from(word)
            }
        }

        impl From<I> for $name {
            fn from(value: I) -> Self {
                let word: [u8; 32] = value.into();
                Self::new(word[32 - $bytes..].try_into().unwrap())
            }
        }

        impl AsRef<[u8; $bytes]> for $name {
            fn as_ref(&self) -> &[u8; $bytes] {
                self.as_array()
            }
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                self.as_array()
            }
        }

        impl EvmCdSerialise for $name {
            fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
                let extension = if self.0[0] & 0x80 == 0 { 0 } else { 0xff };
                writer.write_all(&[extension; 32 - $bytes])?;
                writer.write_all(&self.0)
            }

            fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
                hasher.update(b"int").update_usize($bits)
            }
        }

        impl EvmCdDeserialise for $name {
            fixed_deserialise_buffer!([u8; 32]);

            fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
                let mut word = [0u8; 32];
                reader.read_exact(&mut word)?;
                let extension = if word[32 - $bytes] & 0x80 == 0 {
                    0
                } else {
                    0xff
                };
                if word[..32 - $bytes].iter().any(|byte| *byte != extension) {
                    return Err(invalid_data());
                }
                Ok(Self(word[32 - $bytes..].try_into().unwrap()))
            }

            fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
                hasher.update(b"int").update_usize($bits)
            }
        }
    };
}

macro_rules! evm_cd_integer_range {
    ($uint_macro:ident, $int_macro:ident) => {
        $uint_macro!(EvmCdU24, 24, 3);
        $int_macro!(EvmCdI24, 24, 3);
        $uint_macro!(EvmCdU40, 40, 5);
        $int_macro!(EvmCdI40, 40, 5);
        $uint_macro!(EvmCdU48, 48, 6);
        $int_macro!(EvmCdI48, 48, 6);
        $uint_macro!(EvmCdU56, 56, 7);
        $int_macro!(EvmCdI56, 56, 7);
        $uint_macro!(EvmCdU72, 72, 9);
        $int_macro!(EvmCdI72, 72, 9);
        $uint_macro!(EvmCdU80, 80, 10);
        $int_macro!(EvmCdI80, 80, 10);
        $uint_macro!(EvmCdU88, 88, 11);
        $int_macro!(EvmCdI88, 88, 11);
        $uint_macro!(EvmCdU96, 96, 12);
        $int_macro!(EvmCdI96, 96, 12);
        $uint_macro!(EvmCdU104, 104, 13);
        $int_macro!(EvmCdI104, 104, 13);
        $uint_macro!(EvmCdU112, 112, 14);
        $int_macro!(EvmCdI112, 112, 14);
        $uint_macro!(EvmCdU120, 120, 15);
        $int_macro!(EvmCdI120, 120, 15);
        $uint_macro!(EvmCdU136, 136, 17);
        $int_macro!(EvmCdI136, 136, 17);
        $uint_macro!(EvmCdU144, 144, 18);
        $int_macro!(EvmCdI144, 144, 18);
        $uint_macro!(EvmCdU152, 152, 19);
        $int_macro!(EvmCdI152, 152, 19);
        $uint_macro!(EvmCdU160, 160, 20);
        $int_macro!(EvmCdI160, 160, 20);
        $uint_macro!(EvmCdU168, 168, 21);
        $int_macro!(EvmCdI168, 168, 21);
        $uint_macro!(EvmCdU176, 176, 22);
        $int_macro!(EvmCdI176, 176, 22);
        $uint_macro!(EvmCdU184, 184, 23);
        $int_macro!(EvmCdI184, 184, 23);
        $uint_macro!(EvmCdU192, 192, 24);
        $int_macro!(EvmCdI192, 192, 24);
        $uint_macro!(EvmCdU200, 200, 25);
        $int_macro!(EvmCdI200, 200, 25);
        $uint_macro!(EvmCdU208, 208, 26);
        $int_macro!(EvmCdI208, 208, 26);
        $uint_macro!(EvmCdU216, 216, 27);
        $int_macro!(EvmCdI216, 216, 27);
        $uint_macro!(EvmCdU224, 224, 28);
        $int_macro!(EvmCdI224, 224, 28);
        $uint_macro!(EvmCdU232, 232, 29);
        $int_macro!(EvmCdI232, 232, 29);
        $uint_macro!(EvmCdU240, 240, 30);
        $int_macro!(EvmCdI240, 240, 30);
        $uint_macro!(EvmCdU248, 248, 31);
        $int_macro!(EvmCdI248, 248, 31);
        $uint_macro!(EvmCdU256, 256, 32);
        $int_macro!(EvmCdI256, 256, 32);
    };
}

evm_cd_integer_range!(evm_cd_uint, evm_cd_int);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EvmCdAddress(pub [u8; 20]);

impl EvmCdAddress {
    pub const fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    pub const fn into_array(self) -> [u8; 20] {
        self.0
    }

    pub const fn as_array(&self) -> &[u8; 20] {
        &self.0
    }
}

impl From<[u8; 20]> for EvmCdAddress {
    fn from(bytes: [u8; 20]) -> Self {
        Self::new(bytes)
    }
}

impl From<EvmCdAddress> for [u8; 20] {
    fn from(address: EvmCdAddress) -> Self {
        address.into_array()
    }
}

impl AsRef<[u8; 20]> for EvmCdAddress {
    fn as_ref(&self) -> &[u8; 20] {
        self.as_array()
    }
}

impl AsRef<[u8]> for EvmCdAddress {
    fn as_ref(&self) -> &[u8] {
        self.as_array()
    }
}

impl EvmCdSerialise for EvmCdAddress {
    fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&[0; 12])?;
        writer.write_all(&self.0)
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"address")
    }
}

impl EvmCdDeserialise for EvmCdAddress {
    fixed_deserialise_buffer!([u8; 32]);

    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut word = [0u8; 32];
        reader.read_exact(&mut word)?;
        if word[..12].iter().any(|byte| *byte != 0) {
            return Err(invalid_data());
        }
        Ok(Self(word[12..].try_into().unwrap()))
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"address")
    }
}

impl<const N: usize> EvmCdSerialise for [u8; N] {
    fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        if N == 0 || N > 32 {
            return Err(invalid_data());
        }
        writer.write_all(self)?;
        const ZEROES: [u8; 32] = [0; 32];
        writer.write_all(&ZEROES[..32 - N])
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"bytes").update_usize(N)
    }
}

impl<const N: usize> EvmCdDeserialise for [u8; N] {
    fixed_deserialise_buffer!([u8; 32]);

    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        if N == 0 || N > 32 {
            return Err(invalid_data());
        }
        let mut word = [0u8; 32];
        reader.read_exact(&mut word)?;
        if word[N..].iter().any(|byte| *byte != 0) {
            return Err(invalid_data());
        }
        Ok(word[..N].try_into().unwrap())
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"bytes").update_usize(N)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvmCdArrayError {
    InvalidBounds,
    TooShort,
    TooLong,
}

pub struct EvmCdArray<T, const MIN: usize, const CAP: usize> {
    len: usize,
    values: [core::mem::MaybeUninit<T>; CAP],
}

impl<T, const MIN: usize, const CAP: usize> EvmCdArray<T, MIN, CAP> {
    fn empty() -> Self {
        Self {
            len: 0,
            values: [const { core::mem::MaybeUninit::uninit() }; CAP],
        }
    }

    fn validate_len(len: usize) -> Result<(), EvmCdArrayError> {
        if MIN > CAP {
            return Err(EvmCdArrayError::InvalidBounds);
        }
        if len < MIN {
            return Err(EvmCdArrayError::TooShort);
        }
        if len > CAP {
            return Err(EvmCdArrayError::TooLong);
        }
        Ok(())
    }

    fn push(&mut self, value: T) {
        debug_assert!(self.len < CAP);
        self.values[self.len].write(value);
        self.len += 1;
    }

    pub fn try_from_array(values: [T; CAP], len: usize) -> Result<Self, EvmCdArrayError> {
        Self::validate_len(len)?;
        let mut out = Self::empty();
        for value in values.into_iter().take(len) {
            out.push(value);
        }
        Ok(out)
    }

    pub fn try_from_slice(values: &[T]) -> Result<Self, EvmCdArrayError>
    where
        T: Clone,
    {
        Self::validate_len(values.len())?;
        let mut out = Self::empty();
        for value in values {
            out.push(value.clone());
        }
        Ok(out)
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn capacity(&self) -> usize {
        CAP
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: the first `len` slots are initialized by every constructor
        // and `push`, and `len` can never exceed CAP.
        unsafe { core::slice::from_raw_parts(self.values.as_ptr().cast::<T>(), self.len) }
    }
}

impl<T, const MIN: usize, const CAP: usize> Drop for EvmCdArray<T, MIN, CAP> {
    fn drop(&mut self) {
        for value in &mut self.values[..self.len] {
            // SAFETY: exactly the first `len` slots are initialized.
            unsafe { value.assume_init_drop() };
        }
    }
}

impl<T: Clone, const MIN: usize, const CAP: usize> Clone for EvmCdArray<T, MIN, CAP> {
    fn clone(&self) -> Self {
        Self::try_from_slice(self.as_slice()).expect("an existing EvmCdArray has valid bounds")
    }
}

impl<T: core::fmt::Debug, const MIN: usize, const CAP: usize> core::fmt::Debug
    for EvmCdArray<T, MIN, CAP>
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.debug_list().entries(self.as_slice()).finish()
    }
}

impl<T: PartialEq, const MIN: usize, const CAP: usize> PartialEq for EvmCdArray<T, MIN, CAP> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq, const MIN: usize, const CAP: usize> Eq for EvmCdArray<T, MIN, CAP> {}

impl<T, const MIN: usize, const CAP: usize> AsRef<[T]> for EvmCdArray<T, MIN, CAP> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> EvmCdSerialise for &[T]
where
    T: EvmCdSerialise,
{
    fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        U::from_u32(32).serialise_value(writer)?;
        self.serialise_abi_tail(writer)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        self.len()
            .saturating_mul(T::abi_head_size())
            .saturating_add(32)
            .saturating_add(self.iter().fold(0usize, |size, value| {
                size.saturating_add(value.abi_tail_size())
            }))
    }

    fn serialise_abi_head<W: Write>(
        &self,
        tail_offset: usize,
        writer: &mut W,
    ) -> Result<(), Error> {
        U::from_usize(tail_offset).serialise_value(writer)
    }

    fn serialise_abi_tail<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        U::from_usize(self.len()).serialise_value(writer)?;
        let mut tail_offset = self
            .len()
            .checked_mul(T::abi_head_size())
            .ok_or_else(invalid_data)?;
        for value in *self {
            value.serialise_abi_head(tail_offset, writer)?;
            tail_offset = tail_offset
                .checked_add(value.abi_tail_size())
                .ok_or_else(invalid_data)?;
        }
        for value in *self {
            value.serialise_abi_tail(writer)?;
        }
        Ok(())
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        T::append_abi_type(hasher).update(b"[]")
    }
}

impl<T, const MIN: usize, const CAP: usize> EvmCdSerialise for EvmCdArray<T, MIN, CAP>
where
    T: EvmCdSerialise,
{
    fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        U::from_u32(32).serialise_value(writer)?;
        self.serialise_abi_tail(writer)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        self.len
            .saturating_mul(T::abi_head_size())
            .saturating_add(32)
            .saturating_add(self.as_slice().iter().fold(0usize, |size, value| {
                size.saturating_add(value.abi_tail_size())
            }))
    }

    fn serialise_abi_head<W: Write>(
        &self,
        tail_offset: usize,
        writer: &mut W,
    ) -> Result<(), Error> {
        U::from_usize(tail_offset).serialise_value(writer)
    }

    fn serialise_abi_tail<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        U::from_usize(self.len).serialise_value(writer)?;
        let mut tail_offset = self
            .len
            .checked_mul(T::abi_head_size())
            .ok_or_else(invalid_data)?;
        for value in self.as_slice() {
            value.serialise_abi_head(tail_offset, writer)?;
            tail_offset = tail_offset
                .checked_add(value.abi_tail_size())
                .ok_or_else(invalid_data)?;
        }
        for value in self.as_slice() {
            value.serialise_abi_tail(writer)?;
        }
        Ok(())
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        T::append_abi_type(hasher).update(b"[]")
    }
}

impl<T, const MIN: usize, const CAP: usize> EvmCdDeserialise for EvmCdArray<T, MIN, CAP>
where
    T: EvmCdDeserialise,
{
    type Buffer = <<T::Buffer as EvmCdDecodeBuffer>::Kind as EvmCdBufferKind>::Buffer<(
        [u8; 64],
        [T::Buffer; CAP],
    )>;

    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        if read_usize_word(reader)? != 32 {
            return Err(invalid_data());
        }
        Self::deserialise_tail(reader)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        self.len
            .saturating_mul(T::abi_head_size())
            .saturating_add(32)
            .saturating_add(self.as_slice().iter().fold(0usize, |size, value| {
                size.saturating_add(value.abi_tail_size())
            }))
    }

    fn deserialise_abi_head<R: Read>(reader: &mut R) -> Result<EvmCdHead<Self>, Error> {
        Ok(EvmCdHead::Offset(read_usize_word(reader)?))
    }

    fn deserialise_abi_finish<R: Read>(
        head: EvmCdHead<Self>,
        expected_tail_offset: usize,
        reader: &mut R,
    ) -> Result<Self, Error> {
        match head {
            EvmCdHead::Offset(offset) if offset == expected_tail_offset => {
                Self::deserialise_tail(reader)
            }
            _ => Err(invalid_data()),
        }
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        T::append_abi_type(hasher).update(b"[]")
    }
}

impl<T, const MIN: usize, const CAP: usize> EvmCdArray<T, MIN, CAP>
where
    T: EvmCdDeserialise,
{
    fn deserialise_tail<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let len = read_usize_word(reader)?;
        Self::validate_len(len).map_err(|_| invalid_data())?;
        let mut out = Self::empty();

        if T::is_abi_dynamic() {
            let mut offsets = [0usize; CAP];
            for offset in &mut offsets[..len] {
                match T::deserialise_abi_head(reader)? {
                    EvmCdHead::Offset(value) => *offset = value,
                    EvmCdHead::Value(_) => return Err(invalid_data()),
                }
            }
            let mut expected_tail_offset = len
                .checked_mul(T::abi_head_size())
                .ok_or_else(invalid_data)?;
            for offset in offsets[..len].iter().copied() {
                let value = T::deserialise_abi_finish(
                    EvmCdHead::Offset(offset),
                    expected_tail_offset,
                    reader,
                )?;
                expected_tail_offset = expected_tail_offset
                    .checked_add(value.abi_tail_size())
                    .ok_or_else(invalid_data)?;
                out.push(value);
            }
        } else {
            for _ in 0..len {
                let head = T::deserialise_abi_head(reader)?;
                out.push(T::deserialise_abi_finish(head, 0, reader)?);
            }
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvmCdBytesError {
    TooLong,
}

#[derive(Clone)]
pub struct EvmCdBytes<const CAP: usize> {
    pub len: usize,
    pub bytes: [u8; CAP],
}

impl<const CAP: usize> EvmCdBytes<CAP> {
    pub const fn new() -> Self {
        Self {
            len: 0,
            bytes: [0; CAP],
        }
    }

    pub fn try_from_slice(value: &[u8]) -> Result<Self, EvmCdBytesError> {
        if value.len() > CAP {
            return Err(EvmCdBytesError::TooLong);
        }
        let mut bytes = [0; CAP];
        bytes[..value.len()].copy_from_slice(value);
        Ok(Self {
            len: value.len(),
            bytes,
        })
    }

    pub fn try_from_serialised<T: EvmCdSerialise + ?Sized>(value: &T) -> Result<Self, Error> {
        let mut bytes = Self::new();
        value.serialise(&mut bytes)?;
        Ok(bytes)
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn capacity(&self) -> usize {
        CAP
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn extend_from_slice(&mut self, value: &[u8]) -> Result<(), EvmCdBytesError> {
        let end = self
            .len
            .checked_add(value.len())
            .filter(|end| *end <= CAP)
            .ok_or(EvmCdBytesError::TooLong)?;
        self.bytes[self.len..end].copy_from_slice(value);
        self.len = end;
        Ok(())
    }
}

impl<const CAP: usize> Default for EvmCdBytes<CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize> From<[u8; CAP]> for EvmCdBytes<CAP> {
    fn from(bytes: [u8; CAP]) -> Self {
        Self { len: CAP, bytes }
    }
}

impl<const CAP: usize> TryFrom<&[u8]> for EvmCdBytes<CAP> {
    type Error = EvmCdBytesError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_slice(value)
    }
}

#[cfg(feature = "alloc")]
impl<const CAP: usize> TryFrom<Vec<u8>> for EvmCdBytes<CAP> {
    type Error = EvmCdBytesError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_slice(&value)
    }
}

impl<const CAP: usize> AsRef<[u8]> for EvmCdBytes<CAP> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<const CAP: usize> core::fmt::Debug for EvmCdBytes<CAP> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_slice(), formatter)
    }
}

impl<const CAP: usize> PartialEq for EvmCdBytes<CAP> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<const CAP: usize> Eq for EvmCdBytes<CAP> {}

impl<const CAP: usize> PartialOrd for EvmCdBytes<CAP> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const CAP: usize> Ord for EvmCdBytes<CAP> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<const CAP: usize> core::hash::Hash for EvmCdBytes<CAP> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::hash::Hash::hash(self.as_slice(), state)
    }
}

#[doc(hidden)]
pub struct EvmCdBytesWriter<'a, const CAP: usize> {
    pub bytes: &'a mut EvmCdBytes<CAP>,
}

#[cfg(not(feature = "std"))]
impl<const CAP: usize> Write for EvmCdBytesWriter<'_, CAP> {
    fn write(&mut self, value: &[u8]) -> Result<usize, Error> {
        let written = core::cmp::min(CAP - self.bytes.len, value.len());
        let end = self.bytes.len + written;
        self.bytes.bytes[self.bytes.len..end].copy_from_slice(&value[..written]);
        self.bytes.len = end;
        Ok(written)
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.bytes.len == CAP
    }
}

#[cfg(feature = "std")]
impl<const CAP: usize> Write for EvmCdBytesWriter<'_, CAP> {
    fn write(&mut self, value: &[u8]) -> Result<usize, Error> {
        let written = core::cmp::min(CAP - self.bytes.len, value.len());
        let end = self.bytes.len + written;
        self.bytes.bytes[self.bytes.len..end].copy_from_slice(&value[..written]);
        self.bytes.len = end;
        Ok(written)
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl<const CAP: usize> EvmCdWriteTarget for EvmCdBytes<CAP> {
    type Writer<'a> = EvmCdBytesWriter<'a, CAP>;

    fn writer(&mut self) -> Self::Writer<'_> {
        EvmCdBytesWriter { bytes: self }
    }
}

impl<const CAP: usize> EvmCdSerialise for EvmCdBytes<CAP> {
    fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        write_dynamic_bytes(self.as_slice(), writer)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        dynamic_tail_size(self.len)
    }

    fn serialise_abi_head<W: Write>(
        &self,
        tail_offset: usize,
        writer: &mut W,
    ) -> Result<(), Error> {
        U::from_usize(tail_offset).serialise_value(writer)
    }

    fn serialise_abi_tail<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        write_dynamic_tail(self.as_slice(), writer)
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"bytes")
    }
}

impl<const CAP: usize> EvmCdDeserialise for EvmCdBytes<CAP> {
    type Buffer = EvmCdBuffer<([u8; 64], [u8; CAP], [u8; 31])>;

    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let (bytes, len) = read_dynamic_bytes::<CAP, _>(reader)?;
        Ok(Self { len, bytes })
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        dynamic_tail_size(self.len)
    }

    fn deserialise_abi_head<R: Read>(reader: &mut R) -> Result<EvmCdHead<Self>, Error> {
        Ok(EvmCdHead::Offset(read_usize_word(reader)?))
    }

    fn deserialise_abi_finish<R: Read>(
        head: EvmCdHead<Self>,
        expected_tail_offset: usize,
        reader: &mut R,
    ) -> Result<Self, Error> {
        match head {
            EvmCdHead::Offset(offset) if offset == expected_tail_offset => {
                let (bytes, len) = read_dynamic_tail::<CAP, _>(reader)?;
                Ok(Self { len, bytes })
            }
            _ => Err(invalid_data()),
        }
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"bytes")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvmCdStringError {
    InvalidBounds,
    TooShort,
    TooLong,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EvmCdString<const MIN: usize, const CAP: usize> {
    pub len: usize,
    pub bytes: [u8; CAP],
}

impl<const MIN: usize, const CAP: usize> EvmCdString<MIN, CAP> {
    pub fn try_from_str(value: &str) -> Result<Self, EvmCdStringError> {
        if MIN > CAP {
            return Err(EvmCdStringError::InvalidBounds);
        }
        if value.len() < MIN {
            return Err(EvmCdStringError::TooShort);
        }
        if value.len() > CAP {
            return Err(EvmCdStringError::TooLong);
        }
        let mut bytes = [0u8; CAP];
        bytes[..value.len()].copy_from_slice(value.as_bytes());
        Ok(Self {
            len: value.len(),
            bytes,
        })
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn capacity(&self) -> usize {
        CAP
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub fn as_str(&self) -> &str {
        // SAFETY: constructors and deserialisation validate UTF-8, and no API
        // exposes mutable access to the initialized bytes.
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }
}

impl<const MIN: usize, const CAP: usize> TryFrom<&str> for EvmCdString<MIN, CAP> {
    type Error = EvmCdStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl<const MIN: usize, const CAP: usize> AsRef<str> for EvmCdString<MIN, CAP> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const MIN: usize, const CAP: usize> AsRef<[u8]> for EvmCdString<MIN, CAP> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<const MIN: usize, const CAP: usize> core::borrow::Borrow<str> for EvmCdString<MIN, CAP> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const MIN: usize, const CAP: usize> core::fmt::Display for EvmCdString<MIN, CAP> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl<const MIN: usize, const CAP: usize> core::fmt::Debug for EvmCdString<MIN, CAP> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_str(), formatter)
    }
}

impl<const MIN: usize, const CAP: usize> core::str::FromStr for EvmCdString<MIN, CAP> {
    type Err = EvmCdStringError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(value)
    }
}

impl<const MIN: usize, const CAP: usize> EvmCdSerialise for EvmCdString<MIN, CAP> {
    fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        write_dynamic_bytes(self.as_bytes(), writer)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        dynamic_tail_size(self.len)
    }

    fn serialise_abi_head<W: Write>(
        &self,
        tail_offset: usize,
        writer: &mut W,
    ) -> Result<(), Error> {
        U::from_usize(tail_offset).serialise_value(writer)
    }

    fn serialise_abi_tail<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        write_dynamic_tail(self.as_bytes(), writer)
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"string")
    }
}

impl<const MIN: usize, const CAP: usize> EvmCdDeserialise for EvmCdString<MIN, CAP> {
    type Buffer = EvmCdBuffer<([u8; 64], [u8; CAP], [u8; 31])>;

    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        if MIN > CAP {
            return Err(invalid_data());
        }
        let (bytes, len) = read_dynamic_bytes::<CAP, _>(reader)?;
        if len < MIN || core::str::from_utf8(&bytes[..len]).is_err() {
            return Err(invalid_data());
        }
        Ok(Self { len, bytes })
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        dynamic_tail_size(self.len)
    }

    fn deserialise_abi_head<R: Read>(reader: &mut R) -> Result<EvmCdHead<Self>, Error> {
        Ok(EvmCdHead::Offset(read_usize_word(reader)?))
    }

    fn deserialise_abi_finish<R: Read>(
        head: EvmCdHead<Self>,
        expected_tail_offset: usize,
        reader: &mut R,
    ) -> Result<Self, Error> {
        match head {
            EvmCdHead::Offset(offset) if offset == expected_tail_offset => {
                if MIN > CAP {
                    return Err(invalid_data());
                }
                let (bytes, len) = read_dynamic_tail::<CAP, _>(reader)?;
                if len < MIN || core::str::from_utf8(&bytes[..len]).is_err() {
                    return Err(invalid_data());
                }
                Ok(Self { len, bytes })
            }
            _ => Err(invalid_data()),
        }
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"string")
    }
}

#[cfg(feature = "alloc")]
impl<const MIN: usize, const CAP: usize> From<EvmCdString<MIN, CAP>> for String {
    fn from(value: EvmCdString<MIN, CAP>) -> Self {
        String::from(value.as_str())
    }
}

#[cfg(feature = "alloc")]
fn vec_is_bytes<T: 'static>() -> bool {
    core::any::TypeId::of::<T>() == core::any::TypeId::of::<u8>()
}

#[cfg(feature = "alloc")]
fn vec_as_bytes<T: 'static>(values: &[T]) -> &[u8] {
    debug_assert!(vec_is_bytes::<T>());
    // SAFETY: this helper is called only when TypeId proves that T is u8.
    unsafe { core::slice::from_raw_parts(values.as_ptr().cast::<u8>(), values.len()) }
}

#[cfg(feature = "alloc")]
impl<T> EvmCdSerialise for Vec<T>
where
    T: EvmCdSerialise + 'static,
{
    fn serialise_writer<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        U::from_u32(32).serialise_value(writer)?;
        self.serialise_abi_tail(writer)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        if vec_is_bytes::<T>() {
            dynamic_tail_size(self.len())
        } else {
            self.len()
                .saturating_mul(T::abi_head_size())
                .saturating_add(32)
                .saturating_add(self.iter().fold(0usize, |size, value| {
                    size.saturating_add(value.abi_tail_size())
                }))
        }
    }

    fn serialise_abi_head<W: Write>(
        &self,
        tail_offset: usize,
        writer: &mut W,
    ) -> Result<(), Error> {
        U::from_usize(tail_offset).serialise_value(writer)
    }

    fn serialise_abi_tail<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        if vec_is_bytes::<T>() {
            return write_dynamic_tail(vec_as_bytes(self), writer);
        }

        U::from_usize(self.len()).serialise_value(writer)?;
        let mut tail_offset = self
            .len()
            .checked_mul(T::abi_head_size())
            .ok_or_else(invalid_data)?;
        for value in self {
            value.serialise_abi_head(tail_offset, writer)?;
            tail_offset = tail_offset
                .checked_add(value.abi_tail_size())
                .ok_or_else(invalid_data)?;
        }
        for value in self {
            value.serialise_abi_tail(writer)?;
        }
        Ok(())
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        if vec_is_bytes::<T>() {
            hasher.update(b"bytes")
        } else {
            T::append_abi_type(hasher).update(b"[]")
        }
    }
}

#[cfg(feature = "alloc")]
const MAX_ALLOC_DESERIALISE_LEN: usize = 16 * 1024 * 1024;

#[cfg(feature = "alloc")]
fn read_vec_bytes_tail<R: Read>(reader: &mut R) -> Result<Vec<u8>, Error> {
    let len = read_usize_word(reader)?;
    if len > MAX_ALLOC_DESERIALISE_LEN {
        return Err(invalid_data());
    }
    let mut out = Vec::new();
    out.try_reserve_exact(len).map_err(|_| invalid_data())?;
    out.resize(len, 0);
    reader.read_exact(&mut out)?;
    let padding = (32 - len % 32) % 32;
    let mut padding_bytes = [0u8; 31];
    reader.read_exact(&mut padding_bytes[..padding])?;
    if padding_bytes[..padding].iter().any(|byte| *byte != 0) {
        return Err(invalid_data());
    }
    Ok(out)
}

#[cfg(feature = "alloc")]
fn bytes_into_vec<T: 'static>(bytes: Vec<u8>) -> Vec<T> {
    debug_assert!(vec_is_bytes::<T>());
    let mut bytes = core::mem::ManuallyDrop::new(bytes);
    // SAFETY: TypeId proves that T is u8, so the allocation layout and elements match.
    unsafe {
        Vec::from_raw_parts(
            bytes.as_mut_ptr().cast::<T>(),
            bytes.len(),
            bytes.capacity(),
        )
    }
}

#[cfg(feature = "alloc")]
fn validate_vec_array_len<T: EvmCdDeserialise>(len: usize) -> Result<(), Error> {
    let head_bytes = len
        .checked_mul(T::abi_head_size())
        .ok_or_else(invalid_data)?;
    let value_bytes = len
        .checked_mul(core::mem::size_of::<T>())
        .ok_or_else(invalid_data)?;
    let offset_bytes = if T::is_abi_dynamic() {
        len.checked_mul(core::mem::size_of::<usize>())
            .ok_or_else(invalid_data)?
    } else {
        0
    };
    if head_bytes > MAX_ALLOC_DESERIALISE_LEN
        || value_bytes > MAX_ALLOC_DESERIALISE_LEN
        || offset_bytes > MAX_ALLOC_DESERIALISE_LEN
    {
        return Err(invalid_data());
    }
    Ok(())
}

#[cfg(feature = "alloc")]
fn read_vec_array_tail<T, R>(reader: &mut R) -> Result<Vec<T>, Error>
where
    T: EvmCdDeserialise + 'static,
    R: Read,
{
    let len = read_usize_word(reader)?;
    validate_vec_array_len::<T>(len)?;

    let mut out = Vec::new();
    out.try_reserve_exact(len).map_err(|_| invalid_data())?;
    if T::is_abi_dynamic() {
        let mut offsets = Vec::new();
        offsets.try_reserve_exact(len).map_err(|_| invalid_data())?;
        for _ in 0..len {
            match T::deserialise_abi_head(reader)? {
                EvmCdHead::Offset(offset) => offsets.push(offset),
                EvmCdHead::Value(_) => return Err(invalid_data()),
            }
        }
        let mut expected_tail_offset = len
            .checked_mul(T::abi_head_size())
            .ok_or_else(invalid_data)?;
        for offset in offsets {
            let value =
                T::deserialise_abi_finish(EvmCdHead::Offset(offset), expected_tail_offset, reader)?;
            expected_tail_offset = expected_tail_offset
                .checked_add(value.abi_tail_size())
                .ok_or_else(invalid_data)?;
            out.push(value);
        }
    } else {
        for _ in 0..len {
            let head = T::deserialise_abi_head(reader)?;
            out.push(T::deserialise_abi_finish(head, 0, reader)?);
        }
    }
    Ok(out)
}

#[cfg(feature = "alloc")]
fn read_vec_tail<T, R>(reader: &mut R) -> Result<Vec<T>, Error>
where
    T: EvmCdDeserialise + 'static,
    R: Read,
{
    if vec_is_bytes::<T>() {
        read_vec_bytes_tail(reader).map(bytes_into_vec)
    } else {
        read_vec_array_tail(reader)
    }
}

#[cfg(feature = "alloc")]
impl<T> EvmCdDeserialise for Vec<T>
where
    T: EvmCdDeserialise + 'static,
{
    type Buffer = EvmCdDynamicBuffer<()>;

    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        if read_usize_word(reader)? != 32 {
            return Err(invalid_data());
        }
        read_vec_tail(reader)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        if vec_is_bytes::<T>() {
            dynamic_tail_size(self.len())
        } else {
            self.len()
                .saturating_mul(T::abi_head_size())
                .saturating_add(32)
                .saturating_add(self.iter().fold(0usize, |size, value| {
                    size.saturating_add(value.abi_tail_size())
                }))
        }
    }

    fn deserialise_abi_head<R: Read>(reader: &mut R) -> Result<EvmCdHead<Self>, Error> {
        Ok(EvmCdHead::Offset(read_usize_word(reader)?))
    }

    fn deserialise_abi_finish<R: Read>(
        head: EvmCdHead<Self>,
        expected_tail_offset: usize,
        reader: &mut R,
    ) -> Result<Self, Error> {
        match head {
            EvmCdHead::Offset(offset) if offset == expected_tail_offset => read_vec_tail(reader),
            _ => Err(invalid_data()),
        }
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        if vec_is_bytes::<T>() {
            hasher.update(b"bytes")
        } else {
            T::append_abi_type(hasher).update(b"[]")
        }
    }
}

#[cfg(all(test, not(feature = "std")))]
mod tests {
    use super::{EvmCdDeserialise, EvmCdSerialise};

    #[derive(
        Debug, PartialEq, Eq, bobcat_cd_derive::EvmCdSerialise, bobcat_cd_derive::EvmCdDeserialise,
    )]
    struct SliceValue {
        small: u8,
        large: u32,
    }

    #[test]
    fn mutable_slice_is_a_serialisation_writer_and_deserialisation_reader() {
        let value = SliceValue {
            small: 7,
            large: 0x1234_5678,
        };
        let mut storage = [0u8; 64];

        let mut writer = storage.as_mut_slice();
        value.serialise(&mut writer).unwrap();
        assert!(writer.is_empty());

        let mut reader = storage.as_mut_slice();
        assert_eq!(SliceValue::deserialise_reader(&mut reader).unwrap(), value);
        assert!(reader.is_empty());
    }
}
