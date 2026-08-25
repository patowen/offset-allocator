macro_rules! debug {
    ($($arg:tt)*) => {
        println!($($arg)*)
    };
}

pub(crate) use debug;
