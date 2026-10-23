// https://github.com/rust-lang/rust/issues/164250

use std::fmt::Debug;

struct Q<T: Debug, F: FnMut(&T)>(Option<T>, F);

impl<T, F> Drop for Q<T, F>
where
    F: FnMut(&T),
    T: Debug,
    for<'a> &'a T: Debug,
{
    fn drop(&mut self) {
        // Code in here has access to a non-'static &String
        // but assumes that it's a &'static String

        // closure takes a shared ref to T -> &T;
        let fn_arg = self.0.as_ref().unwrap();
        // run closure
        self.1(fn_arg)
    }
}

fn main() -> Result<(), String> {
    let mut out: Option<&'static _> = None;
    {
        let l = vec![vec!["dropped"]];
        let mut q = Q(None, |s| {
            // deref `s` from &T to T
            out = Some(*s);
            println!("{:p}: {:?} // value in closure", &out, out);
        });

        // T = &Vec<Vec<&str>>
        q.0 = Some(&l);
    }
    println!("drop completed.");

    // when this drops, we
    // 1. `q` is initialised with its drop fn holding a pointer-ref to q.0; initially a None.
    // 2. by covariant field assignment, this now refers to a `&Vec<Vec<_>>`, and l is dropped.
    // 3. out is now pointing at the memory previously held by l.
    // 4. we can re-allocate to prevent SIGSEGV and show that we've replaced what used to be l.
    let _k = vec![vec!["never gonna give you up"]];
    println!("{:p}: {:?}", &out, out);

    Ok(())
}
