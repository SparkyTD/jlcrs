use num_traits::FromPrimitive;
use serde_json::Value;

#[derive(Debug)]
pub struct JsonArrayReader {
    array: Vec<Value>,
    index: usize,
}

#[allow(unused)]
impl JsonArrayReader {
    pub fn new(array: Vec<Value>) -> Self {
        Self { array, index: 0 }
    }

    pub fn read_string(&mut self) -> Option<String> {
        self.index += 1;
        self.array[self.index - 1].as_str().map(|s| s.to_string())
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        self.index += 1;
        self.array[self.index - 1].as_u64().map(|n| n as u8)
    }

    pub fn read_u16(&mut self) -> Option<u16> {
        self.index += 1;
        self.array[self.index - 1].as_u64().map(|n| n as u16)
    }

    pub fn read_u32(&mut self) -> Option<u32> {
        self.index += 1;
        self.array[self.index - 1].as_u64().map(|n| n as u32)
    }

    pub fn read_u64(&mut self) -> Option<u64> {
        self.index += 1;
        self.array[self.index - 1].as_u64()
    }

    pub fn read_i8(&mut self) -> Option<i8> {
        self.index += 1;
        self.array[self.index - 1].as_i64().map(|n| n as i8)
    }

    pub fn read_i16(&mut self) -> Option<i16> {
        self.index += 1;
        self.array[self.index - 1].as_i64().map(|n| n as i16)
    }

    pub fn read_i32(&mut self) -> Option<i32> {
        self.index += 1;
        self.array[self.index - 1].as_i64().map(|n| n as i32)
    }

    pub fn read_i64(&mut self) -> Option<i64> {
        self.index += 1;
        self.array[self.index - 1].as_i64()
    }

    pub fn read_f32(&mut self) -> Option<f32> {
        self.index += 1;
        self.array[self.index - 1].as_f64().map(|n| n as f32)
    }

    pub fn read_f64(&mut self) -> Option<f64> {
        self.index += 1;
        self.array[self.index - 1].as_f64()
    }

    pub fn read_bool(&mut self) -> Option<bool> {
        self.index += 1;
        let value = &self.array[self.index - 1];
        if value.is_boolean() {
            value.as_bool()
        } else {
            value.as_u64().map(|n| n == 1)
        }
    }

    pub fn read_value(&mut self) -> Option<Value> {
        self.index += 1;
        Some(self.array[self.index - 1].clone())
    }

    pub fn read_enum<T>(&mut self) -> Option<T>
    where T : FromPrimitive {
        self.index += 1;
        self.read_u64().map(|n| FromPrimitive::from_u64(n).unwrap())
    }

    pub fn can_read(&self) -> bool {
        self.index < self.array.len()
    }

    pub fn remaining(&self) -> usize {
        self.array.len() - self.index
    }
}