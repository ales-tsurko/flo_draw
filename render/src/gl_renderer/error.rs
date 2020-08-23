use gl;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum GlError {
    /// Error without a string translation
    UnknownError(u32),

    /// Error where we can provide a string versiom
    Error(u32, String)
}

///
/// Collects OpenGL errors and panics if there are any
///
pub fn panic_on_gl_error() {
    let errors = check_for_gl_errors();

    if errors.len() > 0 {
        panic!("Unexpected OpenGL errors: {:?}", errors);
    }
}

///
/// Returns all errors that are currently set in a GL context
///
pub fn check_for_gl_errors() -> Vec<GlError> {
    let mut result = vec![];

    // Read all of ther errors that are set in the current context
    while let Some(error) = check_next_gl_error() {
        result.push(error)
    }

    result
}

///
/// Returns the next GL error
///
fn check_next_gl_error() -> Option<GlError> {
    let error = unsafe { gl::GetError() };

    match error {
        gl::NO_ERROR    => None,
        unknown         => Some(GlError::UnknownError(unknown))
    }
}
