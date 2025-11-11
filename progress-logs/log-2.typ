#import "shared.typ": *
#show: shared-styles

#align(center)[
  #text(18pt)[Progress Log #text(purple)[*\#2*]]

  #smallcaps(text(12pt, purple)[Ray Tracing Project])

  Pierson M

  25 March 2025 --- Career Life Connections 12 --- Period 2.1
]

#outline()

= Capstone Project

My Capstone project is to create and document a ray-tracing engine based on
the guide _Ray Tracing in One Weekend_ by Peter Shirley et al. The code used
to create the project is published on my GitHub at
https://github.com/piemot/raytracing.

I won't be directly following the guide. I'll be writing my engine in Rust,
instead of C++, because it's a language I prefer to code in and is capable
of similar performance. Unlike Shirley's 3-dimensional vector class being
used for a 3D point, an RGB color, and more, I'll enable dimensional analysis
by making them different types, even separating normalized and un-normalized
vectors.

= Work

The following contents are an edited version of a log
I've kept as I've worked on this project.
The full log can be found at https://github.com/piemot/raytracing/blob/main/JOURNAL.md.

This is the second of three Action Logs in this series. The first focused
on creating a basic path-tracing ray-tracer. This log focuses on my journey
to implement textures, shapes other than spheres, bounding volume hierarchy
speed optimizations, emissive primitives (lights), and constant-density
convex mediums (smoke).

== Getting Started #math.dot *Mar 6 -- 10*

=== Motion Blur

As a warmup, the first thing I implemented was motion blur - specifically,
the motion blur produced by a single linear motion over the period of
exposure.

Time is involved, so the `Ray3` struct needs to become a `Ray4` struct.
Instead of only containing a 3D origin and a 3D vector, a `Ray4` also includes
a given point in time, ranging from 0% -- the start of the camera shutter --
to 100% -- the end of the exposure period.

Since multiple rays are shot towards each pixel (most of the images in this
log are rendered at \~500 rays/px), they can be divided up to be simulated
at different times. In other words, instead of producing 500 3D rays, 100 4D
rays are produced at $t =$ 0%, 100 at $t =$ 20%, 100 more at $t =$ 40%,
and so on. This would create five "bars" of where each object is, so to
make it entirely blurry every ray has a randomly-selected time attached to it.

The scene displayed in @motionblur, showing balls bouncing up and down,
isn't quite accurate to its real life analog because the balls are moving
completely linearly. A good point for future development would be allowing
objects to specify their position at a given time, allowing arbitrary
animations across a frame.

#figure(
  image("../assets/motionblur.png", width: 70%),
  caption: [
    A set of balls, moving vertically with a motion-blur effect
  ],
)<motionblur>

=== Bounding Volume Hierarchies: an overview

Bounding Volume Hierarchies are a technique used to improve the speed
of the ray tracer. In the current version, when a ray bounces off an object
it has to check every other `Hittable` in the scene to see if it intersects.

If the scene could be divided into a hierarchy of smaller regions, the ray
could choose to only check the nearest region in the desired direction.

For instance, if some spheres are placed inside a cube, the cube can be
checked for an intersection. If the cube is not hit, there's a guarantee
that none of the spheres contained within it are hit either.

@boundingbox demonstrates a set of rays fired towards a bounding box
surrounding two circles.
Blue rays intersect the bounding box and the circles; these are
"true positives". Red rays intersect the bounding box but *don't* hit a
circle; these are "false positives" and show rays that the bounding box
does not help to optimize. However, the green rays miss the bounding box
and are optimized away. Not pictured are the many other rays, pointing
sideways or backwards, that would also be optimized out by this method.

#figure(
  image("../assets/03-09-boundingbox.svg", width: 70%),
  caption: [
    A diagram of rays intersecting a bounding box
  ],
)<boundingbox>

Notably, in order for the BVH strategy to work,
there needs to be a fast way to
#enum(numbering: "a.")[
  create useful volumes, and
][
  quickly check whether a ray intersects with a hierarchy of volumes.
]


