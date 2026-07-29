use std::ffi::c_void;

pub mod self_test;

#[link(name = "orb_slam2_vendored", kind = "static")]
extern "C" {
    fn orb_slam2_init() -> *mut c_void;
    fn orb_slam2_track(handle: *mut c_void, img_data: *const u8, len: usize) -> i32;
    fn orb_slam2_get_pose(handle: *mut c_void, x: *mut f64, y: *mut f64, z: *mut f64);
    fn orb_slam2_destroy(handle: *mut c_void);
}

#[derive(Debug)]
pub enum SlamError {
    TrackFailed,
    InvalidHandle,
}

pub struct Handle {
    ptr: *mut c_void,
}

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: The pointer was allocated by C++ and is valid. We call the C++ destructor to free the memory.
            unsafe { orb_slam2_destroy(self.ptr) };
        }
    }
}

pub fn slam_init() -> Handle {
    // SAFETY: FFI call to C++ to instantiate a new ORB-SLAM2 system.
    let ptr = unsafe { orb_slam2_init() };
    Handle { ptr }
}

pub fn slam_track(handle: &mut Handle, image_data: &[u8]) -> Result<(), SlamError> {
    if handle.ptr.is_null() {
        return Err(SlamError::InvalidHandle);
    }
    // SAFETY: We pass a valid raw pointer to the C++ track function. The length of the slice is also passed to prevent OOB access.
    let res = unsafe { orb_slam2_track(handle.ptr, image_data.as_ptr(), image_data.len()) };
    if res == 0 {
        Ok(())
    } else {
        Err(SlamError::TrackFailed)
    }
}

pub fn slam_get_pose(handle: &Handle) -> (f64, f64, f64) {
    if handle.ptr.is_null() {
        return (0.0, 0.0, 0.0);
    }
    let mut x = 0.0;
    let mut y = 0.0;
    let mut z = 0.0;
    // SAFETY: Passing valid mutable references to floats to receive the output from C++.
    unsafe {
        orb_slam2_get_pose(handle.ptr, &mut x, &mut y, &mut z);
    }
    (x, y, z)
}
