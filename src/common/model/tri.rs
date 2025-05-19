use serde::Serialize;

#[derive(Clone, PartialEq, Default)]
pub enum TriState<T> {
    #[default]
    None,
    Null,
    Some(T),
}

impl<T> TriState<T> {
    // pub fn is_some(&self) -> bool {
    //     matches!(self, TriState::Some(_))
    // }

    // pub fn is_null(&self) -> bool {
    //     matches!(self, TriState::Null)
    // }

    #[inline(always)]
    pub const fn is_none(&self) -> bool {
        matches!(*self, TriState::None)
    }
}

impl<T> Serialize for TriState<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            TriState::None => serializer.serialize_none(),
            TriState::Null => serializer.serialize_unit(),
            TriState::Some(value) => value.serialize(serializer),
        }
    }
}

// impl<'de, T> Deserialize<'de> for TriState<T>
// where
//     T: Deserialize<'de>,
// {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let opt = Option::<T>::deserialize(deserializer);

//         match opt {
//             Ok(Some(value)) => Ok(TriState::Some(value)),
//             Ok(None) => Ok(TriState::Null),
//             Err(_) => Ok(TriState::None),
//         }
//     }
// }

// impl<T> From<Option<T>> for TriState<T> {
//     fn from(option: Option<T>) -> Self {
//         match option {
//             Some(value) => TriState::Some(value),
//             None => TriState::Null,
//         }
//     }
// }
