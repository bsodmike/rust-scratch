/// # Drafts
///
/// - Incorrect attempt at reasoning about PhantomData and its relationship to auto-traits (for fn() -> T).
///     - compiles: https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=701c0dadbe4d41bd8b39cef40a06f6f7
///     - panics: https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=e474c408d86604185b5933a9ae63d133
///     - re-write the above to demonstrate impact on auto-traits.

#[cfg(test)]
mod draft_1 {

    #[test]
    fn do_it() {}
}
