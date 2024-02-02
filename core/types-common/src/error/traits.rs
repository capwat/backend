use either::Either;

pub trait SerializeCategory {
    fn has_data(&self) -> bool;
    fn serialize_data<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>;
}

pub trait CategoryMessage {
    fn message(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

pub trait SubcategoryMessage {
    fn message(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

pub trait DeserializeCategory: Sized {
    fn deserialize<D: serde::de::Error>(
        subcode: Option<u64>,
        data: Option<serde_json::Value>,
    ) -> Result<Either<Self, Option<serde_json::Value>>, D>;
}

pub trait Category:
    DeserializeCategory + SerializeCategory + CategoryMessage
{
    fn subcode(&self) -> Option<u64>;
    fn has_subcode(&self) -> bool;
}

impl<T: SubcategoryMessage> SubcategoryMessage for Box<T> {
    fn message(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        T::message(&*self, f)
    }
}

impl<T: CategoryMessage> CategoryMessage for Box<T> {
    fn message(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        T::message(&*self, f)
    }
}

impl<T: SerializeCategory> SerializeCategory for Box<T> {
    fn has_data(&self) -> bool {
        T::has_data(&*self)
    }

    fn serialize_data<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        T::serialize_data(&*self, serializer)
    }
}

impl<T: DeserializeCategory> DeserializeCategory for Box<T> {
    fn deserialize<D: serde::de::Error>(
        subcode: Option<u64>,
        data: Option<serde_json::Value>,
    ) -> Result<Either<Self, Option<serde_json::Value>>, D> {
        let value = T::deserialize(subcode, data)?;
        match value {
            Either::Left(inner) => Ok(Either::Left(Box::new(inner))),
            Either::Right(n) => Ok(Either::Right(n)),
        }
    }
}

impl<T: Category> Category for Box<T> {
    fn has_subcode(&self) -> bool {
        T::has_subcode(&*self)
    }

    fn subcode(&self) -> Option<u64> {
        T::subcode(&*self)
    }
}
