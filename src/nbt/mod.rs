use std::{
    collections::HashMap,
    io::{self, Write},
};

pub trait WriteNbtExt: Write {
    fn write_type(&mut self, value: NbtType) -> io::Result<()> {
        self.write_all(&[value as u8])
    }

    fn write_u8(&mut self, value: Nbt<'_>) -> io::Result<()> {
        if let Nbt::Byte(value) = value {
            self.write_all(&[value])
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Value is not u8"))
        }
    }

    fn write_i16(&mut self, value: Nbt<'_>) -> io::Result<()> {
        if let Nbt::Short(value) = value {
            self.write_all(&value.to_be_bytes())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Value is not i16"))
        }
    }

    fn write_i32(&mut self, value: Nbt<'_>) -> io::Result<()> {
        if let Nbt::Int(value) = value {
            self.write_all(&value.to_be_bytes())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Value is not i32"))
        }
    }

    fn write_i64(&mut self, value: Nbt<'_>) -> io::Result<()> {
        if let Nbt::Long(value) = value {
            self.write_all(&value.to_be_bytes())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Value is not i64"))
        }
    }

    fn write_string(&mut self, value: Nbt<'_>) -> io::Result<()> {
        if let Nbt::String(value) = value {
            self.write_len_prefixed_string(value)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Value is not a string",
            ))
        }
    }

    fn write_len_prefixed_string(&mut self, value: &str) -> io::Result<()> {
        let len = value.len() as u16;
        self.write_all(&len.to_be_bytes())?;
        self.write_all(value.as_bytes())
    }

    fn write_f32(&mut self, value: Nbt<'_>) -> io::Result<()> {
        if let Nbt::Float(value) = value {
            self.write_all(&value.to_be_bytes())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Value is not f32"))
        }
    }

    fn write_f64(&mut self, value: Nbt<'_>) -> io::Result<()> {
        if let Nbt::Double(value) = value {
            self.write_all(&value.to_be_bytes())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Value is not f64"))
        }
    }

    fn write_tag(&mut self, value: Nbt<'_>) -> io::Result<()> {
        match value {
            Nbt::Byte(_) => self.write_u8(value),
            Nbt::Short(_) => self.write_i16(value),
            Nbt::Int(_) => self.write_i32(value),
            Nbt::Long(_) => self.write_i64(value),
            Nbt::Float(_) => self.write_f32(value),
            Nbt::Double(_) => self.write_f64(value),
            Nbt::ByteArray(_) => todo!(),
            Nbt::String(_) => self.write_string(value),
            Nbt::List(_) => todo!(),
            Nbt::Compound(c) => self.write_compound(c),
            Nbt::IntArray(_) => todo!(),
            Nbt::LongArray(_) => self.write_long_array(value),
        }
    }

    fn write_compound(&mut self, value: NbtCompound<'_>) -> io::Result<()> {
        self.write_type(NbtType::Compound)?;
        self.write_len_prefixed_string("")?;
        for (key, value) in value.0 {
            self.write_type(value.get_type())?;
            self.write_len_prefixed_string(key)?;
            self.write_tag(value)?;
        }
        self.write_type(NbtType::End)?;

        Ok(())
    }

    fn write_long_array(&mut self, value: Nbt<'_>) -> io::Result<()> {
        if let Nbt::LongArray(value) = value {
            self.write_i32(Nbt::Int(value.len() as i32))?;

            for v in value {
                self.write_i64(Nbt::Long(v))?;
            }

            Ok(())
        } else {
            Err(io::Error::other("Value is not LongArray"))
        }
    }
}

impl<W: Write + ?Sized> WriteNbtExt for W {}

#[derive(Debug)]
pub struct NbtCompound<'a>(HashMap<&'a str, Nbt<'a>>);

impl<'a> NbtCompound<'a> {
    pub fn default() -> Self {
        NbtCompound(HashMap::new())
    }

    fn add_element(&mut self, key: &'a str, value: Nbt<'a>) {
        self.0.insert(key, value);
    }

    pub fn set_byte(&mut self, key: &'a str, value: u8) {
        self.add_element(key, Nbt::Byte(value));
    }

    pub fn set_short(&mut self, key: &'a str, value: i16) {
        self.add_element(key, Nbt::Short(value));
    }

    pub fn set_int(&mut self, key: &'a str, value: i32) {
        self.add_element(key, Nbt::Int(value));
    }

    pub fn set_long(&mut self, key: &'a str, value: i64) {
        self.add_element(key, Nbt::Long(value));
    }

