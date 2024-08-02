cmake_minimum_required(VERSION 3.22)

project(Example VERSION 1.0
                DESCRIPTION "Example usage of Spotflow SDK"
                LANGUAGES C)

include(FetchContent)

FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.3.2
)

FetchContent_MakeAvailable(Corrosion)

# Run with debug to speed up builds -- this is the default and currently specifying it directly breaks the build
corrosion_import_crate(MANIFEST_PATH ../../Cargo.toml CRATES spotflow-c)
# Add some variables to further try to speed up compilation times
corrosion_set_env_vars(spotflow CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0)

add_executable(example example.c)
target_link_libraries(example PUBLIC spotflow)
target_include_directories(example PRIVATE ../../spotflow-c/include)

# Include more warnings and add dependencies
if(MSVC)
  add_compile_options(example PRIVATE /W4 /WX)
  target_link_libraries(example PRIVATE ws2_32 bcrypt userenv ntdll crypt32 secur32 ncrypt)
else()
  add_compile_options(example PRIVATE -Wall -Wextra -Wpedantic)
endif()

# Run the example after build
# Use the top-level "examples" directory so that the code can easily call Python scripts for testing purposes
add_custom_target(
  Run ALL
  COMMAND example $ENV{SPOTFLOW_TEST_DEVICE_ID}
  COMMENT "Running the example"
  WORKING_DIRECTORY ../..
)
add_dependencies(Run example)
