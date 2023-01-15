/*!
# Quickstart
To start, create create a struct and implement the ```Logic``` trait on it:
```rust
use ascii_renderer::prelude::*;

struct MyLogic;

impl Logic for MyLogic {
    fn process(&mut self, screen_buf: &mut CharBuffer, delta: f32) -> ProcessReturn {
        ProcessReturn::Continue
    }
}
```
There will be more on this later, but for now just make ```process()``` return ```ProcessReturn::Continue```.

Next, create a ```Runner```, pass an instance of your logic struct to it, and run it.
```rust
use ascii_renderer::prelude::*;

struct MyLogic;

impl Logic for MyLogic {
    fn process(&mut self, screen_buf: &mut CharBuffer, delta: f32) -> ProcessReturn {
        ProcessReturn::Continue
    }
}

fn main() {
    Runner::new(
        5, //Width (in chars)
        5, //Height
        25, //FPS Cap
        MyLogic,
    ).run(true);    //true = clears the terminal between frames
}
```
The runner will proceed to run a loop (with a maximum frequency dictated by the ```fps_cap```) where it will run it's logic's ```process()``` function, which will mutate a ```CharBuffer```, then it will print that ```CharBuffer``` to the screen, and then it will repeat if ```process()``` returned ```ProcessReturn::Continue```.

The ```delta``` parameter is the amount of time (in seconds) that has passed since the last frame was drawn to the screen. It is necesary for non-frame-dependant movement.

The ```CharBuffer``` can be mutated by changing individual chars (```set_char(&mut self, x, y, char)```), filling the entire buffer (```fill(&mut self, char)```), drawing lines (```draw_line(&mut self, line)```), or by rendering 3D graphics to it (more on that later). The buffer is maintained between frames, you almost always should start ```process()``` with ```screen_buf.fill(' ');```.
```rust
use ascii_renderer::prelude::*;

struct MyLogic;

impl Logic for MyLogic {
    fn process(&mut self, screen_buf: &mut CharBuffer, delta: f32) -> ProcessReturn {
        screen_buf.fill(' ');

        let fps_string: String = (1.0 / delta).into();
        let mut fps_chars = fps.chars();

        screen_buf.set_char(0, 0, fps_chars.next().unwrap()).unwrap(); //Will write the fps to the screen
        screen_buf.set_char(1, 0, fps_chars.next().unwrap()).unwrap();

        screen_buf.draw_line(Line {
            char: '=',
            points: (vec2!(0.0, 3.0), vec2!(5.0, 3.0)),
        }); //Will draw a line to the screen using '='

        ProcessReturn::Continue
    }
}

fn main() {
    Runner::new(
        5, //Width
        5, //Height
        25, //FPS Cap
        MyLogic,
    ).run(true);    //true = clears the terminal between frames
}
```
To render 3D graphics to the ```CharBuffer```, we need to use a ```Renderer```. We don't want to instantiate a new ```Renderer``` every single frame, so we should store an instance of a ```Renderer``` wtihin a field of our logic struct. To draw graphics to the ```CharBuffer```, simply call ```draw()``` on the renderer, passing a mutable reference to the ```CharBuffer``` to it. In order to have something to render, you can create a 2x2x2 cube mesh using the ```create_cube()``` function and pass the cube to the renderer within it's declaration.
```rust
use ascii_renderer::prelude::*;

struct MyLogic {
    pub renderer: Renderer,
}

impl Logic for MyLogic {
    fn process(&mut self, screen_buf: &mut CharBuffer, delta: f32) -> ProcessReturn {
        screen_buf.fill(' ');

        let fps_string: String = (1.0 / delta).into();
        let mut fps_chars = fps.chars();

        self.renderer.draw(screen_buf);

        self.renderer.meshs[0].rotation.x += delta * 2.0;
        self.renderer.meshs[0].rotation.y += delta; //Rotates the cube. Because it's just a wireframe model, if there isn't any movement it won't look 3D.

        ProcessReturn::Continue
    }
}

fn main() {
    Runner::new(
        5, //Width
        5, //Height
        25, //FPS Cap
        MyLogic {
            renderer: Renderer {
                meshs: vec![ascii_renderer::create_cube()],
                camera: Camera {
                    position: vec3!(0.0, 0.0, -7.0),
                    rotation: vec3!(0.0, 0.0, 0.0),
                    fov: vec2!(0.8, 0.8),   //Is in RADIANS. Make sure this is proportional to the dimensions of the CharBuffer, otherwise there will be stretching.
                },
            },
        },
    ).run(true);    //true = clears the terminal between frames
}
```
For any values that need to be consistent, more fields can be added to the logic struct. For example, this logic contains a field that keeps track of how much time (in seconds) has passed since the runner started, and ```process()``` feeds that value into a sin function which determines the cube's scale in each dimension, creating a cool looking effect (as shown in [this](https://youtu.be/faViJzniUQA) video):
```rust
use ascii_renderer::prelude::*;

struct MyLogic {
    pub renderer: Renderer,
    pub time_offset: f32,
}

impl Logic for MyLogic {
    fn process(&mut self, screen_buf: &mut CharBuffer, delta: f32) -> ProcessReturn {
        screen_buf.fill(' ');

        self.time_offset += delta; //Keeps track of time

        self.renderer.draw(screen_buf);
        self.renderer.meshs[0].rotation.x += delta * 0.8; //Rotates the cube
        self.renderer.meshs[0].rotation.y += delta * 1.0;
        self.renderer.meshs[0].rotation.z += delta * 1.2;

        self.renderer.meshs[0].scale.x = 1.0 + (self.time_offset * 2.0).sin() * 0.5; //Scales the cube according to sin(time)
        self.renderer.meshs[0].scale.y = 1.0 + (self.time_offset * 3.0).sin() * 0.5;
        self.renderer.meshs[0].scale.z = 1.0 + (self.time_offset * 5.0).sin() * 0.5;

        ProcessReturn::Continue
    }
}

fn main() {
    let mut runner = Runner::new(
        50,
        50,
        25,
        MyLogic {
            renderer: Renderer {
                meshs: vec![ascii_renderer::create_cube()],
                camera: Camera {
                    position: vec3!(0.0, 0.0, -7.0),
                    rotation: vec3!(0.0, 0.0, 0.0),
                    fov: vec2!(0.8, 0.8),
                },
            },
            time_offset: 0.0,
        },
    );
    runner.run(true);
}
```
Finally, to load meshes from file (currently only .OBJ is supported), run the function ```AsciiObj::load(path)```, which will return a ```Result<AsciiObj, ObjError>```. After ```unwrap()```ing it, the ```AsciiObj``` can be converted into a ```Vec<Mesh>``` using ```into()```, which all together would look like ```let my_meshes: Vec<Mesh> = AsciiObj::load("face.obj").unwrap().into();```. However, often times meshes are far from the origin, causing the mesh to appear to spin in a large circle centered around the origin rather than rotate around a point when rotated. Because of that, allways run the ```recenter()``` method on the mesh before passing it to the renderer. ```recenter()``` returns the position the mesh was originally centered at, if you wish to maintain it's in-file position. This example demonstrates overall how to load objs:
```rust
use ascii_renderer::prelude::*;

#[derive(Debug)]
struct MyLogic {
    pub renderer: Renderer,
}

impl Logic for MyLogic {
    fn process(&mut self, screen_buf: &mut CharBuffer, delta: f32) -> ProcessReturn {
        screen_buf.fill(' ');

        self.renderer.draw(screen_buf);

        self.renderer.meshs.first_mut().unwrap().rotation.y += delta;

        ProcessReturn::Continue
    }
}

fn main() {
    let mut my_meshes: Vec<Mesh> = AsciiObj::load("face.obj").unwrap().into();
    my_meshes.iter_mut().for_each(|mesh| {
        // * Scales the obj down. rotates it so that it is rightside up, and recenters it.
        mesh.scale = vec3!(0.01, 0.01, 0.01);
        mesh.rotation = vec3!(std::f32::consts::PI, 0.0, 0.0);
        mesh.recenter();   // * This OBJ is really far from the origin for some reason, so if it is not recentered it 
    });
    let mut runner = Runner::new(
        50,
        50,
        25,
        MyLogic {
            renderer: Renderer {
                meshs: my_meshes,
                camera: Camera {
                    position: vec3!(0.0, 0.0, -3.0),
                    rotation: vec3!(0.0, 0.0, 0.0),
                    fov: vec2!(0.8, 0.8),
                },
            },
        },
    );
    runner.run(true);
}
```
*/


