#import "shared.typ": *
#show: shared-styles

#align(center)[
  #text(18pt)[Progress Log #text(purple)[*\#1*]]

  #smallcaps(text(12pt, purple)[Ray Tracing Project])

  Pierson M

  1 March 2025 --- Career Life Connections 12 --- Period 2.1
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

== What is a ray-tracing engine?

Ray tracing is a technique used to realistically render (produce an image of)
a scene (any assortment of shapes or objects). It's often used in games,
allowing shadows and colored reflections to be rendered accurately to the
real world, as well as in film and television, where more advanced renderers
are used for excellent realism while taking much more time and computing
power to run.

Ray tracing works by starting at a point (the "camera point"), projecting a
ray forwards, and seeing what it bounces into and what color those objects
are. If that ray is run in reverse, it should start at a light source (like
the sky) and bounce off of objects before it lands back in the camera point.

I'm going to start by creating a sky, then placing some spheres into the
scene (spheres were chosen because they're easy to compute), and gradually
add materials and properties to the objects in the scene.

==== Terminology

/ `struct`: Short for "structure". The definition of some concept's
  properties and features. For instance, the _concept_ of a 3-D vector can be
  represented by a `Vec3` struct. A single vector is an _instance_ of that
  `Vec3` `struct`, and contains a point that the vector points towards.
/ Scene: The group of spheres/other objects that need to be rendered, defined
  mathematically.

== Initial Setup #math.dot *Feb 16 -- 17*

=== Vectors and Points

Created `Vec3` and `Point3` structs.

Each of these represent a point in 3-dimensional space,
but have different semantics: a `Vec3` can be imagined as an arrow,
while a `Point3` should only be thought about as a single point.

A *normalized* vector is a vector with a length of `1.0`, meaning that it
lands on a point in the unit sphere.
@vectors illustrates this in 2 dimensions.

Arrows landing on the #text(rgb("6741d9"), font: "Comic Neue")[purple]
unit circle are normalized vectors,
while #text(rgb("e03131"), font: "Comic Neue")[red]
(non-normalized) vectors can be normalized by extending
or retracting their length so that they touch the unit circle.

#figure(
  image("assets/02-16-vectors.svg", width: 70%),
  caption: [An illustration of vectors in 2D space],
)<vectors>

A vector can be trivially converted to a point, and vice versa. See how the point $(1.75, 1.0)$ can be represented both by a vector pointing to it, and solely by a point.

=== Vector Math

Vectors can be added and subtracted (@vector-math); subtracting a vector is equivalent to adding the negative of that vector.

#figure(
  image("assets/02-16-vector-math.svg", width: 70%),
  caption: [An illustration vector subtraction and addition in 2D space],
)<vector-math>

The length of a vector can be found with the Pythagorean theorem:
in 2D space, $L = sqrt(x^2+y^2)$ and in 3D space, $L =
sqrt(x^2+y^2+z^2)$.
Other functions can be computed on vectors, like
the dot product: $arrow(a) dot arrow(b) =
a_x times b_x + a_y times b_y + a_z times b_z$
and the cross product, a vector orthogonal to both $arrow(a)$ and $arrow(b)$.

Vectors can also be multiplied by a number, multiplying its length
by that number.

=== Rays

`Vec3` structs, as implemented, don't store a starting, or origin, point.
When an origin is needed, like in a ray that bounces off an object, a `Ray`
(or `Ray3`, in the code) is needed.
A `Ray` associates an origin point with a vector pointing away from it.

#note[
  The `3` in "`Ray3`" represents the dimensional space the ray exists in.
  In this case, that's 3D space.
  Later on in the project, more dimensions may be used -
  for instance, a fourth dimension of time.
]

In more mathematical terms, a `Ray` represents a vector that starts at a
single point, and permits the solving of a point along that vector
given a percentage of the distance along the vector.

It can be thought of as a function $P(t) = A + t arrow(B)$, where $A$ is the
`Point3` it originates from and $arrow(B)$ is a vector travelling
outwards from that point.

A value of $t = 0.0$ therefore returns the point $A$, and $t = 1.0$ returns
the point at the end of vector $arrow(B)$.
Values outside the range (0.0, 1.0) are extended past the
space that vector $arrow(B)$ covers.

=== The PPM Image Format

The PPM format is one of the simplest formats possible (and therefore
easy to write a program for).

