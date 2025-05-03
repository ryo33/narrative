pub trait Value: std::any::Any + std::fmt::Debug + serde::Serialize {}

pub trait DynValue: std::any::Any + std::fmt::Debug + erased_serde::Serialize {}

impl<T: std::fmt::Debug + serde::Serialize + 'static> Value for T {}
impl<T: std::fmt::Debug + serde::Serialize + 'static> DynValue for T {}

pub struct BoxedValue(Box<dyn DynValue>);

impl BoxedValue {
    pub fn new(value: impl DynValue) -> Self {
        Self(Box::new(value))
    }
}

impl std::fmt::Debug for BoxedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl serde::Serialize for BoxedValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (&*self.0 as &dyn erased_serde::Serialize).serialize(serializer)
    }
}
