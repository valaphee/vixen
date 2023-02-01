use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use crc::Crc;

use crate::stu::{
    crc64,
    error::{Error, Result},
    DynamicDataHeader, Field, Header, InlineArray, Instance,
};

pub struct Deserializer<'de> {
    data: &'de [u8],
    dynamic_data: &'de [u8],

    header_crc: u64,
    instances: &'de [Instance],
    inline_arrays: &'de [InlineArray],
    fields: Vec<&'de [Field]>,

    current_field_hash: u32,
    current_field_size: u32,
}

impl<'de> Deserializer<'de> {
    pub fn from_slice(data: &'de mut [u8]) -> Result<Self> {
        let header: &Header = bytemuck::from_bytes(&data[..std::mem::size_of::<Header>()]);
        let header_crc = crc64(&data[..std::mem::size_of::<Header>()]);

        let instances: &[Instance] = bytemuck::cast_slice(
            &data[header.instance_offset as usize
                ..header.instance_offset as usize
                    + header.instance_count as usize * std::mem::size_of::<Instance>()],
        );
        let inline_arrays: &[InlineArray] = bytemuck::cast_slice(
            &data[header.inline_array_offset as usize
                ..header.inline_array_offset as usize
                    + header.inline_array_count as usize * std::mem::size_of::<InlineArray>()],
        );
        let mut fields: Vec<&[Field]> = Vec::new();
        {
            let mut sub_data = &data[header.field_set_offset as usize
                ..header.field_set_offset as usize + header.field_set_count as usize * 8];
            for _ in 0..header.field_set_count {
                let count = sub_data.read_u32::<LittleEndian>()? as usize;
                let offset = sub_data.read_u32::<LittleEndian>()? as usize;
                fields.push(bytemuck::cast_slice(
                    &data[offset..offset + count * std::mem::size_of::<Field>()],
                ))
            }
        }

        Ok(Self {
            data: &data[header.data_offset as usize..],
            dynamic_data: &data[header.dynamic_data_offset as usize
                ..header.dynamic_data_offset as usize + header.dynamic_data_size as usize],

            instances,
            inline_arrays,
            fields,
            header_crc,

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

        // likely to be an obfuscated GUID
        let mut value = self.data.read_u64::<LittleEndian>().map_err(Error::Io)?;
        value ^= (self.current_field_hash as u64 | ((self.current_field_hash as u64) << 32))
            ^ self.header_crc;
        let mut bytes = value.to_le_bytes();
        bytes.swap(0, 3);
        bytes.swap(7, 1);
        bytes.swap(2, 6);
        bytes.swap(4, 5);
        visitor.visit_u64(u64::from_le_bytes(bytes))
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

        let offset = self.data.read_u32::<LittleEndian>()? as usize;
        let dynamic_data_header: &DynamicDataHeader = bytemuck::from_bytes(
            &self.dynamic_data[offset..offset + std::mem::size_of::<DynamicDataHeader>()],
        );
        let dynamic_data = &self.dynamic_data[dynamic_data_header.offset as usize
            ..dynamic_data_header.offset as usize + dynamic_data_header.size as usize];
        {
            let mut digest = CRC32.digest();
            digest.update(dynamic_data);
            if dynamic_data_header.crc != digest.finalize() {
                return Err(Error::IntegrityError);
            }
        }

        visitor.visit_string(std::str::from_utf8(dynamic_data).unwrap().to_string())
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

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        // there is no optional type, but this is needed for supporting optional fields
        visitor.visit_some(self)
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
        let fields = self.fields.get(fields_id as usize).unwrap().iter();
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
        self.data = &self.data[self.current_field_size as usize..];
        visitor.visit_unit()
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

    fields: std::slice::Iter<'de, Field>,
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
