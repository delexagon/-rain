
#[macro_export]
macro_rules! errstr {
    ($error:expr) => {
        format!("{}:{}; Error: {}", file!(), line!(), $error)
    }
}

#[macro_export]
macro_rules! err_plus {
    ($error:expr, $add:expr) => {
        match $error {
            Ok(x) => Ok(x),
            Err(e) => Err(format!("{}, {:?}", crate::errstr!(e), $add))
        }
    }
}

#[macro_export]
macro_rules! err {
    ($error:expr) => {
        match $error {
            Ok(x) => Ok(x),
            Err(e) => Err(crate::errstr!(e))
        }
    }
}

// FOR USE WITH RESOURCEHANDLER
#[macro_export]
macro_rules! false_if_err {
    ($expression:expr, $resources:ident) => {
        match $resources.eat(crate::err!($expression)) {
            Some(x) => x,
            _ => return false,
        }
    };
}