It starts with a header of "P3", the width and height of the image in pixels,
and the maximum value a color channel can reach. In typical RGB-255 format,
this would be `255`.

#figure(
  text(rgb("15803d"))[
  ```
  P3
  <width> <height>
  255
  ```
  ],
  caption: [The header format of a PPM file]
)

After the header, each row is a pixel, from left to right and top to bottom,
consisting of three numbers where each represents the brightness of the red,
green, or blue color channels.

A common graphics testing image is a 256$times$256 pixel image, with the red
color channel increasing from 0 to 255 across the left to right axis, and
the green channel increasing across the top to bottom axis. This can be seen in @ppm-example.

#figure(
  image("assets/test-ppm.png"),
  caption: [
    An example PPM image, increasing in the red channel\ from left to right
    and the green channel from top to bottom.
  ],
)<ppm-example>

Since pixels are ordered one at a time, left to right, the first few lines of
@ppm-example look like this:

#figure(
  text(rgb("15803d"))[
  ```
  P3
  256 256
  255
  0   0   0
  1   0   0
  2   0   0
  3   0   0
  4   0   0
  5   0   0
  6   0   0
  7   0   0
  8   0   0
  9   0   0
  10  0   0
  11  0   0
  12  0   0
  ```
  ],
  caption: [The start of an example PPM file]
)

== Ray Tracing #math.dot *Feb 17 -- 20*

=== Sending Rays

Fundamentally, a ray tracer does a few simple steps, many times.

1. Create a ray from the camera through a given pixel (see @raycast).
2. Remember what objects in the scene the ray touches as it bounces around.
3. Find the color of the nearest touched object.

Notably, this process is the *reverse* of the idea of light we're used to.
We usually think of light as coming from a source, bouncing around, and
being absorbed by our eyes or any camera - and that _is_ how it works
in the real world.

In simulations, however, most of those rays won't hit
the camera and so it's much more efficient to reverse the process.

The result is the same, because rays that hit the camera are just the
reverse of rays that would hit the camera, if they came from a light source.

#figure(
  image("assets/02-17-raycast.svg", width: 70%),
  caption: [
    A ray is cast from an origin through a plane called the viewport
  ],
)<raycast>

=== Viewport Details

There's some math involved in figuring out _where_ exactly each pixel is
in the viewport. The viewport exists in 2D space as a grid of pixels,
but also in 3d space as a grid of points on a plane
that rays are sent towards.

2D vectors $arrow(u)$ and $arrow(v)$ are vectors that cross the x and y axes
of the viewport, and the distance _between_ pixels on the viewport can be
found with each vector being divided by the viewport's width and height,
respectively. This can be called $Delta arrow(u)$ and $Delta arrow(v)$.

The problem is, the first pixel is still centered on the upper-left viewport
corner. All the pixels need to be shifted left by $1/2 Delta arrow(u)$ and
down by $1/2 Delta arrow(v)$. This ensures that the points representing the
pixels are at the center of the pixels, and can be represented as a shift of
$1/2 Delta arrow(u) + 1/2 Delta arrow(v)$, or the
#text(rgb("e03131"), font: "Comic Neue")[red] vector in @viewport.


#figure(
  image("assets/02-17-viewport.svg", width: 70%),
  caption: [
    A ray is cast from an origin through a plane called the viewport
  ],
)<viewport>

=== The Sky (Background)

If rays don't hit anything at all, what should be rendered?
_Ray Tracing In One Weekend_ has a (relatively complicated) formula for
rendering a bluish color based on the $y$ component of the ray when it's
normalized. Since the ray is normalized, the rendered color actually forms
an arc (@background) because the normalized $y$ component
to the left and right will be smaller than that in
the center, where the $x$ component of the ray is 0.0 and so the
$y$ component must be greater..

#figure(
  image("assets/background.png", width: 70%),
  caption: [
    A blue gradient as a background/sky
  ],
)<background>

=== Sphere Intersection

Time to start adding some objects!
A sphere is an easy way to start, because it's mathematically simple.

The formula of a sphere is $x^2+y^2+z^2=r^2$.
This describes a point $(x, y, z)$ as being *on the surface* of a sphere
if $x^2+y^2+z^2=r^2$, *inside* the sphere if $x^2+y^2+z^2<r^2$,
and *outside* the sphere if $x^2+y^2+z^2>r^2$.
#footnote[
  This assumes that the sphere is at $(0, 0, 0)$ -
  the formulas actually implemented account for this.
]

