#[cfg(test)]
mod illustrate_covariance {
    use std::marker::PhantomData;

    #[test]
    fn ex_1() {
        fn whatever<'short>() {
            // By specifying PhantomData<fn() -> T> instead of PhantomData<T>, given F<T> we can
            // say &'a T is covariant over 'a.
            //
            // The Rustonomicon specifies it is ok to treat &'a T as a subtype of &'b T
            // if 'a <: 'b (Sub <: Super).
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
}