pub mod char_buffer;
pub mod line;
pub mod obj;
pub mod rendering;
pub mod runner;

pub use rendering::{Vector2, Vector3};

pub mod prelude {
    pub use super::char_buffer::CharBuffer;
    pub use super::line::Line;
    pub use super::obj::AsciiObj;
    pub use super::obj::ObjError;
    pub use super::rendering::{Camera, Mesh, Renderer};
    pub use super::runner::ProcessReturn;
    pub use super::runner::{Logic, Runner};
    pub use super::{vec2, vec3, Vector2, Vector3};
}

pub fn create_cube() -> rendering::Mesh {
    //!Generates a 2 x 2 x 2 cube for testing and sampling

    let mut cube = rendering::Mesh::default();
    //Top Square
    cube.insert_vertex(0, vec3!(1.0, 1.0, 1.0));
    cube.insert_vertex(1, vec3!(-1.0, 1.0, 1.0));
    cube.insert_vertex(2, vec3!(-1.0, -1.0, 1.0));
    cube.insert_vertex(3, vec3!(1.0, -1.0, 1.0));

    cube.add_edge((0, 1));
    cube.add_edge((1, 2));
    cube.add_edge((2, 3));
    cube.add_edge((3, 0));

    //Bottom Square
    cube.insert_vertex(4, vec3!(1.0, 1.0, -1.0));
    cube.insert_vertex(5, vec3!(-1.0, 1.0, -1.0));
    cube.insert_vertex(6, vec3!(-1.0, -1.0, -1.0));
    cube.insert_vertex(7, vec3!(1.0, -1.0, -1.0));

    cube.add_edge((4, 5));
    cube.add_edge((5, 6));
    cube.add_edge((6, 7));
    cube.add_edge((7, 4));

    //Connecting the squares
    cube.add_edge((0, 4));
    cube.add_edge((1, 5));
    cube.add_edge((2, 6));
    cube.add_edge((3, 7));

    cube
}
