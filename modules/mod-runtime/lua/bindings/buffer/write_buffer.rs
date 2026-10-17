use half::f16;

use super::{Buffer, ByteOrder, Result};

impl Buffer {
    pub fn write_bool(&mut self, value: bool) -> Result<()> {
        self.write_bytes(&[u8::from(value)])
    }

    pub fn write_u8(&mut self, value: u8) -> Result<()> {
        self.write_bytes(&[value as u8])
    }

    pub fn write_i8(&mut self, value: i8) -> Result<()> {
        self.write_bytes(&[value as u8])
    }

    pub fn write_u16(&mut self, value: u16) -> Result<()> {
        let bytes = match self.order {
            ByteOrder::BigEndian => value.to_be_bytes(),
            ByteOrder::LittleEndian => value.to_le_bytes(),
        };
        self.write_bytes(&bytes)
    }

    pub fn write_i16(&mut self, value: i16) -> Result<()> {
        let bytes = match self.order {
            ByteOrder::BigEndian => value.to_be_bytes(),
            ByteOrder::LittleEndian => value.to_le_bytes(),
        };
        self.write_bytes(&bytes)
    }

    pub fn write_u32(&mut self, value: u32) -> Result<()> {
        let bytes = match self.order {
            ByteOrder::BigEndian => value.to_be_bytes(),
            ByteOrder::LittleEndian => value.to_le_bytes(),
        };
        self.write_bytes(&bytes)
    }

    pub fn write_i32(&mut self, value: i32) -> Result<()> {
        let bytes = match self.order {
            ByteOrder::BigEndian => value.to_be_bytes(),
            ByteOrder::LittleEndian => value.to_le_bytes(),
        };
        self.write_bytes(&bytes)
    }

    pub fn write_u64(&mut self, value: u64) -> Result<()> {
        let bytes = match self.order {
            ByteOrder::BigEndian => value.to_be_bytes(),
            ByteOrder::LittleEndian => value.to_le_bytes(),
        };
        self.write_bytes(&bytes)
    }

    pub fn write_i64(&mut self, value: i64) -> Result<()> {
        let bytes = match self.order {
            ByteOrder::BigEndian => value.to_be_bytes(),
            ByteOrder::LittleEndian => value.to_le_bytes(),
        };
        self.write_bytes(&bytes)
    }

    pub fn write_f16(&mut self, value: f16) -> Result<()> {
        let bytes = match self.order {
            ByteOrder::BigEndian => value.to_be_bytes(),
            ByteOrder::LittleEndian => value.to_le_bytes(),
        };
        self.write_bytes(&bytes)
    }

    pub fn write_f32(&mut self, value: f32) -> Result<()> {
        let bytes = match self.order {
            ByteOrder::BigEndian => value.to_be_bytes(),
            ByteOrder::LittleEndian => value.to_le_bytes(),
        };
        self.write_bytes(&bytes)
    }

    pub fn write_f64(&mut self, value: f64) -> Result<()> {
        let bytes = match self.order {
            ByteOrder::BigEndian => value.to_be_bytes(),
            ByteOrder::LittleEndian => value.to_le_bytes(),
        };
        self.write_bytes(&bytes)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.check_not_closed()?;
        self.check_writable()?;
        self.reserve(bytes.len())?;

        self.data[self.tail..self.tail + bytes.len()].copy_from_slice(bytes);
        self.tail += bytes.len();

        Ok(())
    }
}
