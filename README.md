# nbodies

### A demo app for  [dynamics](https://github.com/samiBendou/dynamics) framework
This project is an application that can simulate any stellar system configuration and render it in 3D.

![Demo](assets/demo.png)

#### Features
- Build and simulate your own stellar systems using the GUI
- Visualize trajectories and relative trajectories
- Simulate long periods (> 1Gy) with stability

#### Usage
Before launching this code you have to clone a few libraries a made specially for this app :
- [dynamics](https://github.com/samiBendou/dynamics)
- [unitflow](https://github.com/samiBendou/unitflow)
- [geomath](https://github.com/samiBendou/geomath)

Once you are ready, open your Cargo.toml and bind the frameworks using the following syntax `package={path="path/to/repo"}`
for each package.
That's the only way to compile for now since I did not put yet the above packages on crates.io

Clone the project and run `cargo build --release`. You can run the application using the following command:
```
$ path/to/repo/target/release/nbodies [-o path/to/data.json] [-d] [-t] [-w] [-h] [-s]
```
If you don't specify `-o` the application will start empty. You can add bodies by right clicking, first click sets positions,
second sets speed.

You may have to adjust   the time/distance scaling using U, I, Comma and Semicolon keys.

A keymap is contained in the file `src/common.rs`, for know you can refer to it to use the app. I'll be writing a tutorial soon.

Other optional options are provided:
- `-d` specify the distance scale of the simulation in px/m
- `-t` specify the distance scale of the simulation in s/real s
- `-w` and `-h` specify the size of the app in px
- `-s` specify the simulation oversampling rate in number of iterations/step