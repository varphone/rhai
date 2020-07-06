//! Implement serialization support of `Dynamic` for [`serde`](https://crates.io/crates/serde).

use crate::any::Dynamic;
use crate::result::EvalAltResult;
use crate::token::Position;

#[cfg(not(feature = "no_index"))]
use crate::engine::Array;
#[cfg(not(feature = "no_object"))]
use crate::engine::Map;

use serde::ser::{
    Error, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant, Serializer,
};
use serde::Serialize;

use crate::stdlib::{any::type_name, fmt, mem};

/// Serializer for `Dynamic` which is kept as a reference.
pub struct DynamicSerializer {
    /// Buffer to hold a temporary key.
    key: Dynamic,
    /// Buffer to hold a temporary value.
    value: Dynamic,
}

impl DynamicSerializer {
    /// Create a `DynamicSerializer` from a `Dynamic` value.
    pub fn new(value: Dynamic) -> Self {
        Self {
            key: Default::default(),
            value,
        }
    }
}

/// Serialize a Rust type that implements `serde::Serialize` into a `Dynamic`.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<rhai::EvalAltResult>> {
/// # #[cfg(not(feature = "no_index"))]
/// # #[cfg(not(feature = "no_object"))]
/// # #[cfg(not(feature = "no_float"))]
/// # {
/// use rhai::{Dynamic, Array, Map, INT};
/// use rhai::ser::to_dynamic;
/// use serde::Serialize;
///
/// #[derive(Debug, serde::Serialize, PartialEq)]
/// struct Point {
///     x: f64,
///     y: f64
/// }
///
/// #[derive(Debug, serde::Serialize, PartialEq)]
/// struct MyStruct {
///     a: i64,
///     b: Vec<String>,
///     c: bool,
///     d: Point
/// }
///
/// let x = MyStruct {
///     a: 42,
///     b: vec![ "hello".into(), "world".into() ],
///     c: true,
///     d: Point { x: 123.456, y: 999.0 }
/// };
///
/// // Convert the 'MyStruct' into a 'Dynamic'
/// let value = to_dynamic(x)?;
///
/// assert!(value.is::<Map>());
///
/// let map = value.cast::<Map>();
/// let point = map.get("d").unwrap().downcast_ref::<Map>().unwrap();
/// assert_eq!(*point.get("x").unwrap().downcast_ref::<f64>().unwrap(), 123.456);
/// assert_eq!(*point.get("y").unwrap().downcast_ref::<f64>().unwrap(), 999.0);
/// # }
/// # Ok(())
/// # }
/// ```
pub fn to_dynamic<T: Serialize>(value: T) -> Result<Dynamic, Box<EvalAltResult>> {
    let mut s = DynamicSerializer::new(Default::default());
    value.serialize(&mut s)
}

impl Error for Box<EvalAltResult> {
    fn custom<T: fmt::Display>(err: T) -> Self {
        Box::new(EvalAltResult::ErrorRuntime(
            err.to_string(),
            Position::none(),
        ))
    }
}

