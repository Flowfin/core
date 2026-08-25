//! Producing measurements.
//!
//! 0003 puts named spans, their values, the spread across repeated runs, and a
//! statement of what a run did not measure inside the core. The records are
//! 0008, 0061 and 0064, and the issues are #61 through #67.
//!
//! 0061 refused a tracing library and 0064 names the two numbers the core does
//! not report. Both are reasons this module exists as the core's own facility
//! rather than as a seam onto somebody else's.
