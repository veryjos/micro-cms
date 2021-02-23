use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;

use serde::de;
use serde::{Deserialize, Deserializer};

pub trait FromKeyAndVal {
    fn from_key_and_val(key: &str, val: &str) -> Result<Self, Box<dyn Error>>
        where Self: Sized;

    fn set_key(&mut self, key: &str);
}

struct KeyValOrStruct<'a, T>(&'a str, PhantomData<T>);

impl<'de, T> de::DeserializeSeed<'de> for KeyValOrStruct<'de, T>
where
    T: Deserialize<'de> + FromKeyAndVal
{
    type Value = T;

    fn deserialize<D>(self, deser: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>
    {
        struct KeyValOrStructVisitor<'a, T>(&'a str, PhantomData<T>);

        impl<'de, 'a, T> de::Visitor<'de> for KeyValOrStructVisitor<'a, T>
        where
            T: Deserialize<'de> + FromKeyAndVal
        {
            type Value = T;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "str or struct")
            }

            fn visit_str<E>(self, val: &str) -> Result<Self::Value, E>
            where
                E: de::Error
            {
                // TODO: Fix this unwrap
                Ok(T::from_key_and_val(self.0, val).unwrap())
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>
            {
                let mut val: Self::Value = Deserialize::deserialize(
                    de::value::MapAccessDeserializer::new(map)
                )?;

                val.set_key(self.0);
                Ok(val)
            }
        }

        deser.deserialize_map(KeyValOrStructVisitor(self.0, PhantomData))
    }
}

pub fn keyval_map<'de, T, D>(deser: D) -> Result<HashMap<String, T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + FromKeyAndVal,
{
    struct KeyValVisitor<T> {
        marker: PhantomData<T>
    };

    impl<'de, T> de::Visitor<'de> for KeyValVisitor<T>
    where
        T: Deserialize<'de> + FromKeyAndVal
    {
        type Value = HashMap<String, T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>
        {
            let mut out = HashMap::new();

            while let Ok(Some(key)) = map.next_key::<&str>() {
                if let Ok(val) = map.next_value_seed(KeyValOrStruct(key, PhantomData)) {
                    out.insert(key.to_owned(), val);
                }
            }

            Ok(out)
        }
    }

    deser.deserialize_any(KeyValVisitor { marker: PhantomData })
}
