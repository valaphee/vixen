use crate::tank::stu::error::{Error, Result};
use crate::tank::stu::{read_bag, Field, InlineArray, Instance};
use byteorder::{LittleEndian, ReadBytesExt};
use crc::Crc;
use std::io::{Cursor, Read, Seek, SeekFrom};

pub struct Deserializer<'de> {
    data: &'de [u8],

    instances: Vec<Instance>,
    inline_arrays: Vec<InlineArray>,
    fields: Vec<Vec<Field>>,
    dynamic_data: Cursor<Vec<u8>>,

    current_field_hash: u32,
    current_field_size: u32,
}

impl<'de> Deserializer<'de> {
    pub fn from_slice(data: &'de mut [u8]) -> Result<Self> {
        let mut data_cursor = Cursor::new(data);

        let instances = read_bag(&mut data_cursor, Instance::read_from).map_err(Error::Io)?;
        let inline_arrays =
            read_bag(&mut data_cursor, InlineArray::read_from).map_err(Error::Io)?;
        let field_sets = read_bag(&mut data_cursor, |read| read_bag(read, Field::read_from))
            .map_err(Error::Io)?;
        let dynamic_data_size = data_cursor.read_u32::<LittleEndian>().map_err(Error::Io)?;
        let dynamic_data_offset = data_cursor.read_u32::<LittleEndian>().map_err(Error::Io)?;
        let data_offset = data_cursor.read_u32::<LittleEndian>().map_err(Error::Io)?;

        let mut dynamic_data = vec![0; dynamic_data_size as usize];
        if dynamic_data_size > 0 {
            data_cursor
                .seek(SeekFrom::Start(dynamic_data_offset as u64))
                .map_err(Error::Io)?;
            data_cursor
                .read_exact(&mut dynamic_data)
                .map_err(Error::Io)?;
        }
        data_cursor
            .seek(SeekFrom::Start(data_offset as u64))
            .map_err(Error::Io)?;

        Ok(Self {
            data: &mut data_cursor.into_inner()[data_offset as usize..],

            instances,
            inline_arrays,
            fields: field_sets,
            dynamic_data: Cursor::new(dynamic_data),

            current_field_hash: 0,
            current_field_size: 0,
        })
    }
}

impl<'de, 'a> serde::de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.current_field_size != 1 {
            return Err(Error::InvalidLength {
                length: self.current_field_size,
                expected: 1,
            });
        }

        visitor.visit_i8(self.data.read_i8().map_err(Error::Io)?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.current_field_size != 2 {
            return Err(Error::InvalidLength {
                length: self.current_field_size,
                expected: 2,
            });
        }

        visitor.visit_i16(self.data.read_i16::<LittleEndian>().map_err(Error::Io)?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.current_field_size != 4 {
            return Err(Error::InvalidLength {
                length: self.current_field_size,
                expected: 4,
            });
        }

        visitor.visit_i32(self.data.read_i32::<LittleEndian>().map_err(Error::Io)?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.current_field_size != 8 {
            return Err(Error::InvalidLength {
                length: self.current_field_size,
                expected: 8,
            });
        }

        visitor.visit_i64(self.data.read_i64::<LittleEndian>().map_err(Error::Io)?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.current_field_size != 1 {
            return Err(Error::InvalidLength {
                length: self.current_field_size,
                expected: 1,
            });
        }

        visitor.visit_u8(self.data.read_u8().map_err(Error::Io)?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.current_field_size != 2 {
            return Err(Error::InvalidLength {
                length: self.current_field_size,
                expected: 2,
            });
        }

        visitor.visit_u16(self.data.read_u16::<LittleEndian>().map_err(Error::Io)?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.current_field_size != 4 {
            return Err(Error::InvalidLength {
                length: self.current_field_size,
                expected: 4,
            });
        }

        visitor.visit_u32(self.data.read_u32::<LittleEndian>().map_err(Error::Io)?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.current_field_size != 8 {
            return Err(Error::InvalidLength {
                length: self.current_field_size,
                expected: 8,
            });
        }

        visitor.visit_u64(self.data.read_u64::<LittleEndian>().map_err(Error::Io)?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.current_field_size != 4 {
            return Err(Error::InvalidLength {
                length: self.current_field_size,
                expected: 4,
            });
        }

        visitor.visit_f32(self.data.read_f32::<LittleEndian>().map_err(Error::Io)?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.current_field_size != 8 {
            return Err(Error::InvalidLength {
                length: self.current_field_size,
                expected: 8,
            });
        }

        visitor.visit_f64(self.data.read_f64::<LittleEndian>().map_err(Error::Io)?)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.current_field_size != 4 {
            return Err(Error::InvalidLength {
                length: self.current_field_size,
                expected: 4,
            });
        }

        let offset = self.data.read_u32::<LittleEndian>().unwrap();
        self.dynamic_data
            .seek(SeekFrom::Start(offset as u64))
            .unwrap();
        let field_size = self.dynamic_data.read_u32::<LittleEndian>().unwrap();
        let field_data_hash = self.dynamic_data.read_u32::<LittleEndian>().unwrap();
        let field_offset = self.dynamic_data.read_u64::<LittleEndian>().unwrap();
        self.dynamic_data
            .seek(SeekFrom::Start(field_offset))
            .unwrap();
        let mut field_data = vec![0; field_size as usize];
        self.dynamic_data.read_exact(&mut field_data).unwrap();

        {
            let mut digest = CRC32.digest();
            digest.update(&field_data);
            if field_data_hash != digest.finalize() {
                return Err(Error::IntegrityError);
            }
        }

        visitor.visit_string(String::from_utf8(field_data).unwrap())
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let field_size = self.data.read_u32::<LittleEndian>().unwrap();
        visitor.visit_seq(SeqAccess {
            de: self,

            field_size,
        })
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let fields_id = self.data.read_u32::<LittleEndian>().map_err(Error::Io)?;
        let fields = self
            .fields
            .get(fields_id as usize)
            .unwrap()
            .clone()
            .into_iter();
        visitor.visit_map(MapAccess { de: self, fields })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_str(&format!("{:08X}", self.current_field_hash))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut field_data = vec![0; self.current_field_size as usize];
        self.data.read_exact(&mut field_data).unwrap();
        visitor.visit_byte_buf(field_data)
    }
}

struct SeqAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,

    field_size: u32,
}

impl<'a, 'de> serde::de::SeqAccess<'de> for SeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.field_size == 0 {
            return Ok(None);
        }
        self.field_size -= 1;

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.field_size as usize)
    }
}

struct MapAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,

    fields: std::vec::IntoIter<Field>,
}

impl<'a, 'de> serde::de::MapAccess<'de> for MapAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if let Some(field) = self.fields.next() {
            self.de.current_field_hash = field.hash;
            self.de.current_field_size = field.size;
            if self.de.current_field_size == 0 {
                self.de.current_field_size =
                    self.de.data.read_u32::<LittleEndian>().map_err(Error::Io)?;
            }

            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

static CRC32: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
