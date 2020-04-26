use gsasl_sys::*;
use std::fmt;
use std::ffi::CStr;

pub type Result<T> = std::result::Result<T, SaslError>;

#[derive(Debug)]
pub struct SaslError(pub libc::c_int);

impl fmt::Display for SaslError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", gsasl_err_to_str(self.0))
    }
}

/// Convert an error code to a human readable description of that error
pub const fn gsasl_err_to_str(err: libc::c_int) -> &'static str {
    // gsasl returns the normal zero-terminated string
    let ptr = unsafe { gsasl_strerror(err) };
    let cstr = CStr::from_ptr(ptr);
    // Yes, this could potentially fail. But we're talking about an array of static, compiled-in
    // strings here. If they aren't UTF-8 that's clearly a bug.
    cstr.to_str().expect("GSASL library contains bad UTF-8 error descriptions")
}

/// Convert an error code to the human readable name of that error.
/// i.e. gsasl_errname_to_str(GSASL_OK) -> "GSASL_OK"
pub const fn gsasl_errname_to_str(err: libc::c_int) -> &'static str {
    // gsasl returns the normal zero-terminated string
    let ptr = unsafe { gsasl_strerror_name(err) };
    let cstr = CStr::from_ptr(ptr);
    // Yes, this could potentially fail. But we're talking about an array of static, compiled-in
    // strings here. If they aren't UTF-8 that's clearly a bug.
    cstr.to_str().expect("GSASL library contians bad UTF-8 error names")
}
