#[cfg(test)]
mod variance_phantomdata_and_drop_check {
    use std::fmt::Debug;
    use std::marker::PhantomData;

    struct TouchDrop<T: Debug>(T);

    impl<T: Debug> Drop for TouchDrop<T> {
        fn drop(&mut self) {
            // we cause a move here.
            println!("Touch drop! {:?}", self.0);
        }
    }

    struct Deserializer<T> {
        // some fields
        _t: PhantomData<T>,
    }
    struct Deserializer2<T> {
        // some fields
        _t: PhantomData<fn() -> T>,
    }
    struct Deserializer3<T> {
        // some fields
        _t: PhantomData<fn(T)>,
    }

    #[test]
    fn ex2_newtype_drop_impl_rustc_error() {
        let x = String::new();

        // this is commented out to allow tests to compile.
        // TouchDrop borrows a reference to x here.
        let z = vec![TouchDrop(&x)];

        // we cannot drop x, as it is borrowed above.
        // drop(x); // <--- this will not compile

        // we also have an implicit drop here, as it's the end of the block.  The compiler shows us
        // > borrow might be used here, when `z` is dropped and runs the `Drop` code for type `Vec`
        // DO NOT UNCOMMENT: this is only used for explanation purposes.
        // drop(z);
    }

    #[test]
    fn ex2_newtype_drop_impl() {
        let x = String::new();

        // this is commented out to allow tests to compile.
        // TouchDrop borrows a reference to x here.
        // let z = vec![TouchDrop(&x)];

        // we cannot drop x, as it is borrowed above.
        // drop(x);

        // we also have an implicit drop here, as it's the end of the block.  The compiler shows us
        // > borrow might be used here, when `z` is dropped and runs the `Drop` code for type `Vec`
        // DO NOT UNCOMMENT: this is only used for explanation purposes.
        // drop(z);

        // This is fine as vec will only drop inner types, if they impl Drop themselves, and they do so here.
        let z = vec![&x];
        drop(x);
    }

    #[test]
    fn ex1_no_problemo() {
        let x = String::new();
        let z = vec![&x];

        // x is never used beyond this point, so it's ok to drop the vector. It checks every T (in Vec<T>) and run the drop impl on each T.
        drop(x);
    }

    #[test]
    fn ex1_cannot_drop_holding_borrow() {
        let x = String::new();
        let z = vec![&x];
        drop(x);

        // this is commented out to allow tests to compile.
        // we cannot drop z as it is holding references to x.
        // drop(z);

        // error[E0505]: cannot move out of `x` because it is borrowed
        // --> src/language_foundations/subtyping_variance_jon_gjengset.rs:8:14
        // |
        // 6 |         let x = String::new();
        // |             - binding `x` declared here
        // 7 |         let z = vec![&x];
        // |                      -- borrow of `x` occurs here
        // 8 |         drop(x);
        // |              ^ move out of `x` occurs here
        // 9 |         drop(z);
        // |              - borrow later used here
    }
}

#[cfg(test)]
mod strtok {
    pub fn strtok<'a, 'b>(s: &'a mut &'b str, delimiter: char) -> &'b str {
        if let Some(i) = s.find(delimiter) {
            let prefix = &s[..i];
            let suffix = &s[(i + delimiter.len_utf8())..];
            *s = suffix;
            prefix
        } else {
            let prefix = *s;
            *s = "";
            prefix
        }
    }

    #[test]
    fn it_works() {
        let mut x = "hello rust";
        let hello = strtok(&mut x, ' ');
        assert_eq!(hello, "hello");
        assert_eq!(x, "rust");
    }
}
