use std::os::raw::{c_int, c_void};

extern "C" {
    fn zmq_version(major: *mut i32, minor: *mut i32, patch: *mut i32)
        -> c_void;
}

pub fn version() -> (i32, i32, i32) {
    let mut major = 0;
    let mut minor = 0;
    let mut patch = 0;
    unsafe {
        zmq_version(
            &mut major as *mut c_int,
            &mut minor as *mut c_int,
            &mut patch as *mut c_int,
        );
    }

    (major, minor, patch)
}

#[cfg(test)]
mod test {
    use super::version;

    #[test]
    fn version_works() {
        let version = version();
        println!("{:?}", version);
        assert_eq!(version, (4, 3, 4));
    }
}
