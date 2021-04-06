use anyhow::{anyhow, bail, Context, Result};
use bincode::{
    config::{AllowTrailing, FixintEncoding, WithOtherIntEncoding, WithOtherTrailing},
    DefaultOptions, Options,
};
use encoding_rs::{UTF_16LE, WINDOWS_1252};
use imgui::ImString;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use num_traits::FromPrimitive;
use serde::Deserialize;
use std::{hash::Hash, mem::size_of, usize};

use crate::ui::Ui;

lazy_static! {
    pub static ref BINCODE: WithOtherTrailing<WithOtherIntEncoding<DefaultOptions, FixintEncoding>, AllowTrailing> =
        bincode::DefaultOptions::new().with_fixint_encoding().allow_trailing_bytes();
}

pub struct SaveCursor {
    position: usize,
    bytes: Vec<u8>,
}

impl SaveCursor {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { position: 0, bytes }
    }

    pub fn read(&mut self, num_bytes: usize) -> Result<&[u8]> {
        let end = self.position + num_bytes;
        if self.bytes.len() < end {
            bail!("unexpected end of file");
        }

        let slice = &self.bytes[self.position..end];
        self.position = end;

        Ok(slice)
    }
}

pub trait SaveData
where
    Self: Sized,
{
    fn deserialize(input: &mut SaveCursor) -> Result<Self>;
    fn draw_raw_ui(&mut self, ui: &Ui, ident: &str);

    // Generic
    fn deserialize_from<'a, D>(input: &'a mut SaveCursor) -> Result<D>
    where
        D: Deserialize<'a>,
    {
        let size = size_of::<D>();
        let bytes = input.read(size)?;

        BINCODE.deserialize::<D>(bytes).map_err(|e| anyhow!(e))
    }

    fn deserialize_from_bool(input: &mut SaveCursor) -> Result<bool> {
        Ok(Self::deserialize_from::<i32>(input)? != 0)
    }

    fn deserialize_enum_from_u8<E>(input: &mut SaveCursor) -> Result<E>
    where
        E: FromPrimitive,
    {
        E::from_u8(Self::deserialize_from::<u8>(input)?).context("invalid enum representation")
    }

    fn deserialize_enum_from_u32<E>(input: &mut SaveCursor) -> Result<E>
    where
        E: FromPrimitive,
    {
        E::from_u32(Self::deserialize_from::<u32>(input)?).context("invalid enum representation")
    }

    // String
    fn deserialize_from_string(input: &mut SaveCursor) -> Result<ImString> {
        let len = Self::deserialize_from::<i32>(input)?;

        if len == 0 {
            return Ok(ImString::default());
        }

        let string = if len < 0 {
            // Unicode
            let string_len = (len.abs() * 2) as usize;

            let bytes = input.read(string_len)?.to_owned();

            let (decoded, _, had_errors) = UTF_16LE.decode(&bytes);
            if had_errors {
                bail!("string encoding error");
            }

            ImString::new(decoded)
        } else {
            // Ascii
            let string_len = len as usize;

            let bytes = input.read(string_len)?.to_owned();

            let (decoded, _, had_errors) = WINDOWS_1252.decode(&bytes);
            if had_errors {
                bail!("string encoding error");
            }

            ImString::new(decoded)
        };

        Ok(string)
    }

    // Array
    fn deserialize_from_array<D>(input: &mut SaveCursor) -> Result<Vec<D>>
    where
        D: SaveData,
    {
        let len = Self::deserialize_from::<u32>(input)?;
        let mut vec = Vec::with_capacity(len as usize);
        if len == 0 {
            return Ok(vec);
        }

        for _ in 0..len {
            vec.push(D::deserialize(input)?);
        }

        Ok(vec)
    }

    // IndexMap
    fn deserialize_from_indexmap<K, V>(input: &mut SaveCursor) -> Result<IndexMap<K, V>>
    where
        K: SaveData + Eq + Hash,
        V: SaveData,
    {
        let len = Self::deserialize_from::<u32>(input)?;
        let mut map = IndexMap::with_capacity(len as usize);
        if len == 0 {
            return Ok(map);
        }

        for _ in 0..len {
            map.insert(K::deserialize(input)?, V::deserialize(input)?);
        }

        Ok(map)
    }
}

// Implémentation des dummy
impl<const LENGTH: usize> SaveData for [u8; LENGTH] {
    fn deserialize(input: &mut SaveCursor) -> Result<Self> {
        let mut array = [0; LENGTH];
        for byte in array.iter_mut() {
            *byte = Self::deserialize_from(input)?
        }
        Ok(array)
    }

    fn draw_raw_ui(&mut self, _: &Ui, _: &str) {}
}

// Implémentation des types std
impl SaveData for i32 {
    fn deserialize(input: &mut SaveCursor) -> Result<Self> {
        Self::deserialize_from(input)
    }

    fn draw_raw_ui(&mut self, ui: &Ui, ident: &str) {
        ui.draw_edit_i32(ident, self);
    }
}

impl SaveData for f32 {
    fn deserialize(input: &mut SaveCursor) -> Result<Self> {
        Self::deserialize_from(input)
    }

    fn draw_raw_ui(&mut self, ui: &Ui, ident: &str) {
        ui.draw_edit_f32(ident, self);
    }
}

macro_rules! impl_save_data {
    ($type:ty) => {
        impl SaveData for $type {
            fn deserialize(input: &mut SaveCursor) -> Result<Self> {
                Self::deserialize_from(input)
            }

            fn draw_raw_ui(&mut self, _ui: &Ui, _ident: &str) {}
        }
    };
}

impl_save_data!(u8);
impl_save_data!(i8);
impl_save_data!(u32);

impl SaveData for bool {
    fn deserialize(input: &mut SaveCursor) -> Result<Self> {
        Self::deserialize_from_bool(input)
    }

    fn draw_raw_ui(&mut self, ui: &Ui, ident: &str) {
        ui.draw_edit_bool(ident, self);
    }
}

impl SaveData for ImString {
    fn deserialize(input: &mut SaveCursor) -> Result<Self> {
        Self::deserialize_from_string(input)
    }

    fn draw_raw_ui(&mut self, ui: &Ui, ident: &str) {
        ui.draw_edit_string(ident, self);
    }
}

impl<D> SaveData for Vec<D>
where
    D: SaveData,
{
    fn deserialize(input: &mut SaveCursor) -> Result<Self> {
        Self::deserialize_from_array(input)
    }

    fn draw_raw_ui(&mut self, ui: &Ui, ident: &str) {
        ui.draw_vec(ident, self.len(), |i|{
            let ident = i.to_string();
            self[i].draw_raw_ui(ui, &ident);
        });
    }
}

impl<K, V> SaveData for IndexMap<K, V>
where
    K: SaveData + Eq + Hash,
    V: SaveData,
{
    fn deserialize(input: &mut SaveCursor) -> Result<Self> {
        Self::deserialize_from_indexmap(input)
    }

    fn draw_raw_ui(&mut self, _ui: &Ui, _ident: &str) {}
}
