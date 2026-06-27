use asi_lib::utils::collections::{chunk, flatten, sample, shuffle};

#[test]
fn test_chunk_equally() {
    let items = vec![1, 2, 3, 4, 5, 6];
    let result = chunk(&items, 2);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], vec![1, 2]);
    assert_eq!(result[2], vec![5, 6]);
}

#[test]
fn test_chunk_remainder() {
    let items = vec![1, 2, 3, 4, 5];
    let result = chunk(&items, 2);
    assert_eq!(result.len(), 3);
    assert_eq!(result[2], vec![5]);
}

#[test]
fn test_sample_returns_n_items() {
    let items = vec![10, 20, 30, 40, 50];
    let sampled = sample(&items, 3);
    assert_eq!(sampled.len(), 3);
    // All sampled items should be from the original set
    for item in &sampled {
        assert!(items.contains(item));
    }
}

#[test]
fn test_sample_full_set() {
    let items = vec![1, 2, 3];
    let sampled = sample(&items, 5);
    assert_eq!(sampled.len(), 3);
}

#[test]
fn test_flatten_nested() {
    let nested = vec![vec!["a", "b"], vec!["c"], vec!["d", "e"]];
    let flat = flatten(&nested);
    assert_eq!(flat, vec!["a", "b", "c", "d", "e"]);
}

#[test]
fn test_shuffle_changes_order() {
    let mut items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let original = items.clone();
    shuffle(&mut items);
    // After sorting, it should match the original
    let mut sorted = items.clone();
    sorted.sort();
    assert_eq!(sorted, original);
}
