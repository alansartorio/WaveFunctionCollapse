use enum_map::EnumMap;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::Read;
use crate::direction::Direction;

mod rotation_parsing {
    use serde::{
        de::{Error, SeqAccess, Unexpected, Visitor},
        Deserializer,
    };

    struct MyVisitor;
    impl<'de> Visitor<'de> for MyVisitor {
        type Value = Vec<i8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str(
                "A rotation value. Either \"All\" or an array with rotations, like [0, 2].",
            )
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut vec = vec![];

            while let Some(elem) = seq.next_element()? {
                vec.push(elem);
            }

            Ok(vec)
        }

        fn visit_str<E: Error>(self, s: &str) -> Result<Self::Value, E> {
            if s == "All" {
                Ok(vec![0, 1, 2, 3])
            } else {
                Err(Error::invalid_value(Unexpected::Str(s), &self))
            }
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<i8>, D::Error> {
        d.deserialize_any(MyVisitor)
    }
}

fn default_rotation() -> Vec<i8> {
    vec![0]
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileEntry {
    pub file: String,
    pub sockets: EnumMap<Direction, String>,
    #[serde(deserialize_with = "rotation_parsing::deserialize")]
    #[serde(default = "default_rotation")]
    pub rotations: Vec<i8>,
}

pub fn parse_tiles<R: Read>(r: R) -> Result<Vec<TileEntry>, serde_json::Error> {
    serde_json::from_reader(r)
}