All the mathematical details about deriving a useful formula for ray-sphere
intersection are available in _Ray Tracing in One Weekend_
#footnote[https://raytracing.github.io/books/RayTracingInOneWeekend.html#addingasphere/ray-sphereintersection],
so I won't go into too much detail, but essentially I can derive a
quadratic function that solves for where a given ray hits the edge of
a sphere.
For now, if a ray does hit the sphere, it's rendered in a color that
describes the normal vector off of its surface.
A "normal vector" is a vector that is perpendicular to the surface of the
object, at the point where the ray hits it. The normal vector of the
surface is an essential property to know once rays start bouncing,
because the reflection and refraction of nearly all materials is based
significantly off of the normal vector.

This gives us the gradient sphere shown in @normals, where
red is assigned to the $x$ axis, green to they $y$ axis, and blue
to the $z$ axis - and correctly, green is strong on the top (because the $y$
axis goes upwards) and blue is strong on the side (because the $z$ axis goes
approximately leftwards).

#figure(
  image("assets/normals.png", width: 70%),
  caption: [
    A sphere colored according to its surface normals
  ],
)<normals>

=== Traits: `Hittable`

In Rust, a `trait` describes a property of `struct`s. A `Hittable` struct is
one that can be hit by a ray. The function signature of `Hittable` is this:
```rs
pub trait Hittable {
    fn hit(&self, ray: &Ray3, ray_t: Interval) -> Option<HitRecord>;
}
```

When a `Hittable` object is hit, it takes in a `Ray` and the argument
`ray_t`; an interval of positions along the ray where a hit is valid.
This allows us to restrict hit objects to only those closest to us.

It returns a `HitRecord`, which is a collection of information about the hit;
the material of the surface hit, what the normal of the surface was,
the point at which the hit occurred, and what distance across the ray ($t$)
the hit occurred at.

=== Antialiasing

The images that are being created are only 400px wide, so of course there
will be some jaggedness. However, since the pixels are colored according to
whether or not the _center_ of them intersects an object, there's a very
sudden change in color at the edge of the object.
Antialiasing is the solution to this problem. Instead of always calculating a
single pixel with a single ray, each given pixel is sampled multiple times at
slightly varying positions, and the resulting colors are averaged together.

#note[
  Pixels aren't always squares #footnote[https://www.researchgate.net/publication/244986797_A_Pixel_Is_Not_A_Little_Square_A_Pixel_Is_Not_A_Little_Square_A_Pixel_Is_Not_A_Little_Square],
  but it's an assumption that usually holds well enough.
]

How are those positions varied? There are a few options. I chose to sample
from a 1px$times$1px square centered at the center of the pixel, but I
also tried sampling from a disc with a 1px radius. I was curious to know if
sampling from a disc instead of a square affects the resultant rendering.

#figure(
  image("assets/02-20-antialiasing.svg", width: 70%),
  caption: [
    A comparison of antialiasing techniques and samples per pixel
  ],
)<antialiasing>

After some experimentation, there doesn't seem to be a discernable
difference, as far as I can tell. I'll just stick to squares for now for
their simplicity.

Obviously, 100 samples per pixel is significantly smoother than 10,
but the 10x speed increase is much more valuable to me on a
relatively slow MacBook. For development, I'll probably keep a very low
sampling value (\~10 rays/pixel).

=== Bouncing Rays

Finally, the rays actually start being traced! Each time a ray bounces (in a
random direction in a hemisphere off the normal), the degree to which it
impacts the color of the pixel is reduced (multiplied by $1/2$).
To avoid near-infinite recursions causing a stack overflow, rays stop
travelling and instead yield a black color after 10 bounces.

=== Albedo

It's important to stop here and define what an albedo is. The albedo of a
surface refers to the percentage of light that is reflected by that surface.
A surface with $alpha = 0.0$ will appear completely black, because it absorbs
all incoming light. A surface with $alpha = 1.0$ will reflect all light,
but this reflection is not necessarily a mirror. If the reflected light is
diffuse (i.e. the object is matte), it'll appear bright but won't be a mirror.

