use std::collections::HashSet;

pub fn merge_messages<T: std::hash::Hash + std::cmp::Eq + Clone>(a: Vec<T>, b: Vec<T>) -> Vec<T> {
    HashSet::<T>::from_iter(a)
        .union(&HashSet::from_iter(b))
        .into_iter()
        .map(|x| (*x).clone())
        .collect::<Vec<T>>()
}
