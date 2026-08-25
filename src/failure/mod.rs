//! The error vocabulary every client shares.
//!
//! This is not one of the six things 0003 names. That record places the mapping
//! of every failure onto one vocabulary inside "reaching a server", and every
//! other module here produces failures too, so the vocabulary sits beside the six
//! rather than inside one of them. The records are 0004 and 0037, and the issues
//! are #4 and #37.
//!
//! 0037 requires one point at which a failure becomes a kind, with nothing
//! falling through to a default. On the chosen means that is a refusal rather
//! than a convention: a value of the set is built inside this module and the
//! compiler refuses construction anywhere else. 0011 carries the measurement that
//! says so.
