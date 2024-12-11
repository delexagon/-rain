
pub enum SliceOrVec<'a, T> {
    Slice(&'a [T]),
    Vec(Vec<T>)
} impl <'a, T> SliceOrVec<'a, T> {
    pub fn len(&self) {
        
    }
}
impl<'a, T> From<Vec<T>> for SliceOrVec<'a, T> {
    fn from(value: Vec<T>) -> Self {
        return Self::Vec(value);
    }
}
impl<'a, T> From<&'a T> for SliceOrVec<'a, T> {
    fn from(value: &'a T) -> Self {
        return Self::Slice(std::slice::from_ref(value));
    }
}
impl<'a, T> From<&'a [T]> for SliceOrVec<'a, T> {
    fn from(value: &'a [T]) -> Self {
        return Self::Slice(value);
    }
}
impl<'a, T> std::ops::Index<usize> for SliceOrVec<'a, T> {
    type Output = T;
    
    fn index(&self, i: usize) -> &Self::Output {
        match self {
            Self::Vec(v) => &v[i],
            Self::Slice(v) => &v[i]
        }
    }
}
impl<'a, T> IntoIterator for &'a SliceOrVec<'a, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            SliceOrVec::Vec(v) => v.as_slice().iter(),
            SliceOrVec::Slice(v) => v.iter()
        }
    }
}
