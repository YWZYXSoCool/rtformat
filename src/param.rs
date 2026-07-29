use crate::arg::FormatArg;

/// All arguments of one `format` call, handed to the visitor one by one in
/// declaration order.
///
/// The reference lifetime is tied to `&self`, so callers can collect the
/// `&dyn FormatArg` values and reuse them (the same argument may be
/// referenced by multiple placeholders with different specs).
pub trait FormatParam {
    fn visit<'fmt>(&'fmt self, visitor: &mut dyn FnMut(&'fmt dyn FormatArg));
}

impl FormatParam for () {
    fn visit<'fmt>(&'fmt self, _visitor: &mut dyn FnMut(&'fmt dyn FormatArg)) {}
}

macro_rules! impl_format_param {
   ($($idx:tt $T:ident),+ $(,)?) => {
        impl<$($T: FormatArg),+> FormatParam for ($($T,)+) {
            fn visit<'fmt>(&'fmt self, visit: &mut dyn FnMut(&'fmt dyn FormatArg)) {
                $(visit(&self.$idx);)+
            }
        }
    };
}

impl_format_param!(0 A);
impl_format_param!(0 A, 1 B);
impl_format_param!(0 A, 1 B, 2 C);
impl_format_param!(0 A, 1 B, 2 C, 3 D);
impl_format_param!(0 A, 1 B, 2 C, 3 D, 4 E);
impl_format_param!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F);
impl_format_param!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G);
impl_format_param!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H);
impl_format_param!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I);
impl_format_param!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J);
impl_format_param!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K);
impl_format_param!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L);
impl_format_param!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M);
