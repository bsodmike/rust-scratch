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

/// Ref: https://doc.rust-lang.org/nomicon/phantom-data.html
#[cfg(test)]
mod illustrate_phantom_data_auto_traits {
    use std::marker::PhantomData;
    use std::rc::Rc;
    use std::sync::Arc;

    // A non-Send type as Rc<T> is NOT Send
    struct NotSend(Rc<u8>);

    struct CanSend(Arc<u8>);

    // Should not be Send if T is not Send
    fn assert_send<T: Send>() {}

    /// PhantomData<T> is covariant over T
    mod phantom_data_t {
        use super::*;

        struct Wrapper<T> {
            _marker: PhantomData<T>,
        }

        #[test]
        fn non_send_type_cannot_compile() {
            // error[E0277]: `Rc<u8>` cannot be sent between threads safely
            //     --> src/language_foundations/subtyping_variance.rs:136:23
            //     |
            //     136 |         assert_send::<Wrapper<NotSend>>();
            // |                       ^^^^^^^^^^^^^^^^ `Rc<u8>` cannot be sent between threads safely
            //     |
            //     = help: within `Wrapper<NotSend>`, the trait `Send` is not implemented for `Rc<u8>`
        }

        #[test]
        fn send_type_ok() {
            assert_send::<Wrapper<CanSend>>();
        }
    }

    /// PhantomData<fn() -> T> is covariant over T
    ///
    /// # Auto-traits:
    ///
    /// In the context of `Wrapper<T>`:
    /// - Implies that T must be Send and Sync for the containing type to be Send and Sync, unless T is a ZST (Zero-Sized Type).
    /// - PhantomData<fn() -> T> is covariant on fn() -> T which in turn is covariant on T. so all fields of WrapperFn<T> (there’s just one to be precise but in generalised terms) only need covariance on T, so the type (`Wrapper<T>`) is covariant on T
    ///
    /// The reason for requiring PhantomData is that it enables "by example" variance. Thus for the majority
    /// of cases, code doesn't need to directly consider variance, and can instead say
    /// - "I own T" (PhantomData<T>),
    /// - "I borrow T" (PhantomData<&[mut] T>),
    /// - "I produce T (PhantomData<fn() -> T>),
    /// - "I consume T" (PhantomData<fn(T)>), or some combination of those.
    mod phantom_data_fn_t {
        use super::*;

        struct WrapperFn<T> {
            _marker: PhantomData<fn() -> T>,
        }

        /// This is prevented as `NotSend` is not `Send`
        #[test]
        fn non_send_type_prevented() {
            assert_send::<WrapperFn<NotSend>>();
        }

        /// This is allowed for the Send type.
        #[test]
        fn send_type_ok() {
            assert_send::<WrapperFn<CanSend>>();
        }
    }
}

#[cfg(test)]
mod typed_key_example {
    use super::*;
    use std::fmt::{Debug, Display};
    use std::marker::PhantomData;
    use std::rc::Rc;

    mod ex_1 {
        use super::*;
        use std::thread;

        /// Value type for ID fields
        #[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct Id<T> {
            inner: i32,
            marker: PhantomData<fn() -> T>,
        }

        impl<T> Id<T> {
            pub const fn new(inner: i32) -> Self {
                Self {
                    inner,
                    marker: PhantomData,
                }
            }
        }