In practice, for most (but not all) models, axis-aligned rectangular prisms
tend to solve both of these problems the best. For short, I'll call them
AABBs; axis-aligned bounding boxes.

Unlike collisions with an object, points, normals, and other data don't need
to be calculated when testing AABB intersections. This is because AABBs
aren't rendered; it doesn't matter where they are on the screen, only
whether or not a ray intersects them. This means they can't implement
`Hittable`; it requires either a ```rs None``` value (the ray missed) or
```rs Some(details)``` (the ray did hit, and all the details of the hit are
provided). Later, I'll need to create another struct that _does_ implement
`Hittable` to proxy the events.


The simplest way to create AABBs is to construct them based on the idea that
a $n$-dimensional rectangle (i.e. a rectangle or rectangular prism) is
composed of $n$ intervals (for instance, `(2.0, 3.0)`) - one for each axis.
Given this, a bounding box is just the overlap of the intervals, which forms
a rectangle in 2D space and a rectangular prism in 3D space, as illustrated
in @boundingbox-const.

#figure(
  image("../assets/03-09-boundingbox-const.svg", width: 70%),
  caption: [
    The construction of a bounding box from intervals $x= (2, 3)$ and $y =(0, 1)$
  ],
)<boundingbox-const>

=== Bounding Box Collisions


Remembering that the formula for a ray is $P(t) = A + t B$, an interval of
$t$ values where the ray intersects a given plane can be calculated.
If a $t$ interval is computed for each axis of the box, and all of these
$t$ intervals overlap, the ray must be intersecting the bounding box.

In the image below, $t$--intervals $A_x$ and $A_y$ don't overlap; neither do
areas $C_x$ and $C_y$. But since ray $B$ goes through the bounding box,
areas $B_x$ and $B_y$ intersect inside that box.

#figure(
  image("../assets/03-09-intersect.svg", width: 70%),
  caption: [
    Bounding box intersections: ray $B$ passes through the box while rays $A$ and $C$ miss it.
  ],
)<intersect>


This is really quick to check - just compute the $t$--value of the ray at
each edge of a given axis's interval with $t_(0x) = (x_0 - A) / B$, where $A$
is the origin point of the ray, $B$ is  the ray's vector, $x_0$ is the lowest
value of the x-axis interval, and $t_(0x)$ is the $t$-value where the ray
intersects $x_0$. Swapping out $t_(1x)$, the farther part of the x-axis
interval, the second half of the $t$ interval is $t_(1x) = (x_1 - A)/B$, and
the final x-axis interval of $t$ values is ($t_(0x)$, $t_(1x)$).

This is really quick to compute - just some addition, multiplication, and
intersection logic (which is mostly `<=` and `>=` statements).


=== Hittable Bounding Boxes

All `Hittable` objects should be able to provide a bounding box that
encompasses them, so the `Hittable` trait has been extended with
a new method:

```rs
    fn bounding_box(&self) -> Option<&BoundingBox3>;
```

This indicates that anything that is `Hittable` must be able to return either
its own bounding box (```rs Some<&BoundingBox3>```), or indicate that it
doesn't have a bounding box and therefore doesn't need to be hit
(```rs None```). An example of this behavior would be an empty
`HittableVec`; a `HittableVec` proxies hits to its contained objects, but if
it doesn't have any objects there's no need to accept hit attempts.

A `Sphere` just returns the bounding box of a cube circumscribed around
itself, which is trivial to calculate given the radius and center of the
sphere. A `HittableVec`, if it holds objects, should return the smallest
rectangle that can fit over all its' children's own bounding boxes.

=== Hierarchies

The key to this optimization lies in the name: Bounding Volume *Hierarchies*.

#figure(
  image("../assets/03-09-hierarchy.svg", width: 70%),
  caption: [
    A hierarchy of bounding boxes surrounding objects
  ],
)<bb-hierarchy>

