use std::os::raw::{c_double};

extern "C" {
    // FFI to C++ TrackStereo function
    pub fn TrackStereo(timestamp: c_double, out_pose: *mut c_double);
}

pub fn fetch_slam_pose(timestamp: f64) -> Result<[f64; 6], String> {
    let mut pose = [0.0f64; 6];
    
    // Safety: we trust the C++ library writes exactly 6 doubles.
    // In production, we'd use libloading to load the dynamic library (.dll/.so)
    unsafe {
        TrackStereo(timestamp, pose.as_mut_ptr());
    }
    
    Ok(pose)
}