impl Serializer for &mut DynamicSerializer {
    type Ok = Dynamic;
    type Error = Box<EvalAltResult>;
    type SerializeSeq = DynamicSerializer;
    type SerializeTuple = DynamicSerializer;
    type SerializeTupleStruct = DynamicSerializer;
    #[cfg(not(any(feature = "no_object", feature = "no_index")))]
    type SerializeTupleVariant = TupleVariantSerializer;
    #[cfg(any(feature = "no_object", feature = "no_index"))]
    type SerializeTupleVariant = serde::ser::Impossible<Dynamic, Box<EvalAltResult>>;
    type SerializeMap = DynamicSerializer;
    type SerializeStruct = DynamicSerializer;
    #[cfg(not(feature = "no_object"))]
    type SerializeStructVariant = StructVariantSerializer;
    #[cfg(feature = "no_object")]
    type SerializeStructVariant = serde::ser::Impossible<Dynamic, Box<EvalAltResult>>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Box<EvalAltResult>> {
        Ok(v.into())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "only_i32"))]
        return self.serialize_i64(i64::from(v));
        #[cfg(feature = "only_i32")]
        return self.serialize_i32(i32::from(v));
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "only_i32"))]
        return self.serialize_i64(i64::from(v));
        #[cfg(feature = "only_i32")]
        return self.serialize_i32(i32::from(v));
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "only_i32"))]
        return self.serialize_i64(i64::from(v));
        #[cfg(feature = "only_i32")]
        return Ok(v.into());
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "only_i32"))]
        return Ok(v.into());
        #[cfg(feature = "only_i32")]
        if v > i32::MAX as i64 {
            return Ok(Dynamic::from(v));
        } else {
            return self.serialize_i32(v as i32);
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "only_i32"))]
        return self.serialize_i64(i64::from(v));
        #[cfg(feature = "only_i32")]
        return self.serialize_i32(i32::from(v));
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "only_i32"))]
        return self.serialize_i64(i64::from(v));
        #[cfg(feature = "only_i32")]
        return self.serialize_i32(i32::from(v));
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "only_i32"))]
        return self.serialize_i64(i64::from(v));
        #[cfg(feature = "only_i32")]
        if v > i32::MAX as u32 {
            return Ok(Dynamic::from(v));
        } else {
            return self.serialize_i32(v as i32);
        }
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "only_i32"))]
        if v > i64::MAX as u64 {
            return Ok(Dynamic::from(v));
        } else {
            return self.serialize_i64(v as i64);
        }
        #[cfg(feature = "only_i32")]
        if v > i32::MAX as u64 {
            return Ok(Dynamic::from(v));
        } else {
            return self.serialize_i32(v as i32);
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Box<EvalAltResult>> {
        Ok(Dynamic::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Box<EvalAltResult>> {
        Ok(Dynamic::from(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Box<EvalAltResult>> {
        Ok(v.into())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Box<EvalAltResult>> {
        Ok(v.to_string().into())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Box<EvalAltResult>> {
        Ok(Dynamic::from(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Box<EvalAltResult>> {
        Ok(().into())
    }

    fn serialize_some<T: ?Sized + Serialize>(
        self,
        value: &T,
    ) -> Result<Self::Ok, Box<EvalAltResult>> {
        value.serialize(&mut *self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Box<EvalAltResult>> {
        Ok(().into())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Box<EvalAltResult>> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Box<EvalAltResult>> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Box<EvalAltResult>> {
        value.serialize(&mut *self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "no_object"))]
        {
            let content = to_dynamic(value)?;
            make_variant(variant, content)
        }
        #[cfg(feature = "no_object")]
        return Err(Box::new(EvalAltResult::ErrorMismatchOutputType(
            "Dynamic".into(),
            "map".into(),
            Position::none(),
        )));
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Box<EvalAltResult>> {
        #[cfg(not(feature = "no_index"))]
        return Ok(DynamicSerializer::new(Array::new().into()));
        #[cfg(feature = "no_index")]
        return Err(Box::new(EvalAltResult::ErrorMismatchOutputType(
            "Dynamic".into(),
            "array".into(),
            Position::none(),
        )));
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Box<EvalAltResult>> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Box<EvalAltResult>> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Box<EvalAltResult>> {
        #[cfg(not(any(feature = "no_object", feature = "no_index")))]
        return Ok(TupleVariantSerializer {
            variant,
            array: Array::with_capacity(len),
        });
        #[cfg(any(feature = "no_object", feature = "no_index"))]
        {
            #[cfg(feature = "no_object")]
            let err_type = "map";
            #[cfg(not(feature = "no_object"))]
            let err_type = "array";
            Err(Box::new(EvalAltResult::ErrorMismatchOutputType(
                "Dynamic".into(),
                err_type.into(),
                Position::none(),
            )))
        }
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Box<EvalAltResult>> {
        #[cfg(not(feature = "no_object"))]
        return Ok(DynamicSerializer::new(Map::new().into()));
        #[cfg(feature = "no_object")]
        return Err(Box::new(EvalAltResult::ErrorMismatchOutputType(
            "Dynamic".into(),
            "map".into(),
            Position::none(),
        )));
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Box<EvalAltResult>> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Box<EvalAltResult>> {
        #[cfg(not(feature = "no_object"))]
        return Ok(StructVariantSerializer {
            variant,
            map: Map::with_capacity(len),
        });
        #[cfg(feature = "no_object")]
        return Err(Box::new(EvalAltResult::ErrorMismatchOutputType(
            "Dynamic".into(),
            "map".into(),
            Position::none(),
        )));
    }
}

impl SerializeSeq for DynamicSerializer {
    type Ok = Dynamic;
    type Error = Box<EvalAltResult>;

    fn serialize_element<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Box<EvalAltResult>> {
        #[cfg(not(feature = "no_index"))]
        {
            let value = value.serialize(&mut *self)?;
            let arr = self.value.downcast_mut::<Array>().unwrap();
            arr.push(value);
            Ok(())
        }
        #[cfg(feature = "no_index")]
        unreachable!()
    }

    // Close the sequence.
    fn end(self) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "no_index"))]
        return Ok(self.value);
        #[cfg(feature = "no_index")]
        unreachable!()
    }
}

impl SerializeTuple for DynamicSerializer {
    type Ok = Dynamic;
    type Error = Box<EvalAltResult>;

    fn serialize_element<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Box<EvalAltResult>> {
        #[cfg(not(feature = "no_index"))]
        {
            let value = value.serialize(&mut *self)?;
            let arr = self.value.downcast_mut::<Array>().unwrap();
            arr.push(value);
            Ok(())
        }
        #[cfg(feature = "no_index")]
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "no_index"))]
        return Ok(self.value);
        #[cfg(feature = "no_index")]
        unreachable!()
    }
}

impl SerializeTupleStruct for DynamicSerializer {
    type Ok = Dynamic;
    type Error = Box<EvalAltResult>;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Box<EvalAltResult>> {
        #[cfg(not(feature = "no_index"))]
        {
            let value = value.serialize(&mut *self)?;
            let arr = self.value.downcast_mut::<Array>().unwrap();
            arr.push(value);
            Ok(())
        }
        #[cfg(feature = "no_index")]
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "no_index"))]
        return Ok(self.value);
        #[cfg(feature = "no_index")]
        unreachable!()
    }
}

impl SerializeMap for DynamicSerializer {
    type Ok = Dynamic;
    type Error = Box<EvalAltResult>;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Box<EvalAltResult>> {
        #[cfg(not(feature = "no_object"))]
        {
            self.key = key.serialize(&mut *self)?;
            Ok(())
        }
        #[cfg(feature = "no_object")]
        unreachable!()
    }

    fn serialize_value<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Box<EvalAltResult>> {
        #[cfg(not(feature = "no_object"))]
        {
            let key = mem::take(&mut self.key)
                .take_immutable_string()
                .map_err(|typ| {
                    Box::new(EvalAltResult::ErrorMismatchOutputType(
                        "string".into(),
                        typ.into(),
                        Position::none(),
                    ))
                })?;
            let value = value.serialize(&mut *self)?;
            let map = self.value.downcast_mut::<Map>().unwrap();
            map.insert(key, value);
            Ok(())
        }
        #[cfg(feature = "no_object")]
        unreachable!()
    }

    fn serialize_entry<K: ?Sized + Serialize, T: ?Sized + Serialize>(
        &mut self,
        key: &K,
        value: &T,
    ) -> Result<(), Box<EvalAltResult>> {
        #[cfg(not(feature = "no_object"))]
        {
            let key: Dynamic = key.serialize(&mut *self)?;
            let key = key.take_immutable_string().map_err(|typ| {
                Box::new(EvalAltResult::ErrorMismatchOutputType(
                    "string".into(),
                    typ.into(),
                    Position::none(),
                ))
            })?;
            let value = value.serialize(&mut *self)?;
            let map = self.value.downcast_mut::<Map>().unwrap();
            map.insert(key, value);
            Ok(())
        }
        #[cfg(feature = "no_object")]
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "no_object"))]
        return Ok(self.value);
        #[cfg(feature = "no_object")]
        unreachable!()
    }
}

impl SerializeStruct for DynamicSerializer {
    type Ok = Dynamic;
    type Error = Box<EvalAltResult>;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Box<EvalAltResult>> {
        #[cfg(not(feature = "no_object"))]
        {
            let value = value.serialize(&mut *self)?;
            let map = self.value.downcast_mut::<Map>().unwrap();
            map.insert(key.into(), value);
            Ok(())
        }
        #[cfg(feature = "no_object")]
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Box<EvalAltResult>> {
        #[cfg(not(feature = "no_object"))]
        return Ok(self.value);
        #[cfg(feature = "no_object")]
        unreachable!()
    }
}

#[cfg(not(any(feature = "no_object", feature = "no_index")))]
pub struct TupleVariantSerializer {
    variant: &'static str,
    array: Array,
}

#[cfg(not(any(feature = "no_object", feature = "no_index")))]
impl SerializeTupleVariant for TupleVariantSerializer {
    type Ok = Dynamic;
    type Error = Box<EvalAltResult>;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Box<EvalAltResult>> {
        let value = to_dynamic(value)?;
        self.array.push(value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Box<EvalAltResult>> {
        make_variant(self.variant, self.array.into())
    }
}

#[cfg(not(feature = "no_object"))]
pub struct StructVariantSerializer {
    variant: &'static str,
    map: Map,
}

#[cfg(not(feature = "no_object"))]
impl SerializeStructVariant for StructVariantSerializer {
    type Ok = Dynamic;
    type Error = Box<EvalAltResult>;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Box<EvalAltResult>> {
        let value = to_dynamic(value)?;
        self.map.insert(key.into(), value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Box<EvalAltResult>> {
        make_variant(self.variant, self.map.into())
    }
}

#[cfg(not(feature = "no_object"))]
fn make_variant(variant: &'static str, value: Dynamic) -> Result<Dynamic, Box<EvalAltResult>> {
    let mut map = Map::with_capacity(1);
    map.insert(variant.into(), value);
    Ok(map.into())
}
