# nbodies

### A demonstration application for an general purpose point's physics engine
This project is an application that can simulate any stellar system configuration
and render it in 2D.

#### Features
- Simulate stellar systems with JSON data
- Build and simulate your own stellar systems using the GUI
- Visualize trajectories from any point of view

#### Usage
Clone the project and run `cargo build --release`. You can run the application using the
following command:
```
$ path/to/repo/target/release/nbodies [path/to/data.json]
```
If you don't specify data the application will start empty.