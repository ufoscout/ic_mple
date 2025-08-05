/*
This internal module contains the style and terminal writing implementation.

Its public API is available when the `termcolor` crate is available.
The terminal printing is shimmed when the `termcolor` crate is not available.
*/

mod imp;

pub(in crate::fmt) use self::imp::*;
