use half::f16;

use super::{Buffer, ByteOrder, Result};

impl Buffer {
    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_bytes(1)?[0] as u8)
    }

    pub fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_bytes(1)?[0] as i8)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let order = self.order;
        let bytes = self.read_bytes(2)?.try_into().unwrap();
        Ok(match order {
            ByteOrder::BigEndian => u16::from_be_bytes(bytes),
            ByteOrder::LittleEndian => u16::from_le_bytes(bytes),
        })
    }

    pub fn read_i16(&mut self) -> Result<i16> {
        let order = self.order;
        let bytes = self.read_bytes(2)?.try_into().unwrap();
        Ok(match order {
            ByteOrder::BigEndian => i16::from_be_bytes(bytes),
            ByteOrder::LittleEndian => i16::from_le_bytes(bytes),
        })
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let order = self.order;
        let bytes = self.read_bytes(4)?.try_into().unwrap();
        Ok(match order {
            ByteOrder::BigEndian => u32::from_be_bytes(bytes),
            ByteOrder::LittleEndian => u32::from_le_bytes(bytes),
        })
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        let order = self.order;
        let bytes = self.read_bytes(4)?.try_into().unwrap();
        Ok(match order {
            ByteOrder::BigEndian => i32::from_be_bytes(bytes),
            ByteOrder::LittleEndian => i32::from_le_bytes(bytes),
        })
    }

    pub fn read_u64(&mut self) -> Result<u64> {
        let order = self.order;
        let bytes = self.read_bytes(8)?.try_into().unwrap();
        Ok(match order {
            ByteOrder::BigEndian => u64::from_be_bytes(bytes),
            ByteOrder::LittleEndian => u64::from_le_bytes(bytes),
        })
    }

    pub fn read_i64(&mut self) -> Result<i64> {
        let order = self.order;
        let bytes = self.read_bytes(8)?.try_into().unwrap();
        Ok(match order {
            ByteOrder::BigEndian => i64::from_be_bytes(bytes),
            ByteOrder::LittleEndian => i64::from_le_bytes(bytes),
        })
    }

    pub fn read_f16(&mut self) -> Result<f16> {
        let order = self.order;
        let bytes = self.read_bytes(2)?.try_into().unwrap();
        Ok(match order {
            ByteOrder::BigEndian => f16::from_be_bytes(bytes),
            ByteOrder::LittleEndian => f16::from_le_bytes(bytes),
        })
    }

    pub fn read_f32(&mut self) -> Result<f32> {
        let order = self.order;
        let bytes = self.read_bytes(4)?.try_into().unwrap();
        Ok(match order {
            ByteOrder::BigEndian => f32::from_be_bytes(bytes),
            ByteOrder::LittleEndian => f32::from_le_bytes(bytes),
        })
    }

    pub fn read_f64(&mut self) -> Result<f64> {
        let order = self.order;
        let bytes = self.read_bytes(8)?.try_into().unwrap();
        Ok(match order {
            ByteOrder::BigEndian => f64::from_be_bytes(bytes),
            ByteOrder::LittleEndian => f64::from_le_bytes(bytes),
        })
    }

    pub fn read_bytes(&mut self, size: usize) -> Result<&[u8]> {
        self.check_not_closed()?;
        self.check_readable()?;
        self.check_read_bounds(size)?;

        let bytes = &self.data[self.head..self.head + size];
        self.head += size;

        Ok(bytes)
    }
}
