pub use itertools::{izip, Either};

/// Creates an iterator that goes through the given fields
/// in lockstep, yielding
/// `(Option<T1>, Option<T2>, Option<T3>, ..., Option<TN>)`
/// when called on the fields of type
/// `(Option<Vec<T1>>, Option<Vec<T2>>, Option<Vec<T3>>, ..., Option<Vec<TN>>)`.
///
/// The iterator returned by this macro yields
/// `Option<T>` for a field of type `Option<Vec<T>>`.
/// The value is `Option::None` if that field is `None`, otherwise it is `Option::Some`.
///
/// If every field is `None`, it doesn't iterate, otherwise it iterated up to the length of the fields
/// (as every field should be either None or be of the same length).
/// The special case of the single-field iteration still yieds a `Some` for coherence.
///
/// See [iter_partial] for a macro that doesn't return `Option<>` but that required
/// its constituent fields to all be `Some`.
#[macro_export]
macro_rules! iter_partial_optional {
    (
        $variable: expr,
        $field:ident
    ) => {
        {
            use $crate::collections::AsSlice;
            $variable.$field.as_slice().into_iter().map(|x| Some(x))
        }
    };

    (
        $variable: expr,
        $($field:ident),+
    ) => {
        {
            // So as to not execute the expression more than once ?
            let s = $variable;

            // Otherwise we zip them, taking care of a field value of None
            // being transformed in an infinite None so that it doesn't stop the iteration
            // for those field with a Some
            $crate::macros::izip!(
                $(
                    s.$field.map_or_else(
                        // No such field
                        || $crate::macros::Either::Right(core::iter::repeat(None)),
                        // Such a field
                        |v| $crate::macros::Either::Left(v.into_iter().map(|i| Some(i)))
                    )
                ),+
            ).take_while(|($($field),+)|
                ($($field.is_some()) ||+)
            )
        }
    };
}
pub use iter_partial_optional;

/// Creates an iterator that goes through the given fields
/// in lockstep, yielding
/// `(T1, T2, T3, ..., TN)`
/// when called on the fields of type
/// `(Option<Vec<T1>>, Option<Vec<T2>>, Option<Vec<T3>>, ..., Option<Vec<TN>>)`.
///
/// This macro returns an `Option::<impl Iterator<Item=(T1, T2, T3, ..., TN)>>::Some`
/// when every specified field contains a `Option::Some`, or when every field contains a `Option::None`.
/// (if every field contains a `None` then the iterator returned is zero-sized).
///
/// As a special case, when called on a single field, the iterator yields a single value instead of a tuple.
///
/// See [iter_partial_optional] for a macro that always returns an iterator,
/// which however yield tuple of `Option`s instead when the specified fields are `None`.
#[macro_export]
macro_rules! iter_partial {
    (
        $variable: expr,
        $($field:ident),*
    ) => {
        {
            use $crate::collections::AsSlice;

            // So as to not execute the expression more than once ?
            let s = $variable;

            if (
                ($(
                    !s.$field.is_empty()
                )&&+) || ($(
                    !s.$field.is_empty()
                )&&+)
            ) {
                Some($crate::macros::izip!(
                    $(
                        $variable.$field.as_slice().into_iter()
                    ),+
                ))
            } else {
                None
            }
        }
    };
}
pub use iter_partial;

/// Creates an iterator that goes through the given fields
/// in lockstep, yielding
/// `(T1, <Optional ? >T2, T3, ..., TN)`
/// when called on the fields of type
/// `(Option<Vec<T1>>, Option<Vec<T2>>, Option<Vec<T3>>, ..., Option<Vec<TN>>)`.
///
/// It is a merge of [iter_partial] and [iter_partial_optional]
#[macro_export]
macro_rules! iter {
    (
        $variable: expr,
        required (
            $($required_field:ident),+
        ),
        optional (
            $($optional_field:ident),*
        )
    ) => {
        {
            use $crate::collections::AsSlice;

            // So as to not execute the expression more than once ?
            let s = $variable;

            if (
                ($(
                    !s.$required_field.is_empty()
                )&&+) || ($(
                    !s.$required_field.is_empty()
                )&&+)
            ) {
                Some($crate::macros::izip!(
                    $(
                        s.$required_field.as_slice().into_iter()
                    ),*,
                    $(
                        s.$optional_field.map_or_else(
                            // No such field
                            || $crate::macros::Either::Right(core::iter::repeat(None)),
                            // Such a field
                            |v| $crate::macros::Either::Left(v.into_iter().map(|i| Some(i)))
                        )
                    ),*
                ))
            } else {
                None
            }
        }
    };
}
pub use iter;

#[cfg(test)]
mod test {
    #[test]
    pub fn test_partial_optional_nu() {
        let edges = crate::EdgeInformation::<alloc::alloc::Global>::default();

        let unary = iter_partial_optional!(&edges, faces);
        for (_faces) in unary {}

        let binary = iter_partial_optional!(&edges, faces, length);
        for (_left, _right) in binary {}
    }

    #[test]
    pub fn test_partial_optional() {
        let edges = crate::EdgeInformation::<alloc::alloc::Global>::default();

        let unary = iter_partial_optional!(&edges, vertices);
        for _vertex in unary {}

        let binary = iter_partial_optional!(&edges, vertices, length);
        for (_left, _right) in binary {}
    }

    #[test]
    pub fn test_partial_nu() {
        let edges = crate::EdgeInformation::<alloc::alloc::Global>::default();

        let unary = iter_partial!(&edges, faces);
        for (_faces) in unary.unwrap() {}
    }

    #[test]
    pub fn test_partial() {
        let edges = crate::EdgeInformation::<alloc::alloc::Global>::default();

        let unary = iter_partial!(&edges, vertices);
        for (_vertex) in unary.unwrap() {}

        let binary = iter_partial!(&edges, vertices, length);
        for (_left, _right) in binary.unwrap() {}
    }

    #[test]
    pub fn test_iter_nu() {
        let edges = crate::EdgeInformation::<alloc::alloc::Global>::default();

        let unary_req = iter!(&edges, required(faces), optional());
        for (_faces) in unary_req.unwrap() {}
    }

    #[test]
    pub fn test_iter() {
        let edges = crate::EdgeInformation::<alloc::alloc::Global>::default();

        let unary_req = iter!(&edges, required(vertices), optional());
        for (_vertex) in unary_req.unwrap() {}

        let multi = iter!(
            &edges,
            required(vertices),
            optional(length, assignment, faces)
        );
        for (_vertex, _length, _assigment, _faces) in multi.unwrap() {}
    }
}