An albedo can also be expressed in terms of a set of three color spectrums:
red, green, and blue. If a material has an albedo of pure red (which can be
expressed in color form as #display_color("#f00")), red light will be
100% reflected, but green and blue light will be entirely absorbed.
Because only red light is reflected into the camera, the object will appear
red.

=== Rendering the First Scene

Now the first real scene can be rendered (@lambertian): a sphere on
a much larger sphere. Underneath the ball, a shadow is visible.
Both balls have an albedo of 50% on all three color spectra, or
#display_color("#888"). They appear blue because there is more incoming blue
light than red or green (remember, white light consists of red, green,
#underline[and] blue light)

How is there a shadow? There are two ways to think about it.

+ When rays are launched from the camera, if they hit the ground under
  the main sphere they bounce upwards into the upper sphere,
  where they tend to be trapped bouncing up and down until the 10-bounce
  timeout. This means that they don't reach the sky, which is the only
  light source in the scene.
+ Alternatively, it could be though about in the other direction.
  Rays launched from light sources (the sky) are blocked from hitting the
  area under the main sphere.

Both ways of thinking about this result in the same effect. This is why
ray-tracing by firing beams from a camera point works.

#figure(
  image("assets/lambertian.png", width: 70%),
  caption: [
    Two spheres, rendered with ray marching
  ],
)<lambertian>

A better way to bounce light off a sphere (although in this simple case the
result looks very similar) is with a Lambertian distribution
#footnote[https://en.wikipedia.org/wiki/Lambertian_reflectance].

The Lambertian distribution, $B_D = (bold(L) dot bold(N)) "CI"_L$
describes how rays are reflected off of a diffuse (matte) surface.

It's the dot product of the surface's normal vector $bold(N)$ and a
light vector $bold(L)$, multiplied by the color and intensity of the light
hitting the surface.

In other words, the distribution follows Lambert's Cosine Law, that _radiant
intensity_ off a matte object is the same from all angles of observance.
The intensity of a reflected beam is based on its angle compared to the
normal of the surface.

=== Gamma Correction

Gamma correction is a technique to ensure that data isn't wasted
distinguishing between shade that humans can't differentiate between.
With gamma correction, reflectance values should be directly proportional to
the perceived lightness of the pixel.

@gamma-0 shows an image, sliced into strips where the reflectance of the
spheres ranges from 10 to 90%. In other words, 10% of light is reflected on
each bounce in the first strip, 30% by the next, and so on until the final
strip contains spheres reflecting 90% of the light that hits them.

#figure(
  image("assets/gamma-0.png", width: 70%),
  caption: [
    Two spheres, rendered with ray marching, demonstrating a linear gamut
  ],
)<gamma-0>

Sampling the colors of these strips via a color picker, the color of the
ground in the `10%` slice has a value of roughly #display_color("#0e1218").
The slice at `50%` is #display_color("#344155"), and the 90% slice is #display_color("#83a9e1").

HSL (hue, saturation, and lightness) is a color field useful for quantifying
how light a color appears. It can be easily converted to from RGB as
$(C_"max" + C_"min") / 2$, where $C_"max"$ is the most significant color
of red, green, and blue, and $C_"min"$ is the least significant.