A hierarchy of bounding boxes is illustrated in @bb-hierarchy. A parent yellow
box contains green and blue boxes. The green box, in turn, contains red,
orange, and another yellow box.

If a ray misses any element in the hierarchy, _it must also miss all of its
children_. For instance, if a ray misses the green box, all the elements
in the orange, yellow, and red boxes can be eliminated without individually
checking them.

Instead of checking (with a slow algorithm, because checking renderable
objects produces hit metadata) all objects in the scene, it quickly
(without metadata) checks a few parent bounding boxes before deciding there
are only a few objects that need to be hit and rendered fully.

This technique, naively implemented, results in at least a 2x speedup.

== Textures #math.dot *Mar 10*

It would be really nice to be able to draw a texture, like an external image,
onto one of the objects. In theory (although it would be an unnecessarily
complicated process), a `Material` could be created that changes its color
based on some parameter, like the y-value of the hit.
This would appear as something like @dicolor-mat.

#figure(
  image("../assets/dicolor.png", width: 70%),
  caption: [
    A two-colored sphere
  ],
)<dicolor-mat>


To implement this, I just copied the code for `Lambertian` materials, but
added this line:
```rs
attenuation = match hit_point.y > self.y_parameter {
    true => self.color1,
    false => self.color2,
};
```

It does work, but it's hard-coded and making any changes would be difficult.
Plus, it still doesn't let me upload an arbitrary texture; I'd have to hard-
code all the features of whatever texture I wanted to add.

Firstly, some definitions. _Texture Mapping_ is the process of mathematically
applying (_mapping_) a property (the _texture_) onto an object in the scene.
The most common property used is color, but it could also be the shininess,
shape (adding valleys or mountains), transparency, or any other property of
the target object.

For color mapping, a function needs to be defined that takes in a point on
the surface of an object, and returns the color that should be rendered at
that point. The easiest way to provide a texture is with a 2D image, and the
$x$ and $y$ coordinates on that image are, by convention, called $u$ and $v$.


This means a `Texture` trait would look like this:

```rs
trait Texture {
    fn value(&self, u: f64, v: f64, point: &Point3) -> Color;
}
```

`u` and `v` have to be calculated before this function, of course, but the
$u$ and $v$ coordinates are dependent on the type of object - a sphere
has a different $u v$ mapping from a cube.


A solid color would implement the trait very simply, ignoring the arguments
and only returning its own `Color`:

```rs
impl Texture for SolidColor {
    fn value(&self, ..) -> Color {
        self.albedo
    }
}
```

=== Spatial Textures

A "spatial" texture ignores the $u$ and $v$ coordinates, and is solely based
off of the provided point in 3D space. A simple example of this texture is a
checkerboard-style pattern, which takes two textures and a scaling value
and, depending on the point in 3D space, returns either of those input
textures.

Adjusting the Lambertian material to accept textures, instead of only colors,
the ground can be rendered in @checkerboard as a checkerboard.

#figure(
  image("../assets/checkerboard.png", width: 70%),
  caption: [
    The bouncing balls scene, but with a checkerboard green and white ground
  ],
)<checkerboard>

Rendering two checkerboard-textured spheres on top of one another
in @checker-spheres, something kind of weird starts to happen.

#figure(
  image("../assets/checker-spheres.png", width: 70%),
  caption: [
    Two checkered spheres on top of each other
  ],
)<checker-spheres>

The checkerboard suddenly reverses color at certain points, forming lines around the circumference
of the spheres. This happens because the checkerboard texture is a *3D* texture; it's influenced by
the $y$-position as well as $x$ and $z$. What happens if this influence is removed?

#align(center)[
  #image("../assets/checker-spheres-y.png", width: 70%),
]
Okay, that looks pretty good!

#align(center)[
  #image("../assets/checker-spheres-y-zoom.png", width: 70%),
]

Oh.

