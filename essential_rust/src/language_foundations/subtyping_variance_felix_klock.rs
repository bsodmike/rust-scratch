/// YouTube: https://www.youtube.com/watch?v=fI4RG_uq-WU
/// Slide: http://pnkfx.org/presentations/rustfest-berlin-2016/slides.html
/// Source: https://github.com/pnkfelix/presentations/blob/rustfest-berlin-2016/rustfest-berlin-2016/src/slides.md
///
/// ## Subtyping in Rust
///
/// - Starts at https://youtu.be/fI4RG_uq-WU?si=Gwlb9z7yNx5A2PZi&t=1812

#[cfg(test)]
mod slide_17 {

    trait AsciiIncr {
        /// Increments `self` by one unit. (Only works for 7-bit ASCII characters though.)
        fn incr(&mut self);
    }

    impl AsciiIncr for char {
        fn incr(&mut self) {
            *self = (*self as u8 + 1) as char
        }
    }
    trait Receiver {
        fn by_ref(&self);
        fn by_mut(&mut self);
        fn by_val(self)
        where
            Self: Sized;
    }

    impl Receiver for [char; 2] {
        fn by_ref(&self) {
            println!("ref: {:?}", self[0]);
        }
        fn by_mut(&mut self) {
            println!("mut: {:?}", self[0]);
            self[1].incr();
        }
        fn by_val(mut self) {
            println!("val: {:?}", self[0]);
            self[1].incr();
        }
    }

    #[test]
    fn check_ascii_incr() {
        let mut c: char = 'a';
        c.incr();
        assert_eq!(c, 'b');
    }

    #[test]
    fn demo_obvious_cases() {
        let a = ['a', '1'];
        let b = &['b', '4'];
        let c = &mut ['c', '7'];
        println!();

        // [char; 2]        &[char; 2]           &mut [char; 2]
        a.by_val();
        b.by_ref();
        c.by_mut();
        println!("obvious: (a,b,c): {:?}", (a, b, c));
    }

    #[test]
    fn demo_interesting_cases() {
        let mut a = ['a', '1'];
        let b = &['b', '4'];
        let c = &mut ['c', '7'];
        // [char; 2]            &[char; 2]           &mut [char; 2]
        a.by_val();
        b.by_val();
        c.by_val();
        a.by_ref();
        b.by_ref();
        c.by_ref();
        a.by_mut(); /*  ...  */
        c.by_mut();

        // b.by_mut();
        // error[E0596]: cannot borrow `*b` as mutable, as it is behind a `&` reference
        //     --> src/language_foundations/subtyping_variance_felix_klock.rs:74:9
        //     |
        //     74 |         b.by_mut();
        // |         ^ `b` is a `&` reference, so the data it refers to cannot be borrowed as mutable

        println!("interesting: (a,b,c): {:?}", (a, b, c));
    }

    mod subtyping_1 {
        /// Picks either `x` or `y`, based on some internal choice.
        fn pick<'a>(x: &'a i32, y: &'static i32) -> &'a i32 {
            if *x > 0 { x } else { y }
        }

        static GLOBAL: i32 = 100;

        #[test]
        fn pick_test() {
            let temp: i32 = 200;
            let r = pick(&temp, &GLOBAL);

            assert_eq!(r, &200);
        }
    }
    mod demo_variance_and_static_ref {
        pub fn provide(m: &'static i32) {
            let val = 13;
            expect(&val, m);
        }
        fn expect<'a>(_: &'a i32, _r: &'a i32) {
            unimplemented!()
        }
    }

    #[should_panic(expected = "not implemented")]
    #[test]
    fn test_demo_variance_and_static_ref() {
        demo_variance_and_static_ref::provide(&13);
    }

    mod demo_variance_and_static_ref_hof {
        // FIXME: example in the talk does fn prov_hof(f: fn(&usize) -> &'static i32)
        // This seems wrong though.
        pub fn prov_hof(f: fn(&usize) -> &'static i32) {
            let val = 13;
            exp_hof(&val, f);
        }
        fn exp_hof<'a>(_: &'a i32, _f: fn(&'a usize) -> &'a i32) {
            unimplemented!()
        }
    }

    #[should_panic(expected = "not implemented")]
    #[test]
    fn test_demo_variance_and_static_ref_hof() {
        fn operate_on<F>(f: F, val: &usize) -> F
        where
            F: Fn(&usize) -> &'static i32,
        {
            f
        }

        // Rust promotes constexpr values to static variables implicitly, so &0 creates a &'static i32 to a static variable
        let f_ptr: fn(&usize) -> &'static i32 = |_| &42;
        demo_variance_and_static_ref_hof::prov_hof(f_ptr);
    }
}
