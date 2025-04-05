use std::any::Any;

pub trait Reflect: Any {
    fn item_by_index(&self, index: usize) -> Option<Box<dyn Reflect>> {
        None
    }
    fn item_len(&self) -> usize {
        0
    }
    fn as_value(&self) -> Value;
}

impl dyn Reflect {
    pub fn set_value<T: Any>(&mut self, value: T) -> Result<(), ()> {
        *(self as &mut dyn Any).downcast_mut().ok_or(())? = value;
        Ok(())
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref()
    }
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Primitive(PrimitiveValue),
    Struct(),
    Enum(),
    List(),
    Map(),
}

#[derive(Debug, PartialEq)]
pub enum PrimitiveValue {
    Bool(bool),
    Char(char),
    I64(i64),
    U64(u64),
    I128(i128),
    U128(u128),
    I32(i32),
    U32(u32),
    I16(i16),
    U16(u16),
    I8(i8),
    U8(u8),
    Isize(isize),
    Usize(usize),
    F32(f32),
    F64(f64),
    EmptyTuple(()),
    String(String),
}

macro_rules! impl_reflect_for_primitives {
    ($(($type:ty, $variant:ident)),+) => {
        $(
        impl From<$type> for PrimitiveValue {
            fn from(value: $type) -> Self {
                PrimitiveValue::$variant(value)
            }
        }

        impl Reflect for $type {
            fn as_value(&self) -> Value {
                Value::Primitive(PrimitiveValue::$variant(self.clone()))
            }
        }
        )+
    };
}
impl_reflect_for_primitives!(
    (i8, I8),
    (u8, U8),
    (i16, I16),
    (u16, U16),
    (i32, I32),
    (u32, U32),
    (i64, I64),
    (u64, U64),
    (i128, I128),
    (u128, U128),
    (f32, F32),
    (f64, F64),
    (bool, Bool),
    (char, Char),
    ((), EmptyTuple),
    (String, String),
    (usize, Usize),
    (isize, Isize)
);

#[cfg(test)]
mod tests {
    use super::{PrimitiveValue, Reflect, Value};

    struct Foo;

    #[test]
    fn downcast_ref() {
        let number = 42;
        let reflect_number = &number as &dyn Reflect;
        let downcast_number = reflect_number.downcast_ref::<i32>().unwrap();
        assert_eq!(downcast_number, &42i32);
    }

    #[test]
    fn as_value() {
        let mut number = 42;
        let reflect_number = &mut number as &mut dyn Reflect;

        assert_eq!(
            reflect_number.as_value(),
            Value::Primitive(PrimitiveValue::I32(42))
        );
    }

    #[test]
    fn set_value() {
        let mut number = 42;
        let reflect_number = &mut number as &mut dyn Reflect;
        reflect_number.set_value(99).unwrap();
        assert_eq!(number, 99);
    }
    #[test]
    fn set_value_wrong_type() {
        let mut number = 42;
        let reflect_number = &mut number as &mut dyn Reflect;
        reflect_number.set_value("foo").unwrap_err();
    }
}
