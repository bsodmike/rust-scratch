use std::hash::{Hash, Hasher};
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

pub struct Uid<T> {
    inner: uuid::Uuid,
    marker: PhantomData<fn() -> T>,
}

impl<T> Clone for Uid<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Uid<T> {}

impl<T> Uid<T> {
    const fn new(inner: uuid::Uuid) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<T> From<uuid::Uuid> for Uid<T> {
    fn from(value: uuid::Uuid) -> Self {
        Self::new(value)
    }
}

impl<T> From<Uid<T>> for uuid::Uuid {
    fn from(value: Uid<T>) -> Self {
        value.inner
    }
}

impl<T> Debug for Uid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Uid").field(&self.inner).finish()
    }
}

impl<T> Display for Uid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl<T> PartialEq for Uid<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for Uid<T> {}

impl<T> Hash for Uid<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}
