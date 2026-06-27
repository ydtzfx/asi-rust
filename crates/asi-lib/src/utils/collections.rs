use std::collections::HashMap;
use std::hash::Hash;

pub fn shuffle<T>(items: &mut [T]) {
    fastrand::shuffle(items);
}

pub fn sample<T: Clone>(items: &[T], n: usize) -> Vec<T> {
    if n >= items.len() {
        return items.to_vec();
    }
    let mut indices: Vec<usize> = (0..items.len()).collect();
    fastrand::shuffle(&mut indices);
    indices[..n].iter().map(|&i| items[i].clone()).collect()
}

pub fn chunk<T: Clone>(items: &[T], size: usize) -> Vec<Vec<T>> {
    items.chunks(size).map(|c| c.to_vec()).collect()
}

pub fn flatten<T: Clone>(nested: &[Vec<T>]) -> Vec<T> {
    nested.iter().flat_map(|v| v.iter().cloned()).collect()
}

pub fn uniq<T: Eq + Hash + Clone>(items: &[T]) -> Vec<T> {
    let mut seen = std::collections::HashSet::new();
    items
        .iter()
        .filter(|item| seen.insert((*item).clone()))
        .cloned()
        .collect()
}

pub fn group_by<T, K, F>(items: &[T], key_fn: F) -> HashMap<K, Vec<T>>
where
    T: Clone,
    K: Eq + Hash,
    F: Fn(&T) -> K,
{
    let mut map: HashMap<K, Vec<T>> = HashMap::new();
    for item in items {
        map.entry(key_fn(item)).or_default().push(item.clone());
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniq() {
        assert_eq!(uniq(&[1, 2, 2, 3, 1, 4]), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_group_by_even_odd() {
        let result = group_by(&[1, 2, 3, 4, 5], |x| x % 2);
        assert_eq!(result.get(&0).unwrap(), &vec![2, 4]);
        assert_eq!(result.get(&1).unwrap(), &vec![1, 3, 5]);
    }

    #[test]
    fn test_flatten() {
        let nested = vec![vec![1, 2], vec![3], vec![4, 5, 6]];
        assert_eq!(flatten(&nested), vec![1, 2, 3, 4, 5, 6]);
    }
}
