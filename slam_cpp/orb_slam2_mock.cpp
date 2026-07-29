#include <iostream>
#include <vector>

extern "C" {

    // Mock TrackStereo function returning a simple pose update
    __declspec(dllexport) void TrackStereo(double timestamp, double* out_pose) {
        // Output pose: x, y, z, roll, pitch, yaw
        out_pose[0] = 10.0; // x
        out_pose[1] = 20.0; // y
        out_pose[2] = 5.0;  // z
        out_pose[3] = 0.0;  // roll
        out_pose[4] = 0.0;  // pitch
        out_pose[5] = 0.0;  // yaw
        
        std::cout << "[ORB-SLAM2] TrackStereo called at t=" << timestamp << " -> Pose computed." << std::endl;
    }

}