Because the $y$-axis dependency is removed, the sides of the spheres are just
going to have lines down them. Wrapping a checkerboard around a sphere just
_isn't possible_ with spatial textures - which is where the $u$ and $v$
coordinates mentioned earlier come in.

=== Spherical Surface Textures

Let's review the different types of textures.

/ Constant textures: don't use any coordinates.
/ Spatial textures: use the $(x, y, z)$ coordinates of the 3D space.
/ Surface textures: use the $(x, y, z)$ coordinates of the 3D space
  _and_ the $u v$ *texture* coordinates, which point to locations in a
  separate 2D space (the "texture space").

The mapping from positions on the surface of a 3D object to the texture
coordinates is arbitrary, but for a sphere a good method is to convert
latitude and longitude into 2D texture space.


Latitude and longitude can be represented by $theta$ and $phi$, where
$theta$ is the angle upwards from the bottom pole of the sphere, and $phi$ is
the angle around the $x$-axis, starting in the -$x$ direction and moving
towards +$z$. Mapping $theta$ to $u$ and $phi$ to $v$ across the range [0, 1]
(with $(0, 0)$ in the $u v$ space representing the *bottom-left* corner),
and using a lot of trigonometry, a function can take a point on the unit
sphere and return $(u, v)$ values ranging from 0 to 1.


Again by convention, $u$ and $v$ values range from `0.0` to `1.0`. Converting
a pixel $(i, j)$ in texture space on an $N_x$ by $N_y$ image to a $(u, v)$
value can be done with a fractional coordinate system:
$ u = i/(N_x - 1) $
$ v = i/(N_y - 1) $

With these formulas, an image of the world like @earth-tex

#figure(
  image("../assets/textures/earth.png", width: 70%),
  caption: [
    A rectangular satellite image of the Earth
  ],
)<earth-tex>

can be rendered across a sphere, in @earth!

#figure(
  image("../assets/earth.png", width: 70%),
  caption: [
    The texture from @earth-tex, rendered across a sphere using texture mapping
  ],
)<earth>

== Other Shapes #math.dot *Mar 10 -- 20*

So far, spheres have been the only shape possible. It's time to implement some other options.

=== Parallelograms

An obvious step would be to implement quadrilaterals - of which
parallelograms strike a good balance of simplicity and utility.
A flat parallelogram in 3D space is defined by an origin point $Q$ and two 3D
vectors $v$ and $u$ originating from $Q$.

The corners of this parallelogram are easy to calculate
(see @parallelogram): $Q + v$, $Q + u$, and $Q + v + u$.

#figure(
  image("../assets/03-10-parallelogram.svg", width: 70%),
  caption: [
    A 2D parallelogram and the definition of its corners
  ],
)<parallelogram>

Parallelograms are 2D objects, and therefore flat; they have a width of `0`
if they lie exactly in one of the planes. This can cause issues with
floating-point imprecision; rays could pass through the object because of
tiny rounding errors. In order to fix this, the bounding box will be
slightly larger than the parallelogram itself - which shouldn't have much of
a performance effect because the bounding box is designed to just be an
approximation of the object.


=== Parallelogram Intersections

A parallelogram, or any planar/flat shape, can be thought of as a segment of
a plane parallel to its face.

#figure(
  image("../assets/03-10-2d-plane.svg", width: 70%),
  caption: [
    A 2D object on an infinite plane
  ],
)<2d-plane>

Therefore, determining whether a ray intersects a 2D object takes two steps.
1. Check if the ray intersects the plane containing the object
2. Check if the ray touches the _part_ of the plane with the object on it

In order to check if the ray intersects a given plane, the implicit formula of a plane is used:

$ A x + B y + C z = D $

where $A$, $B$, $C$, and $D$ are constants, and $x$, $y$, and $z$ are the
coordinates of any point $(x, y, z)$ that lies on that plane. The plane is
therefore the set of points $(x, y, z)$ that satisfy that equation.

