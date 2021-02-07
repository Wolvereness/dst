use crate::prelude::*;
use super::Handle;

impl<
    VL: PartialEq<VR>, TL: PartialEq<TR> + ?Sized,
    VR, TR: ?Sized,
> PartialEq<Handle<VR, TR>> for Handle<VL, TL>
{
    fn eq(&self, other: &Handle<VR, TR>) -> bool {
        self.value == other.value
            && self.tail == other.tail
    }
}

impl<V: Eq, T: Eq + ?Sized> Eq for Handle<V, T> {}

impl<V: Hash, T: Hash + ?Sized> Hash for Handle<V, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.tail.hash(state);
    }
}

impl<V: Debug, T: Debug + ?Sized> Debug for Handle<V, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f
            .debug_struct("Handle")
            .field("value", &self.value)
            .field("tail", &&self.tail)
            .finish()
    }
}

impl<V, T: ?Sized> AsRef<T> for Handle<V, T> {
    #[inline(always)]
    fn as_ref(&self) -> &T {
        &self.tail
    }
}

impl<V, T: ?Sized> AsMut<T> for Handle<V, T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        &mut self.tail
    }
}