A table of those lightness values would look like this:
#figure(
  table(
    columns: 3,
    table.header(
      [Reflectance],[Color (RGB)],[Lightness (HSL)]
    ),
    [10%],[#display_color("#0e1218")],[#display_color_img(luma(7%)) 7%],
    [30%],[#display_color("#202a38")],[#display_color_img(luma(17%)) 17%],
    [50%],[#display_color("#344155")],[#display_color_img(luma(27%)) 27%],
    [70%],[#display_color("#5b7398")],[#display_color_img(luma(48%)) 48%],
    [90%],[#display_color("#83a9e1")],[#display_color_img(luma(70%)) 70%],
  ),
  caption: [Lightness values before gamma correction]
)<t-gamma-0>

The lower three lightness values are much closer together than the higher two;
they're difficult to distinguish between.
Trying to find a "middle-gray" between #display_color_img(luma(7%)) 7% and
#display_color_img(luma(70%)) 70%,
it isn't really #display_color_img(luma(27%)) 27% -- it's much closer to
#display_color_img(luma(48%)) 48%. This means that @t-gamma-0 isn't linear.
More "space" is taken up by darker colors than lighter ones, and humans can't
distinguish between them well.

After converting to Gamma 2 encoding by square-rooting each channel of the
color, @gamma-1 is rendered. It's a lighter image
(because all values are calculated on a scale from `0.0` to `1.0`, so they
all increase), but it's also a more _consistent_ gradient of lightness.
The "middle-gray" between the first and fifth columns appears to be column 3,
not column 4 like in @t-gamma-0; the lightness of the strips ramp up
linearly instead of exponentially.

#figure(
  image("assets/gamma-1.png", width: 70%),
  caption: [
    Two spheres, rendered with ray marching --- $gamma$ 2 corrected and
    demonstrating a consistent gamut
  ],
)<gamma-1>

The table derived from this image, @t-gamma-1, has more linear lightness
values, and the "middle-gray" lightness value of
#display_color_img(luma(47%)) 47% corresponds with the 50% reflectance row.

#figure(
  table(
    columns: 3,
    table.header(
      [Reflectance],[Original Lightness (HSL)],[Corrected Lightness (HSL)]
    ),
    [10%],
    [#display_color_img(luma(7%)) 7%],
    [#display_color_img(luma(26%)) 26%],

    [30%],
    [#display_color_img(luma(17%)) 17%],
    [#display_color_img(luma(41%)) 41%],

    [50%],
    [#display_color_img(luma(27%)) 27%],
    [#display_color_img(luma(47%)) 47%],

    [70%],
    [#display_color_img(luma(48%)) 48%],
    [#display_color_img(luma(67%)) 67%],

    [90%],
    [#display_color_img(luma(70%)) 70%],
    [#display_color_img(luma(83%)) 83%],
  ),
  caption: [Lightness values before and after gamma correction]
)<t-gamma-1>

== More Materials #math.dot *Feb 21 -- 27*

The only material created so far is the the one created earlier, which I'm
calling `Lambertian` but could also be called `Diffuse` or `Matte`.
The first step to a more useful material system is to create a `Material`
trait and allow materials to impart a color, known as an _albedo_, onto the
ray hitting them. Effectively, the color of the beam is multiplied by the
albedo, so a pure red albedo will preserve only red light, while a gray
albedo will reflect all three channels of the incoming light
at half the brightness. You may recognize this idea from the spheres above;
they effectively had a 50% gray albedo because they reflected 50% of the
light hitting them.

=== Metal

Reflective, polished, metal uses simple reflection: the angle of incidence
$theta_i$ is equal to the angle of reflection, $theta_r$, relative to the
normal of the surface, which can be expressed in vector form as $V - 2N (V dot N)$.

=== Debugging Metal

After creating the `Metal` struct and its `Material` implementation, I
created a test scene, shown in @metal-bug.

Because the field of view of the camera is relatively shallow, the spheres
on the two sides of @metal-bug appear to be stretched. This is because of
the limitations of cameras, when we view the world with very different eyes.
If you were to place your face close to the image, it would probably look
normal; the way an image should be rendered is dependent on the distance your
eyes are from the screen.

Using a real-world analog, think of the camera as being in a position
relatively close to the spheres. Since they occupy the sides of your vision,
they appear larger and more stretched than the one in the center.

The sphere on the left has an albedo of light gray
(#display_color_img(luma(80%)) 80% reflectance), while
the sphere on the right has an #display_color_img("#cd9a33") albedo that
differs across the color spectrums. It reflects
#display_color_img("#cd0000") 80% of red light,
#display_color_img("#009a00") 60% of green light, and only
#display_color_img("#000033") 20% of blue light.
Since it reflects so little blue light, it looks like a much
warmer color than the sphere on the left.

Why does it look yellow? #display_color_img("#a00") red +
#display_color_img("#0a0") green $approx$ #display_color_img("#aa0") yellow.
It's important to note that the red light in this situation is still coming
from the sky, even though it appears blue. White light is made of all three
colors, so a light blue like the sky contains a significant amount of
red light, even though it has more blue in it than red.

#figure(
  image("assets/metal-bug.png", width: 70%),
  caption: [
    Two metal spheres surrounding a matte sphere - with a bug
  ],
)<metal-bug>

Actually, there's an issue with the sphere on the left. Since all
three channels of its albedo are the same (80%), it shouldn't be tinting
the colors that it reflects. It should be reducing all three channels
equally, by 20% (for instance, #display_color_img(luma(90%)) $->$
#display_color_img(luma(72%)) or #display_color_img("#90cd00") $->$
#display_color_img("#73a400")).
Looking at the reflection of the grass, though,
it seems to be tinting it very blue.

The issue turned out to be a single line of code:

```diff
- b: self.b * rhs.g,
+ b: self.b * rhs.b,
```

As mentioned above, the albedo is multiplied by the incoming ray to determine
the bounced ray's color. When a ray hit a sphere, the ray's blue channel was
influenced by the sphere's green channel, not its blue channel.
This also caused some slight blue tinting of the grass in the right sphere,
although it's a much more subtle effect. @metal-fix shows the fixed image.

#figure(
  image("assets/metal-fix.png", width: 70%),
  caption: [
    Two correctly rendered metal spheres surrounding a matte sphere
  ],
)<metal-fix>

=== Fuzzy Metal

Perfectly smooth metal is great, but metal is often not perfectly smooth
in the real world. Surfaces have imperfections, which can be expressed as
a percentage of "fuzziness".

After reflecting perfectly, a random nudge can be added to the ray to
determine its actual position, as shown in @fuzz.
The radius of the nudge is proportional to the size of the sphere
and the fuzziness value of the material: @fuzzy-metal shows a sphere
with 30% fuzziness (on the left) and a sphere with 100% fuzziness (on the
right). They're keeping their albedos from the last experiment, so
the left sphere is tending to fuzzily reflect most of the light hitting it,
while the right sphere prefers to reflect mostly red and green light.

#figure(
  image("assets/02-25-fuzz.svg", width: 70%),
  caption: [
    The calculation of fuzziness
  ],
)<fuzz>

#figure(
  image("assets/fuzzy.png", width: 70%),
  caption: [
    30% and 100% fuzzy metal balls
  ],
)<fuzzy-metal>

=== Dielectric Materials

Onto the third type of material: dielectrics.

#note[
  *How do dielectrics work?*

  "Dielectric" mediums are materials that respond to a polarizing
  electric field. Since light is an electric field, the medium oscillates
  at an atomic level when the light passes through. These oscillations
  produce their own electric fields, which interfere to form a wave
  with the same frequency and a different (usually shorter) wavelength.
]

Dielectrics are materials like glass, plastic, air, ceramic, and water.
When a light ray hits a dielectric, it can split into up to two rays:
a *reflected* ray and a *refracted* ray. A *reflected* ray is a ray that
bounces off the surface, just like the rays that have been simulated
in the `Lambertian` and `Metal` materials so far. A *refracted* ray is a ray
that continues through the surface. However, due to the change in
speed a dielectric medium imposes, refracted rays typically
change their angle as they pass through the object's surface -
which is how a rod appears to bend when inserted into water,
and how a glass lens flips an image upside-down.

#note[
  Because several rays are sent through each pixel, it's not necessary to
  simulate _splitting_ the ray. Instead, the program selects with a given
  probability (equal to the percentage of the ray that would become reflected/
  refracted) whether that ray becomes reflected or refracted. This ensures
  that there are always a constant number of rays travelling through the
  scene, which is easier to compute.
]

Refraction is described by Snell's Law
#footnote[https://en.wikipedia.org/wiki/Snell%27s_law],
$eta dot sin theta = eta prime dot sin theta prime$,
where $theta$ and $theta prime$ are the angles from the normal, and
$eta$ and $eta prime$ are the refractive indices of the materials involved.

Ray Tracing in One Weekend goes into more detail on the derivation
#footnote[https://raytracing.github.io/books/RayTracingInOneWeekend.html#dielectrics/snell'slaw],
but eventually this equation can be used to determine the refracted ray
$R prime$ from an incoming ray $R$ and the ratio of refractive indices,
$eta / (eta prime)$.

Depending on the angle of the ray, it may not be possible to find a solution
for Snell's Law. Since $sin theta prime = eta / (eta prime) dot sin theta$,
if the ray is passing from glass into air ($eta = 1.5$ and
$eta prime approx 1.00$) and since $sin theta prime$ cannot be greater than
1.0, we have a condition where $1.5 / 1.0 dot sin theta > 1.0$ causes Snell's
law to produce no solutions. In this case, light cannot be refracted and is
instead reflected.

This can be seen when viewing water from a glancing angle, either from below
or above. Below the water and looking straight up, you can see through, but
if you're near the water's surface it becomes a mirror.

Putting this into effect in @glass, the left ball is rendered as a glass ball
($eta = 1.5$) with a bubble of air ($eta approx 1.00$) inside it.

#figure(
  image("assets/glass.png", width: 70%),
  caption: [
    Hollow glass, Lambertian, and fuzzy metal spheres.
  ],
)<glass>

== Final Features #math.dot *Feb 27 -- Mar 1*

=== Camera Customization

Some useful features for the camera would be:
+ An adjustable FOV (field of view): the angle at which the camera sends out
  rays
+ An adjustable origin position (although `(0, 0, 0)` is probably a good
  default)
+ A depth of field / focal distance: real cameras are only focused (i.e. not
  blurry) at a certain distance.

The details of these are mostly just adjusting code, but the depth of field
improvement has a key physical analog.

Cameras produce a depth of field effect because they send rays out to a lens,
then use that lens to re-converge the rays on a focal point a certain
distance away. We don't need to simulate a lens; instead we can just fire
converging rays from across a disc, instead of diverging rays from a singular
point.

This allows the rendering of @dynamic-cam, an image taken from far away but
with a small FOV of 20°.
At this angle, the reflection of the glass sphere is more apparent -- the sky
is visible at a glancing angle, but not at a direct angle.
The blue Lambertian sphere is refracted through the glass sphere, just like
it would be in the real world.

#figure(
  image("assets/dynamic-cam.png", width: 70%),
  caption: [
    An image rendered with a far-away and low-FOV camera.
  ],
)<dynamic-cam>

=== Final Scene

This concludes the implementation listed in _Ray Tracing in One Weekend_.
This is far from being an optimal ray tracer, and only capable of a very
limited subset of shapes and textures, but it can still render an interesting
scene! @final-scene shows a variety of spheres, with different materials,
colors, and sizes. You can also see the focal distance effect, where closer
and farther objects are blurred, while middle-distance objects are kept in
sharp focus.

A few key things to note about the scene:
- The fuzziness of metal objects is varied, so we can see different levels of
  fuzziness across the spheres.
- Lambertian spheres are the most common, followed by metal and then glass.
- Shadows are apparent, but glass spheres don't seem to have them!
  This is also the case in real life, but it does make them
  look a bit strange.

#figure(
  image("assets/final-scene.png", width: 70%),
  caption: [
    A variety of materials and colors.
  ],
)<final-scene>


#note[
  Does it look cool? *Yes!*\
  Does it seem to work? *Yes!*\
  Did it take twenty minutes to render? *Yes!-wait, what?*

  Unfortunately, it's not the most efficient of ray tracers.
  There are ways this can be fixed, but that's a project for
  another time.
]

=== Quality of Life Features

In the next Action Log, I plan to have moved on to the second guide in the
series, _Ray Tracing: The Next Week_. Before moving on, though, there are some
little features I'd like to implement.

Firstly, it would be nice to have some estimation of when the program is
going to complete, when it's running for twenty minutes. A progress bar
would be a useful feature to have - although it will tend to underestimate
the time because rendering the sky is much faster than rendering the spheres
in the lower parts of the image.


Rust has a great library for this: `indicatif`
#footnote[https://docs.rs/indicatif/latest/indicatif/].


#figure(
  image("assets/progressbar.png", width: 70%),
  caption: [
    An `indicatif` progress bar
  ],
)<progressbar>

This looks great, and it's easy to implement. I only needed five lines of code.
When Rayon is eventually used
#footnote[https://docs.rs/rayon/latest/rayon/] for multi-threading,
`indicatif` has a feature that supports multi-threaded iterators.

The second feature I want to add is being able to export to a format other
than `PPM`. The problem with `PPM` is that it isn't supported natively by
most image viewers, including those that typically render Markdown or Typst
files. In order to view it in Visual Studio Code, I had to install a
designated extension - instead of using the native PNG, JPG, or other standard
image viewers already installed.

The PNG (Portable Network Graphics) format is ubiquitous on the Internet.
Since the specifics of PNG encoding isn't the focus of this project, I'll use
the `png` #footnote[https://docs.rs/png/latest/png/] crate to
encode my images.

#note[
  Instead of re-rendering, I used the _amazing_ ImageMagick
  #footnote[https://imagemagick.org/index.php]
  tool to convert my existing PPM
  files in the assets folder.
]