        impl<T> Display for Id<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.inner, f)
            }
        }

        type UserPayrollId = Id<UserPayroll>;

        struct UserPayroll {
            pub id: UserPayrollId,
        }

        #[test]
        fn pending() {}
    }

    /// oklyth:
    /// this is a typed key which is a common strategy for creating type safe keys. eg i might have two collections
    /// HashMap<Id<User>, User> and HashMap<Id<File>, File>, and the keys being unique types means i can't
    /// accidentally try to fetch a file using a user id the specific form of the PhantomData is to prevent autotraits
    /// from T from infecting Id itself because there's no reason that the key has to be !Send just because the type is,
    /// the key is just a number.
    ///
    /// if you had PhantomData<T>, then any !Send, !Sync, !Unpin etc properties of T would get applied to Id<T>
    ///
    /// harudagondi:
    /// my pedantic interpretation of Id<T> is that technically, Id stores a real i32 and a theoretical fn() -> T, where
    /// in the eyes of the compiler, being theoretical doesn't really make a difference. autotraits are a completely orthogonal
    /// concept, since you can opt out of auto traits without PhantomData anyways
    ///
    /// i just remember that (1) auto traits only inherit the traits from the general type (which is fn() -> T in this case), and (2) fn pointers are always Send + Sync, Copy and Unpin.
    ///
    /// so the specific form of PhantomData used (that it's a function pointer instead of T) is to avoid negative auto traits impls on T from infecting Id<T>
    /// If T is !Send this will not make Id<T> !Send, and Id<T> will not be infected by negative autotraits on T.
    mod simplified {
        use super::*;

        struct Id<T> {
            inner: i32,
            _marker: PhantomData<fn() -> T>,
        }

        impl<T> Id<T> {
            pub const fn new(inner: i32) -> Self {
                Self {
                    inner,
                    _marker: PhantomData,
                }
            }
        }

        type UserWalletId = Id<UserWallet>;

        struct UserWallet {
            pub id: UserWalletId,
        }

        #[test]
        fn do_it() {
            // NOTE ID holds a function that returns a UserWallet value.
            // function pointers are always 'static
            let id = Id::new(0_i32);
            let wallet = UserWallet { id };
        }
    }

    /// don't actually do that though, it just increases the size of your type by 8 bytes for no reason.
    /// This is to demonstrate that you can also change variance without PhantomData but it's harder than usual.
    /// you just store an fn() -> T directly with a dummy function; enforcing !Send/Sync misses the point of
    /// opting out of auto traits
    mod simplified_with_dummy {
        use super::*;

        struct Id<T> {
            inner: i32,
            _marker: fn() -> T,
        }

        fn _dummy<T>() -> T {
            unimplemented!()
        }

        impl<T> Id<T> {
            pub const fn new(inner: i32) -> Self {
                Self {
                    inner,
                    _marker: _dummy,
                }
            }
        }

        type UserWalletId = Id<UserWallet>;

        struct UserWallet {
            pub id: UserWalletId,
        }

        #[test]
        fn do_it() {
            // NOTE ID holds a function that returns a UserWallet value.
            // function pointers are always 'static
            let id = Id::new(0_i32);
            let wallet = UserWallet { id };
        }
    }
}

#[cfg(test)]
mod demonstrate_rc_send_compile_error {
    use std::rc::Rc;
    use std::thread;

    #[test]
    fn demonstrate_rc_send_compile_error() {
        // This test demonstrates that `Rc` cannot be sent across threads.
        // Uncommenting the code block below WILL cause a compile-time error.
        // This test *passes* because the problematic code is commented out.
        // The purpose is to illustrate the error, not to actually compile it here.

        let my_rc_value = Rc::new(String::from("Hello from Rc!"));

        // -------------------------------------------------------------
        // UNCOMMENT THE FOLLOWING BLOCK TO SEE THE COMPILE ERROR:
        // -------------------------------------------------------------
        /*
        let handle = thread::spawn(move || {
            // Error: `std::rc::Rc<std::string::String>` cannot be sent between threads safely
            println!("Attempting to use Rc in thread: {}", my_rc_value);
        });
        */
        // If the above line compiled, we would join the thread here.
        // handle.join().unwrap();

        // -------------------------------------------------------------

        println!("This test function demonstrates the `Rc` send error.");
        println!(
            "Please uncomment the `thread::spawn` block within this test to observe the compile error when running `cargo test` or `cargo build`."
        );

        // Assert that the test itself passes (since the error-causing code is commented)
        assert!(true);
    }

    #[test]
    fn demonstrate_arc_send_success() {
        // This test demonstrates that `Arc` can be safely sent across threads.
        use std::sync::Arc;
        use std::thread;

        let my_arc_value = Arc::new(String::from("Hello from Arc!"));

        // Clone the Arc to share ownership with the new thread
        let my_arc_value_for_thread = Arc::clone(&my_arc_value);

        let handle = thread::spawn(move || {
            // This will work because Arc is Send
            println!("Value received in thread: {}", my_arc_value_for_thread);
            assert_eq!(*my_arc_value_for_thread, "Hello from Arc!");
        });

        handle.join().unwrap();

        println!("Original Arc value outside thread: {}", my_arc_value);
        assert_eq!(*my_arc_value, "Hello from Arc!");
    }
}
