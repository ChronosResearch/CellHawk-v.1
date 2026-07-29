#include <cstdint>
#include <cstddef>
extern "C" {
    void* orb_slam2_init() {
        // Return a mock pointer representing the ORB_SLAM2::System instance
        return reinterpret_cast<void*>(0xDEADBEEF);
    }

    int orb_slam2_track(void* handle, const uint8_t* img_data, size_t len) {
        if (!handle || !img_data || len == 0) return -1;
        return 0; // Success
    }

    void orb_slam2_get_pose(void* handle, double* x, double* y, double* z) {
        if (handle) {
            *x = 10.0;
            *y = 20.0;
            *z = 30.0;
        }
    }

    void orb_slam2_destroy(void* handle) {
        (void)handle; // Mock destructor
    }
}
