trait Array {
    type Elem;

    const LEN: usize;

    fn as_slice (self: &'_ Self)
      -> &'_ [Self::Elem]
    ;

    fn as_slice_mut (self: &'_ mut Self)
      -> &'_ mut [Self::Elem]
    ;
}

#[::inline_proc_macros::macro_use]
mod proc_macros {}

impl_arrays!(0 .. 33);

const _: () = {
    fn check<__ : Array> ()
    {}

    fn with<T> ()
    {
        let _ = check::<[T; 0]>;
        let _ = check::<[T; 1]>;
        let _ = check::<[T; 2]>;
        let _ = check::<[T; 3]>;
        let _ = check::<[T; 4]>;
        let _ = check::<[T; 5]>;
    }
};
