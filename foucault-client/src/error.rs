#[macro_export]
macro_rules! try_err {
    ($a:expr, $b:expr) => {
        match $a {
            Ok(res) => res,
            Err(err) => {
                return Ok(State::Error(ErrorStateData {
                    inner_state: Box::new($b),
                    error_message: err.to_string(),
                }))
            }
        }
    };
}