If $n$ is defined as a normal vector perpendicular to the plane --
meaning $n = (A, B, C)$, and $v$ is a vector from the origin to the
point on the plane, (meaning $v = (x, y, z)$),
then the value $D = n dot v$ for any position on the plane
#footnote[https://raytracing.github.io/books/RayTracingTheNextWeek.html#quadrilaterals/ray-planeintersection].


Determining the plane the object exists on is simple. Given the point $Q$ and
the vectors $u$ and $v$, the normal vector to the plane is the cross product
of $u$ and $v$: $n = u times v$. With the normal vector of the plane and a
point on the plane (which $Q$ is), $D = n dot Q$.

To determine whether the ray intersects the relevant _part_ of the plane,
the point $I$ at which the ray intersects the plane is compared to the
origin point $Q$ and the parallelogram's vectors $u$ and $v$:

$ I = Q + alpha u + beta v $

$alpha$ and $beta$ can be derived
#footnote[https://raytracing.github.io/books/RayTracingTheNextWeek.html#quadrilaterals/derivingtheplanarcoordinates]
to find $alpha = w dot (p times v)$ and $beta = w dot (u times p)$, where
$p = I - Q$ (the vector from the origin to the intersection point),
and $w$ is a vector representing the plane's basis frame, which is constant
to a given quadrilateral and equal to $w= n / (n dot n)$.


Now, given $alpha$ and $beta$, since they're fractional coordinates (scaled
from `0.0` to `1.0`), the ray intersects the parallelogram if
$0 <= alpha <= 1$ and $0 <= beta <= 1$.

#figure(
  image("../assets/squares.png", width: 70%),
  caption: [
    Planar squares, rendered on various axes
  ],
)<squares>

=== More Flat Shapes

Extending the parallelogram interface to allow for more shapes is relatively
simple. All of the ray intersection code is the same; the only difference is
in finding which $alpha$ and $beta$ coordinates represent a valid part of
that shape. Recalling that the valid area for a parallelogram is
$(0 <= alpha <= 1) and (0 <= beta <= 1)$
("$and$" represents "and" in logical arithmetic),
a disc could be represented as $sqrt(alpha ^ 2 + beta ^ 2) < R$, where R is
the radius of the disc relative to the size of the $u$ and $v$ vectors.


Similarly, a triangle could be represented as $(alpha > 0) and (beta > 0) and
(alpha + beta < 1)$, which selects a triangle between points $Q$, $Q + u$,
and $Q + v$ (see @parallelogram for these points).

#figure(
  image("../assets/shapes.png", width: 70%),
  caption: [
    Rendered squares, triangles, and discs
  ],
)<shapes>

=== Lighting

Something to note in @squares and @shapes is that shadows fall on the far
edges of the shapes because the light is being blocked by the green central
square. It's not very clear where the light originates from, though
(everywhere; anywhere a ray does not hit an object is light) --
and I can't add any extra lights aside from the sky.

The end goal of this lighting segment is to produce a
#link("https://en.wikipedia.org/wiki/Cornell_box")[Cornell Box],
a test scene created as a physically modellable scene that can be compared
against a rendered image.

It consists of one light source, a green right wall, a left red wall, and
various objects inside the box.

To start, materials can be allowed to emit light by extending the `Material` trait to include
an ```rs emitted()``` method:

```rs
pub trait Material {
    fn scatter(&self, ray_in: &Ray4, record: &HitRecord) -> Option<MaterialResult>;

    fn emitted(&self, u: f64, v: f64, point: &Point3) -> Color;
}
```

```rs emitted()``` returns a color, which is added to the color of the
scattered ray upon a bounce. This color could be brighter than pure white,
which is useful when a light source is small and reflections would dim it too
much. This function has a default implementation that returns
#display_color_img("#000") black `(0, 0, 0)`, so only light-emitting
materials need to specify how they emit light.

A simple diffuse light-emitting material can be implemented in the same way
as the Lambertian material - except instead of looking up scattered albedo
values from a texture, it looks up the emission values from a texture.

