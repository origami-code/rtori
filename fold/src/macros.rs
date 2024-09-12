#[macro_export]
macro_rules! iter_partial_longest {
    (
        $variable: expr,
        $field:ident
    ) => {
        iter_partial!($variable, $field)
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
            itertools::izip!(
                $(
                    s.$field.as_ref().map_or_else(
                        // No such field
                        || itertools::Either::Right(std::iter::repeat(None)),
                        // Such a field
                        |v| itertools::Either::Left(v.iter().map(|i| Some(i)))
                    )
                ),+
            ).take_while(|($($field),+)| 
                ($($field.is_some()) ||+)
            )
        } 
    };
}
pub use iter_partial_longest;

#[macro_export]
macro_rules! iter_partial {
    (
        $variable: expr,
        $field:ident
    ) => {
        $variable.$field.as_ref().map_or_else(
            || [].iter(),
            |v| v.as_slice().iter()
        )
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
            itertools::izip!(
                $(
                    iter_partial!(s, $field)
                ),+
            )
        } 
    };
}
pub use iter_partial;

#[cfg(test)]
mod test {
#[test]
    pub fn test_partial_longest() {
        let edges = crate::EdgeInformation::default();

        let unary = iter_partial_longest!(&edges, vertices);
        let binary = iter_partial_longest!(&edges, vertices, length);
        for (left, right) in binary {

        }
    }

    #[test]
    pub fn test_partial() {
        let edges = crate::EdgeInformation::default();

        let unary = iter_partial!(&edges, vertices);
        let binary = iter_partial!(&edges, vertices, length);
        for (left, right) in binary {

        }
    }
}