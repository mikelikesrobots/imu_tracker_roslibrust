# IMU Tracker roslibrust

This is a ROS 2 package intended to show the roslibrust version of three nodes:

- `counter`: a simple publisher that publishes a count from 0 once per second.
- `imu_summer`: a subscriber that listens on the `/imu` topic for IMU messages, then sums up the x component of the angular velocity from that message.
- `mult_floats`: a service that multiplies two floats together based on a custom service definition.

The three nodes show publishing, subscribing, and service calls in ROS 2, including with custom messages. The purpose is to demonstrate how to build a few basic nodes with roslibrust, but also to provide a direct comparison to the same nodes written with `ros2rust`, available at [https://github.com/mikelikesrobots/imu_tracker_ros2rust](https://github.com/mikelikesrobots/imu_tracker_ros2rust).

## Setup

This package requires access to a custom message type, so it is recommended (but not required) to build it within a ROS 2 workspace. If the custom message type was not required, then no ROS installation would be necessary at all, and the package could be built as a standard Cargo package.

Instructions to set up the workspace are as follows:

```bash
# Install Rust, e.g. as described in https://rustup.rs/
# Install ROS 2 as described in https://docs.ros.org/en/jazzy/Installation.html
mkdir -p roslibrust_ws/src && cd roslibrust_ws
git clone --recurse-submodules https://github.com/mikelikesrobots/imu_tracker_roslibrust src/imu_tracker_roslibrust
git clone https://github.com/mikelikesrobots/mult_msgs src/mult_msgs

# One-time build
colcon build
# Source in every new terminal to get access to mult_msgs
source install/setup.bash

# Build the Cargo package
cd src/imu_tracker_roslibrust
cargo build
```

## Running the Node

Running this package with ROS 2 has a prerequisite to make it work: rosbridge. To install rosbridge, execute:

```bash
sudo apt install -y ros-jazzy-rosbridge-library
```

Then run the bridge using:

```bash
ros2 launch rosbridge_server rosbridge_websocket_launch.xml
```

You can leave the bridge running indefinitely. Run the `imu_track_roslibrust` package by executing:

```bash
cd roslibrust_ws/src/imu_tracker_roslibrust
source ../../install/setup.bash
cargo run
```


This will print out that the nodes have started and start counting:

```bash
Counting node has started!
IMU summing node has started!
Float multiplying service has started!
Published 0
Published 1
Published 2
```

## Interact with the Nodes

### View the Published Count

For the count publishing, open another terminal and execute:

```bash
ros2 topic echo /example_counter
```

This will start to print out numbers, e.g.

```bash
---
data: 40
---
data: 41
---
data: 42
---
```

### Call the Float Multiplication Service

For the float multiplication service, open another terminal and execute:

```bash
cd roslibrust_ws
source install/setup.bash
ros2 service call /mult_floats mult_msgs/MultFloats "{x: 2.5, y: 4.0}"
```

This will result in the following:

```bash
waiting for service to become available...
requester: making request: mult_msgs.srv.MultFloats_Request(x=2.5, y=4.0)

response:
mult_msgs.srv.MultFloats_Response(result=10.0)
```

As 2.5 * 4.0 = 10.0, our service has calculated the result successfully. This also shows that custom messages can be used with the node.

### See the IMU sum increase

The setup for this is a little more complex as it uses the Turtlebot Gazebo simulation (although you can publish your own IMU messages if you prefer).

To install the simulations, execute:

```bash
sudo apt install -y ros-jazzy-turtlebot3-simulations
```

You should then be able to run the simulation by executing:

```bash
export TURTLEBOT3_MODEL=waffle_pi
ros2 launch turtlebot3_gazebo turtlebot3_world.launch.py
```

This will start producing IMU data. The IMU summing node should receive the data and start to sum the x values of the angular velocity:

```bash
Got x: 0.00016605295940326384, sum: 0.0013029570806488157
Got x: -0.00010373969315866868, sum: 0.001199217387490147
Got x: -0.00007269928155955913, sum: 0.0011265181059305878
Got x: 0.00002150106849010582, sum: 0.0011480191744206937
Got x: -0.0004081654500772328, sum: 0.0007398537243434609
Got x: 0.000375131405007245, sum: 0.001114985129350706
Got x: 0.00020263544247801625, sum: 0.0013176205718287222
Got x: 0.0002873354535022987, sum: 0.001604956025331021
Got x: -0.00023864045717770534, sum: 0.0013663155681533155
Got x: -0.000059679387952645265, sum: 0.0013066361802006703
```

## Running Unit Tests

One of the advantages of roslibrust is gaining access to the mock runner for ROS. This makes writing unit tests very quick and simple. To run the unit test suite, execute:

```bash
cd roslibrust_ws/src/imu_tracker_roslibrust
source ../../install/setup.bash
cargo test
```

This _should_ give the following result:

```bash
   Compiling imu_tracker v0.1.0 (/home/mike/roslibrust_ws/src/imu_tracker)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.77s
     Running unittests src/main.rs (target/debug/deps/imu_tracker-a4e62704957b1ef9)

running 3 tests
test test::test_imu_summer ... ok
test test::test_pub_counter ... ok
test test::test_mult_service ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
