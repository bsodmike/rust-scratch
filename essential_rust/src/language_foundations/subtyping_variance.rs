/// # Notes
///
/// - https://stackoverflow.com/a/35394549/449342
///     - concrete examples included.
/// - https://stackoverflow.com/a/32172407/449342
///     - considers complexities around a pointer pointer such as &'x mut &'a mut i32 (and all the resulting shenanigans).

#[cfg(test)]
mod illustrate_covariance {
    use std::marker::PhantomData;

    #[test]
    fn ex_1() {
        fn whatever<'short>() {
            // Given F<T> we can say &'a T is covariant over 'a.
            //
            // The Rustonomicon specifies it is ok to treat &'a T as a subtype of &'b T
            // if 'a <: 'b (Sub <: Super).
            //
            // For the purpose of variance, PhantomData<T> does exactly the same thing as PhantomData<fn() -> T>
            // They only differ in how they affect auto traits.
            struct Foo<T> {
                inner: f64,
                marker: PhantomData<fn() -> T>,
            }
            impl<T> Foo<T> {
                const fn new(value: f64) -> Self {
                    Self {
                        inner: value,
                        marker: PhantomData,
                    }
                }
            }

            // Sub = &'static str
            // Note that there isn't actually a &str anywhere.
            // Structs inner record field is a f64. This is unrelated to this example.
            let foo: Foo<&'static str> = Foo::new(0_f64);

            // Pass it with a shorter lifetime such that Foo<T> is covariant over T,

            // Note that the 'short lifetime came from the surrounding function definition
            // The where clause to specify the lifetime in takes_foo::<'short>().
            fn takes_foo<'a>(_: Foo<&'a str>)
            where
                // NOTE: this is a fix against RA/RR rewriting this. The rust compiler if just fine with 'a:,
                'a: 'a,
            {
            }
            takes_foo::<'short>(foo);
        }
    }

    #[test]
    fn ex_1_lifetimes() {
        // 'longer: 'shorter specifies that longer completely outlives 'shorter
        // 'longer is a subtype of 'shorter
        // 'longer can be a 'shorter but a 'shorter cannot be a 'longer
        fn symptomatic<'shorter, 'longer: 'shorter>(x: &'longer mut u32, y: &'shorter u32) {
            // x = y; // will not type check
        }
    }

    #[test]
    fn shared_mut_invariance_over_t() {
        struct Thing;

        fn foo<'a, 'b>(x: &'a mut &'static Thing) {
            // let _y: &'a mut &'b Thing = x;
        }

        let thing = &mut &Thing {};

        // foo(thing);

        // error: lifetime may not live long enough
        //     --> src/language_foundations/subtyping_variance.rs:63:41
        //     |
        //     62 |         fn foo<'a, 'b>(x: &'a mut &'static Thing) {
        //     |                    -- lifetime `'b` defined here
        //     63 |             let _y: &'a mut &'b Thing = x;
        //     |                                         ^ assignment requires that `'b` must outlive `'static`
        //     |
        //     = note: requirement occurs because of a mutable reference to `&Thing`
        //     = note: mutable references are invariant over their type parameter
        //     = help: see <https://doc.rust-lang.org/nomicon/subtyping.html> for more information about variance
    }

    // Ref: https://www.youtube.com/watch?v=iVYWDIW71jk
    #[cfg(FALSE_CONST_DO_NOT_COMPILE)]
    #[test]
    fn jonhoo_shared_mut_invariance_over_t() {
        fn foo<'a>(s: &mut &'a str, x: &'a str) {
            *s = x;
        }

        // we cannot downgrade from 'static to 'a (where x is a static string reference)
        let mut x: &'static str = "invariance is hard";

        // this is a local thing
        let z = String::new();

        // &z: is a non-static string
        foo(&mut x, &z);
        // trying to reconcile, that these are the same - they are not:
        //  // foo(&mut &'a         str, &'a str)
        //  // foo(&mut &'static    str, &'a str)
        drop(z);

        // This is because mutable references are invariant over their type parameter
        // &mut T is invariant over T

        // If we did not have invariance for mutable references, you could downgrade a mutable reference
        // to something that is a reference to something less useful, stick in something less useful but
        // in the outer scope the thing you took a mutable reference TO still has the more useful type.
        // And so you now have something that still has the more useful type and so now you have something
        // that thinks it's a more useful type, but it's actually a less useful type.
        //
        // This is not okay because it doesn't have some of the properties that it's type indicates it has.
    }
}

#[cfg(test)]
mod illustrate_phantom_fn_ptrs {
    use std::marker::PhantomData;
    use std::rc::Rc; // Rc<T> is NOT Send

    // A non-Send type
    struct NotSend(Rc<u8>);

    // Should not be Send if T is not Send
    fn assert_send<T: Send>() {}

    struct Wrapper<T> {
        _marker: PhantomData<T>,
    }

    #[test]
    fn send_not_allowed() {
        // error[E0277]: `Rc<u8>` cannot be sent between threads safely
        //     --> src/language_foundations/subtyping_variance.rs:136:23
        //     |
        //     136 |         assert_send::<Wrapper<NotSend>>();
        // |                       ^^^^^^^^^^^^^^^^ `Rc<u8>` cannot be sent between threads safely
        //     |
        //     = help: within `Wrapper<NotSend>`, the trait `Send` is not implemented for `Rc<u8>`
    }

    // PhantomData<fn() -> T> is covariant over T and also prevents auto traits like Send and Sync from being derived if T is not Send or Sync.

    struct WrapperFn<T> {
        _marker: PhantomData<fn() -> T>,
    }

    #[test]
    fn block_send() {
        // This is correctly blocked as `NotSend` is not `Send`
        assert_send::<WrapperFn<NotSend>>();
    }

    struct WrapperPtr<T> {
        _marker: PhantomData<*const T>,
    }

    #[test]
    fn should_compile() {
        // assert_send::<WrapperPtr<NotSend>>();

        // error[E0277]: `*const NotSend` cannot be sent between threads safely
        //     --> src/language_foundations/subtyping_variance.rs:147:23
        //     |
        //     147 |         assert_send::<WrapperPtr<NotSend>>();
        // |                       ^^^^^^^^^^^^^^^^^^^ `*const NotSend` cannot be sent between threads safely
        //     |
        //     = help: within `WrapperPtr<NotSend>`, the trait `Send` is not implemented for `*const NotSend`
        // note: required because it appears within the type `PhantomData<*const NotSend>`
    }
}
