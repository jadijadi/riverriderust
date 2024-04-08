/// A type that stores a default value and can be restored to  default at anytime.
pub struct Restorable<T>
where
    T: Clone,
{
    pub value: T,
    solid_value: T,
}

impl<T> Restorable<T>
where
    T: Clone,
{
    pub fn restore(&mut self) {
        self.value = self.solid_value.clone();
    }

    #[allow(dead_code)]
    pub fn value(&self) -> &T {
        &self.value
    }

    #[allow(dead_code)]
    pub fn solid_value(&self) -> &T {
        &self.solid_value
    }
}

impl<T> From<T> for Restorable<T>
where
    T: Clone,
{
    fn from(value: T) -> Self {
        Self::new(value.clone(), value)
    }
}

impl<T> Restorable<T>
where
    T: Clone,
{
    pub fn new(solid_value: T, floating_value: T) -> Self {
        Self {
            solid_value,
            value: floating_value,
        }
    }
}

impl<T: std::ops::Deref> std::ops::Deref for Restorable<T>
where
    T: Clone,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: std::ops::Deref> std::ops::DerefMut for Restorable<T>
where
    T: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