    pub fn set_float(&mut self, key: &'a str, value: f32) {
        self.add_element(key, Nbt::Float(value));
    }

    pub fn set_double(&mut self, key: &'a str, value: f64) {
        self.add_element(key, Nbt::Double(value));
    }

    pub fn set_byte_array(&mut self, key: &'a str, value: &'a [u8]) {
        self.add_element(key, Nbt::ByteArray(value));
    }

    pub fn set_string(&mut self, key: &'a str, value: &'a str) {
        self.add_element(key, Nbt::String(value));
    }

    pub fn set_list(&mut self, key: &'a str, value: Vec<Nbt<'a>>) {
        self.add_element(key, Nbt::List(value));
    }

    pub fn set_compound(&mut self, key: &'a str, value: NbtCompound<'a>) {
        self.add_element(key, Nbt::Compound(value));
    }

    pub fn set_int_array(&mut self, key: &'a str, value: Vec<i32>) {
        self.add_element(key, Nbt::IntArray(value));
    }

    pub fn set_long_array(&mut self, key: &'a str, value: Vec<i64>) {
        self.add_element(key, Nbt::LongArray(value))
    }

    pub fn get_byte(&self, key: &'a str) -> Option<u8> {
        match self.0.get(key) {
            Some(Nbt::Byte(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn get_short(&self, key: &'a str) -> Option<i16> {
        match self.0.get(key) {
            Some(Nbt::Short(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn get_int(&self, key: &'a str) -> Option<i32> {
        match self.0.get(key) {
            Some(Nbt::Int(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn get_long(&self, key: &'a str) -> Option<i64> {
        match self.0.get(key) {
            Some(Nbt::Long(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn get_float(&self, key: &'a str) -> Option<f32> {
        match self.0.get(key) {
            Some(Nbt::Float(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn get_double(&self, key: &'a str) -> Option<f64> {
        match self.0.get(key) {
            Some(Nbt::Double(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn get_byte_array(&self, key: &'a str) -> Option<&'a [u8]> {
        match self.0.get(key) {
            Some(Nbt::ByteArray(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn get_string(&self, key: &'a str) -> Option<&'a str> {
        match self.0.get(key) {
            Some(Nbt::String(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn get_list(&self, key: &'a str) -> Option<&Vec<Nbt<'a>>> {
        match self.0.get(key) {
            Some(Nbt::List(value)) => Some(value),
            _ => None,
        }
    }

    pub fn get_compound(&self, key: &'a str) -> Option<&NbtCompound<'a>> {
        match self.0.get(key) {
            Some(Nbt::Compound(value)) => Some(value),
            _ => None,
        }
    }

    pub fn get_int_array(&self, key: &'a str) -> Option<&[i32]> {
        match self.0.get(key) {
            Some(Nbt::IntArray(value)) => Some(value),
            _ => None,
        }
    }

    pub fn get_long_array(&self, key: &'a str) -> Option<&[i64]> {
        match self.0.get(key) {
            Some(Nbt::LongArray(value)) => Some(value),
            _ => None,
        }
    }
}

impl<'a> NbtCompound<'a> {
    pub fn pack(self) -> io::Result<Vec<u8>> {
        let mut buffer = vec![];
        buffer.write_compound(self)?;

        Ok(buffer)
    }
}

pub enum NbtType {
    End = 0,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ByteArray,
    String,
    List,
    Compound,
    IntArray,
    LongArray,
}

#[derive(Debug)]
pub enum Nbt<'a> {
    Byte(u8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(&'a [u8]),
    String(&'a str),
    List(Vec<Nbt<'a>>),
    Compound(NbtCompound<'a>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl<'a> Nbt<'a> {
    pub fn get_type(&self) -> NbtType {
        match self {
            Self::Byte(_) => NbtType::Byte,
            Nbt::Short(_) => NbtType::Short,
            Nbt::Int(_) => NbtType::Int,
            Nbt::Long(_) => NbtType::Long,
            Nbt::Float(_) => NbtType::Float,
            Nbt::Double(_) => NbtType::Double,
            Nbt::ByteArray(_) => NbtType::ByteArray,
            Nbt::String(_) => NbtType::String,
            Nbt::List(_) => NbtType::List,
            Nbt::Compound(_) => NbtType::Compound,
            Nbt::IntArray(_) => NbtType::IntArray,
            Nbt::LongArray(_) => NbtType::LongArray,
        }
    }
}
