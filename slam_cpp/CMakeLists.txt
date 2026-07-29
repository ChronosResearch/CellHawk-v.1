cmake_minimum_required(VERSION 3.10)
project(ORB_SLAM2_Mock)

set(CMAKE_CXX_STANDARD 14)

# Create a shared library
add_library(orb_slam2 SHARED orb_slam2_mock.cpp)

# (In Linux we'd add -fsanitize=address but MSVC has its own ASAN setup via /fsanitize=address)
if (MSVC)
    target_compile_options(orb_slam2 PRIVATE /fsanitize=address)
    target_link_options(orb_slam2 PRIVATE /INFERASANLIBS)
else()
    target_compile_options(orb_slam2 PRIVATE -fsanitize=address -g)
    target_link_options(orb_slam2 PRIVATE -fsanitize=address)
endif()
