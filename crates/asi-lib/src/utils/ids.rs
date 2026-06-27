use uuid::Uuid;

pub fn new_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub fn nanoid(len: usize) -> String {
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    (0..len).map(|_| chars[rand::random::<usize>() % chars.len()]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_is_unique() {
        let a = new_uuid();
        let b = new_uuid();
        assert_ne!(a, b);
        assert_eq!(a.len(), 36);
    }

    #[test]
    fn test_nanoid_length() {
        assert_eq!(nanoid(8).len(), 8);
        assert_eq!(nanoid(21).len(), 21);
    }

    #[test]
    fn test_nanoid_alphanumeric() {
        let id = nanoid(100);
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
