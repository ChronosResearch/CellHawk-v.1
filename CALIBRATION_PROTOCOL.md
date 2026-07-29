# CELLHAWK Calibration Protocol

## 1. Path Loss Exponent (n) Tuning (Step 4)
*Objective: Estimate the environment-specific log-distance path loss exponent.*
1. Transport the UAV to the center of the designated 2km^2 flight test area.
2. Identify 3 distinct cellular towers with direct Line-of-Sight (LoS).
3. Using a laser rangefinder or RTK-GPS, measure exact physical distances to towers: `d = [50m, 100m, 200m, 500m]`.
4. Run `cellhawk-sdr` in calibration mode: `cargo run --bin calibrate_sdr`.
5. Record averaged RSSI values for each distance bin.
6. Fit to equation: `RSSI = -10 * n * log10(d) + C`.
7. Update `config.toml` -> `[sdr] path_loss_exponent = n`.

## 2. Visual SLAM Calibration (Step 6)
*Objective: Calibrate camera intrinsics and IMU extrinsics.*
1. Use a standard checkerboard (e.g., 8x6, 30mm squares).
2. Record a 60-second ROS bag of the stereo cameras while translating and rotating the drone.
3. Run `kalibr_calibrate_cameras` to solve for intrinsics and distortion coefficients.
4. Update ORB-SLAM2 configuration file `settings.yaml` with the resolved matrix.
5. Compute the Camera-to-IMU extrinsic transformation matrix $T_{c}^{i}$.
6. Enter $T_{c}^{i}$ into the EKF config.

## 3. IMU Bias Estimation (Step 7)
*Objective: Remove static gyro and accelerometer biases.*
1. Place the drone on a perfectly level, vibration-isolated surface.
2. Do not touch or move the drone for exactly 10 minutes.
3. Run `cellhawk-ekf` bias estimator: `cargo run --bin estimate_bias`.
4. Take the 10-minute averaged offsets for $(a_x, a_y, a_z)$ and $(\omega_x, \omega_y, \omega_z)$.
5. Update `config.toml` -> `[ekf.bias]` parameters.
