## RocketSim Visualizer

A light-weight visualizer for [rocketsim-rs](https://github.com/VirxEC/rocketsim-rs) binds that listens for UDP packets.

Any language can communicate with the visualizer by sending UDP packets in the correct format, but rocketsim-rs has a `GameState.to_bytes()` function that does this automatically.

![image](https://user-images.githubusercontent.com/35614515/230228589-f8b94023-17f8-4fb0-a0b4-62a1a3b8b82f.png)

### Usage

To see an example of how to communicate with the visualizer, see the [example](https://github.com/VirxEC/rocketsim-rs/blob/master/examples/rlviser_socket.rs) in the [rocketsim-rs](https://github.com/VirxEC/rocketsim-rs) repository.
