# Journal

A journal of my progress on this project.

## Feb 16

### Init (ffcfaed)
* Initialized project
* Created README.md

### Vectors (ca9c4a4)
Created the `Vec3` and `Point3` structs.

Each of these represent a point in 3-dimensional space. Internally, a Vec3 is represented the same way as a Point3.
The distinction is that a Vec3 has different semantics as to how it can be used.

A vector can be imagined as an arrow pointing to a point.
A *normalized* vector is any vector with a length of 1, meaning that it lands on some point on the unit circle (shown as purple in the diagram below). The orange and green arrows are normalized vectors.

Red arrows are arbitrary vectors. These can be normalized by extending or retracting their length until it's equal to 1.

A vector can be trivially converted to a point, and vice versa. See how the point `(1.75, 1)` can be represented both by a vector pointing to it, and solely by a point.

![vectors](assets/02-16-vectors.svg)

Vectors can be added and subtracted; subtracting a vector is equivalent to adding the negative of that vector.

![vector math](assets/02-16-vector-math.svg)

`Vec3`s store whether or not they are normalized through something called the [typestate pattern](https://cliffle.com/blog/rust-typestate); this makes it a compile-time error to pass an unnormalized vector to a function requiring a normalized vector.

Quoting from the documentation of `Vec3`, this snippet will fail to compile, because `Vec3::new` returns a `Vec3<Unknown>`.
```rs
fn use_normalized(vec: &Vec3<Normalized>) {
    // ...
}
use_normalized(&Vec3::new(1.0, 2.0, 3.0)); // shouldn't compile!
```

This will work, because `as_unit()` returns a vector that is known to be normalized!

```rs
use_normalized(&Vec3::new(1.0, 2.0, 3.0).as_unit());
```

### Color & Interval (5b245d5)
Added the Color and the Interval structs - neither are particularly interesting and mostly copied from previous projects.

## Feb 17

### PPM Format
[assets/test-ppm.ppm](assets/test-ppm.ppm)

PPM is a very simple image format. It starts with a header:
```
P3
<width> <height>
<max-brightness>
```
And from then on, each row is three space-delimited integers, representing the brightness of each RGB value.
The start of `test-ppm` appears as so:

```ppm
P3
256 256
255
0 0 0
1 0 0
2 0 0
3 0 0
4 0 0
5 0 0
6 0 0
7 0 0
8 0 0
9 0 0
10 0 0
11 0 0
12 0 0
...
```

### Using the `Color` struct

Since changing over to the `Color` struct for writing colors, things look fairly different because of gamma-correction.
I'll probably write more on gamma-correction later because it's interesting, but suffice it to say that effectively, 
all color values are square rooted.

### `Ray` struct

`Ray` is a small utility class I forgot to implement yesterday.
It represents a vector that starts at a single point, and allows calculating a point along that vector.
It can also be thought of as a function $P(t) = A + tB$, 
where $A$ is the `point` it originates from and $B$ is a vector travelling from that point.
Accordingly, a $t$ value of `0.0` will return point $A$, a value of `1.0` will return the vector sum of $A$ and $B$,
and values outside that range will be extended **past** the area that vector $B$ covers.

### Sending Rays

Fundamentally, a ray tracer does a few simple steps. 
1. Create a ray from the camera through a given pixel.
![A ray is cast through a viewplane](assets/02-17-raycast.svg)
2. Find what objects in the scene the ray touches.
3. Find the color of the closest touched object.

Since we cast rays through a 2d plane (marked as `Viewport` above), representing 2d coordinates would be a good idea.
2d coordinates can be represented with a Vec3 or Point3 by setting `z` to `0.0`, but having separate structs for a 2d space
is a better practice because 2d coordinates must be limited to a 2d space. 
Setting a different `z` value should not be an option.

### Viewport

![A diagram of the viewport](assets/02-17-viewport.svg)

The viewport has a few properties that need to be considered. First, we need to be able to navigate through it.
We can define 2D vectors $u$ and $v$ as vectors that cross the top and side of the viewport, and then
divide them by the viewport's width and height respectively to get $\Delta u$ and $\Delta v$, 
which define the distance between rendered pixels.

The rays we send towards pixels should be sent to the **center** of those pixels, 
so we need to shift their positions by $\frac{1}{2}\Delta v + \frac{1}2\Delta u$ 
to make them centered. This is represented by the red vector in the above image.

### Background

If rays don't hit anything, what do we render? The guide suggests rendering
a bluish colour based on the y-value of the ray. The ray has to be normalized first,
so that the y component cannot be larger than `1.0`.

> **Sidenote**: In hindsight, enabling gamma correction earlier was a mistake as many of the
> examples used in the guidebook assume it's disabled. Since reenabling it is a
> simple matter of adding `.as_gamma_corrected()`, I'll disable it again for now.