A yellow sphere with no surrounding light except a single light source
shining towards it is rendered in @light.

#figure(
  image("../assets/light.png", width: 70%),
  caption: [
    A sphere with light shining towards it
  ],
)<light>

In this case, the light source is 10 times brighter than pure white.
Since emission is defined by the material of the object, any object can be
emissive. For instance, a sphere could hang from the ceiling to illuminate
from above as well, like in @light2.

#figure(
  image("../assets/light2.png", width: 70%),
  caption: [
    A sphere with light shining towards it from the side and above
  ],
)<light2>

The light sources render as a very stark white, because in the simulation
they produce 10 times the light a pure white object would. This value gets
clamped to white once the object is rendered, but the darkness behind
it is entirely black and so some staggered edges start to appear.


=== Cornell Box (Part I)

A box can be constructed out of five walls, and the light at the top of the
box can consist of a sixth quadrilateral.

#figure(
  image("../assets/cornell-1.png", width: 70%),
  caption: [
    An empty Cornell box
  ],
)<cornell-1>

The empty Cornell box in @cornell-1 is dim and noisy because it's not lit
well; most rays miss the sole light source at the top of the box.
To (somewhat) fix this, any rays that do hit the light source have their
brightness multiplied _15 times_ so that they average out enough to see.

=== 3D Shapes

Usually, Cornell boxes contain rectangular prisms, set at angles.
A 3D box can be represented as a 6-object-long `HittableVec` of
`Parallelogram`s, one for each side of the box. Adding two of these boxes
into the Cornell box scene (and doubling the strength of the light for better
visibility), they render just as expected in @cornell-2.

#figure(
  image("../assets/cornell-2.png", width: 70%),
  caption: [
    A Cornell box with objects inside it
  ],
)<cornell-2>

=== Instancing

A useful trick that can be done with ray tracers is to create an _instance_
of a `Hittable` primitive or set of `Hittable` primitives. This creates a
lightweight copy of the primitive that can be easily translated or rotated.

The trick is that instead of directly rotating or translating the object
itself, which would require a lot of calculations to move each point
on the object, an incoming ray can instead be moved temporarily
in the inverse direction.

For instance, take the scene in @instance-translate. The true box is at
point `(1, 1)`, but the instance is imagined to be at point `(3, 1)`. If a
ray comes in originating from `(2.5, 2)`, and is going to hit the instanced
box, it can be translated so that its origin is at `(0.5, 2)` while the
hit calculation takes place, then moved back afterwards.

#figure(
  image("../assets/03-24-instance-translate.svg", width: 70%),
  caption: [
    An example of instanced translation
  ],
)<instance-translate>


Rotation is a bit more involved. The easiest way to calculate a rotation is to do so around an axis.
This is because a rotation around a given axis only modifies the other two coordinates;
rotation around the $z$ axis only changes the $x$ and $y$ coordinates of the rotated point.
The trig to calculate this rotation is pretty simple, and results in:

$ x^prime = cos(theta) dot x - sin(theta) dot y $

$ y^prime = sin(theta) dot x + cos(theta) dot y $

It works for any rotation, and doesn't require special cases for quadrants.
The formulas for rotating around the other axes are very similar
#footnote[https://raytracing.github.io/books/RayTracingTheNextWeek.html#instances/instancerotation].

Thinking of translation as a movement of the initial ray is okay, but the
analogy starts to fall apart when it comes to rotation.
What's really happening is a changing of coordinate systems. Instead of
moving the ray's origin position by a vector offset, the ray is converted
from "world space" into "object space". In object space, the intersection is
found, and then the intersection point and normal direction are converted
back into world space.

Implementing this instancing, the final version of the classic Cornell box
is shown in @cornell-3.

#figure(
  image("../assets/cornell-3.png", width: 70%),
  caption: [
    A classic Cornell box with rotated boxes inside it
  ],
)<cornell-3>

== Fog and Conveniences #math.dot *Mar 20 -- 30*

=== Volumes

