#[cfg(feature = "tracing")]
#[allow(unused_imports)]
pub use tracing::{
    debug, debug_span, error, error_span, info, info_span, trace, trace_span, warn, warn_span,
};

#[cfg(not(feature = "tracing"))]
#[allow(unused_macros)]
mod noop {
    macro_rules! debug {
        ($($tt:tt)*) => {
            ()
        };
    }

    macro_rules! error {
        ($($tt:tt)*) => {
            ()
        };
    }

    macro_rules! info {
        ($($tt:tt)*) => {
            ()
        };
    }

    macro_rules! trace {
        ($($tt:tt)*) => {
            ()
        };
    }

    macro_rules! _warn {
        ($($tt:tt)*) => {
            ()
        };
    }

    macro_rules! trace_span {
        ($($tt:tt)*) => {{
            struct DummySpan;
            impl DummySpan {
                pub fn entered(&self) -> &Self {
                    self
                }
            }
            DummySpan
        }};
    }
    macro_rules! debug_span {
        ($($tt:tt)*) => {{
            struct DummySpan;
            impl DummySpan {
                pub fn entered(&self) -> &Self {
                    self
                }
            }
            DummySpan
        }};
    }
    macro_rules! info_span {
        ($($tt:tt)*) => {{
            struct DummySpan;
            impl DummySpan {
                pub fn entered(&self) -> &Self {
                    self
                }
            }
            DummySpan
        }};
    }
    macro_rules! warn_span {
        ($($tt:tt)*) => {{
            struct DummySpan;
            impl DummySpan {
                pub fn entered(&self) -> &Self {
                    self
                }
            }
            DummySpan
        }};
    }
    macro_rules! error_span {
        ($($tt:tt)*) => {{
            struct DummySpan;
            impl DummySpan {
                pub fn entered(&self) -> &Self {
                    self
                }
            }
            DummySpan
        }};
    }

    pub(crate) use {
        _warn as warn, debug, debug_span, error, error_span, info, info_span, trace, trace_span,
        warn_span,
    };
}

#[cfg(not(feature = "tracing"))]
#[allow(unused_imports)]
pub(crate) use noop::{
    debug, debug_span, error, error_span, info, info_span, trace, trace_span, warn, warn_span,
};
