use std::any::Any;

pub trait Reflect: Any {
    fn item_by_index(&self, index: usize) -> Option<Box<dyn Reflect>> {
        None
    }
    fn item_len(&self) -> usize {
        0
    }
    fn as_value(&self) -> Value;
    fn set_value(&mut self, value: Value) -> Result<(), ()>;
    //    fn type_info(&self) -> &TypeInfo;
    fn duplicate(&self) -> Box<dyn Reflect>;
}

impl dyn Reflect {
    pub fn assign<T: Any>(&mut self, value: T) -> Result<(), ()> {
        *(self as &mut dyn Any).downcast_mut().ok_or(())? = value;
        Ok(())
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref()
    }
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        (self as &mut dyn Any).downcast_mut()
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

pub enum TypeInfo {
    //Struct(StructInfo),
    Primitive(PrimitiveTypeInfo),
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

            fn set_value(&mut self, value: Value) -> Result<(), ()> {
                match value {
                    Value::Primitive(PrimitiveValue::$variant(new_value)) => {
                        *self = new_value;
                        Ok(())
                    }
                    _ => {
                    Err(())
                    }
                }
            }

            fn duplicate(&self) -> Box<dyn Reflect> {
                Box::new(self.clone())
            }
        }
        )+

            #[derive(Debug, PartialEq)]
    pub enum PrimitiveValue {
            $($variant($type),
    )+
        }
            #[derive(Debug, PartialEq)]
    pub enum PrimitiveTypeInfo {
            $($variant,
    )+
        }
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
    fn assign() {
        let mut number = 42;
        let reflect_number = &mut number as &mut dyn Reflect;
        reflect_number.assign(99).unwrap();
        assert_eq!(number, 99);
    }
    #[test]
    fn assign_wrong_type() {
        let mut number = 42;
        let reflect_number = &mut number as &mut dyn Reflect;
        reflect_number.assign("foo").unwrap_err();
    }

    #[test]
    fn set_value() {
        let mut number = 42;
        let reflect_number = &mut number as &mut dyn Reflect;

        reflect_number
            .set_value(Value::Primitive(PrimitiveValue::I32(99)))
            .unwrap();
        assert_eq!(number, 99i32);
    }
}