A volume, or participating media, is an object that renders somewhat like
smoke or fog. It can be represented by a surface that may or may not exist
for each point inside the volume.

#figure(
  image("../assets/03-24-volume.svg", width: 70%),
  caption: [
    An example volume, scattering some light at various positions
    and allowing some rays to pass through unhindered
  ],
)<volume>

In @volume, the orange ray is able to pass entirely through the volume,
while the other rays are refracted in some way. They may be refracted
backwards, or their path may just be altered slightly.

In code, a `ConstantMedium` (i.e. a medium of constant density) uses another
`Hittable` object as its boundary. This could be a cube, sphere, or other
object. The code does assume that once a ray exits the object it won't
re-enter it, so concave objects aren't ideal. This could be fixed, but the
additional complication isn't worth it for my needs.

The distance from the edge of the cube at which the ray scatters is equal
to $-1/D times ln(R)$, where $D$ is the density of the object and $R$ is a
random number from `0..1`. The refraction is isotropic, meaning that it
returns a vector in the unit sphere, and so rays are refracted equally in all
directions at the same speed.

Putting this together, a few foggy boxes are placed in the Cornell box in
@cornell-4.
#figure(
  image("../assets/cornell-4.png", width: 70%),
  caption: [
    A Cornell box with foggy boxes inside it
  ],
)<cornell-4>

=== 2D Shape Constructors

Discs and triangles are currently specified by their corner and two vectors.
This way of specifying their positions is nonintuitive when it comes to discs;
instead of specifying the disc itself a user has to specify the
dimensions of a rectangle inscribed around the disc.

To fix this, I've added some conversion code so that points can be specified
more easily:

#align(center)[
  #image("../assets/03-25-shapes.svg", width: 70%),
]

For triangles, the three points of the triangle can be specified instead
of one point and two vectors. For discs, it makes a lot more sense to specify
the center point of the disc, as well as two vectors in perpendicular planes
giving it its shape.


=== About Animation

I _was_ going to implement more forms of animation than the current linear
implementation, but then I thought for a second. In all the scenes rendered so
far, the shutter speed of the simulated camera could be imagined to be a
standard small value - probably something near 1/20th to 1/40th of a second.

Assuming a ball weighs 0.5kg and falls from 0.5m to 0.0m
across 1/20th of a second, it's travelling at a speed of
$v= d/t = 0.5/0.05 = 10.0 "m"/"s"$.
By the time it reaches the ground, it would accelerate
due to gravity to only $sqrt(10^2 + 2(9.8)(0.5)) = 10.48 "m"/"s"$.
This difference is barely noticeable, and is very closely simulated
by a simple linear animation.

Maybe in the future a scene will pop up that makes use of
animation, and I'll implement nonlinear animation then - for now I don't
know what I'd even do with it.

== Note

For a while (between Mar 30 and Apr 15), I was working on a relatively minor
feature that would allow the objects in a scene to be described in a
text file, instead of in code (for instance, see this snippet below).
```toml
[textures.red]
type = "SolidColor"
color = "#ff0000"

[textures.cx]
type = "Checkerboard"
textures = ["#ff0", 0xfff]
scale = 1.0

[textures.world]
type = "Image"
path = "assets/textures/earth.png"

[materials.red]
type = "Lambertian"
texture = "red"

[[objects]]
type = "Parallelogram"
origin = [-3, -2, 5]
u = [0, 0, -4]
v = [0, 4, 0]
material = "red"
```

While this is definitely still a project I want to continue to explore,
I realized that I wasn't learning anything new, despite the time I continued
to put in to this project. Much of the challenge of implementing this sort
of system is not in the logic or programming knowledge, but instead requires
a lot of copy-pasting and boilerplate code.

Although I had made significant progress on it, I decided to leave this
feature unfinished for now, and to consider coming back to it later.
The feature isn't important enough to the overall progress towards my goal
of becoming a more knowledgeable and capable programmer to be worth
continuing to pursue.
