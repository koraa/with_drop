use with_drop::{with_drop, WithDrop};
use std::{cell::RefCell, vec::Vec};

#[test]
fn test_drop_with() {
    let drops = RefCell::new(Vec::new());

    {
        // New/drop_with
        let mut a = WithDrop::new(23, |x| drops.borrow_mut().push(x));
        let b = with_drop(32, |x| drops.borrow_mut().push(x));

        // Clone trait
        let c = a.clone();

        // Comparison trait
        assert!(a != b);
        assert!(a == c);
        assert!(a < b);
        assert!(b > a);

        // Deref
        *a += 42;
        assert!(*a == 65);

        // Pre-drop
        assert!(drops.borrow().is_empty());
    };

    assert!(*drops.borrow() == [23, 32, 65]);
}
