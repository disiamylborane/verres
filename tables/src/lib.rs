//! Tables - the experiment storage part of `verres`

#![warn(missing_docs)]


macro_rules! printer {
    (
        $(#[$attr:meta])*
        $funcname:ident($thisname:pat = &$Self:ty, $contextname:ident: &$contextty:ty, $fmtname:ident)
            $code:block
    ) => {
        $(#[$attr])*
        pub fn $funcname<'repr>(&'repr self, $contextname: &'repr $contextty) -> impl core::fmt::Display + 'repr {
            struct Repr<'repr>(&'repr $Self, &'repr $contextty);

            impl core::fmt::Display for Repr<'_> {
                fn fmt(&self, $fmtname: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let Self($thisname, $contextname) = *self;
                    $code
                }
            }

            Repr(self, $contextname)
        }
    }
}


pub mod strings;
pub mod units;
