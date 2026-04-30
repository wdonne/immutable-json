use imbl::Vector;

pub(crate) fn insert<T>(vector: &Vector<T>, index: usize, value: T) -> Vector<T>
where
    T: Clone,
{
    let mut new_vec = vector.clone();

    new_vec.insert(index, value);
    new_vec
}

pub(crate) fn push_back<T>(vector: &Vector<T>, value: T) -> Vector<T>
where
    T: Clone,
{
    let mut new_vec = vector.clone();

    new_vec.push_back(value);
    new_vec
}

pub(crate) fn remove_first<T>(vector: &Vector<T>) -> Vector<T>
where
    T: Clone,
{
    let mut new_vec = vector.clone();

    new_vec.pop_front();
    new_vec
}

pub(crate) fn remove_last<T>(vector: &Vector<T>) -> Vector<T>
where
    T: Clone,
{
    let mut new_vec = vector.clone();

    new_vec.pop_back();
    new_vec
}
