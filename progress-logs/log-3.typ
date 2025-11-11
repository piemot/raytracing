#import "shared.typ": *
#show: shared-styles

#align(center)[
  #text(18pt)[Progress Log #text(purple)[*\#3*]]

  #smallcaps(text(12pt, purple)[Ray Tracing Project])

  Pierson M

  13 June 2025 --- Career Life Connections 12 --- Period 2.1
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

This is the final entry of three Action Logs in this series. The first focused
on creating a basic path-tracing ray-tracer. The second improved it,
adding textures, shapes, optimizations, and lights.
This log will focus noise reduction and the underlying theoretical concepts.
It will probably be a bit more math-focused than the previous entries.

== Monte Carlo Algorithms

Ray tracing is an example of a simulation. There are two types of
randomized simulations (or algorithms in general): Monte Carlo and Las Vegas.
A Las Vegas random algorithm will always produce the exact correct result.
Unfortunately, in most cases this comes at the cost of speed.

An example of a Las Vegas algorithm would be the quick-sort, where, given a
list of values, a pivot is chosen, and values smaller are moved to the left
while values greater than the pivot are moved to the right. Eventually, this
algorithm is guaranteed to produce a sorted list (with an average of
$n log n$ comparisons, where $n$ is the length of the list).

Another Las Vegas algorithm would be selecting a random point within
unit a circle. A simple way to do this would be to select two random values
in `(-1, 1)`, and then check if the distance of the point from the origin
is greater than 1. If it is, the process is repeated. Eventually, this code
will find a valid point inside the unit circle, and it will happen fairly
quickly (although not within any guaranteed time frame).

If the values selected were within `(-10, 10)`, the algorithm would still
be valid, but much slower. In this case, it's easy to think about how much
longer the algorithm will take. The important thing is that it is still
_eventually guaranteed_ to find a valid answer.

In comparison, Monte Carlo algorithms return an answer that may or may not be
valid. Given the example of finding a random point in a circle, a Monte Carlo
algorithm might not bother with checking whether the point chosen has a
distance less than 1 from the origin. This can give incorrect results, but
it will be faster. In some cases, the uncertainty of whether or not the
answer is actually correct is less important than getting _an_ answer in
a relatively quick time frame. Ray tracing is one of those cases.
As long as the image is _mostly_ correct, it's good enough.

=== Estimating Pi -- Stratification

An example of a Monte Carlo algorithm is the estimation of $pi$. Imagining
a square circumscribed around a circle, random points in that square
should fall within the circle $pi / 4$ of the time (because of the ratio of
surface areas: $"area"("circle") / "area"("square") = (pi r^2) / (2 r)^2 = (pi r^2) / (4 r^2)$).

That means that a simple algorithm can be shown in @code.

#figure(
  caption: [
    A Monte Carlo algorithm for finding the value of $pi$
  ]
)[```py
# The number of hits inside the circle
inside_circle = 0
# The number of attempts overall
runs = 0
while True:
    runs += 1
    # A random point, within a (-1, 1) square
    x, y = random(-1, 1), random(-1, 1)
    # Check if the point is inside the circle
    if x*x + y*y < 1:
        inside_circle += 1
    # The estimate of pi is 4 * circle / total runs
    estimate = 4.0 * inside_circle / runs
```]<code>

This is not a fast algorithm. Since it uses random numbers, the results may
vary, but this is what I got when I ran it:
```
Run 10,000  : π = 3.161200000000
Run 20,000  : π = 3.155000000000
Run 30,000  : π = 3.151200000000
Run 40,000  : π = 3.149200000000
Run 50,000  : π = 3.146880000000
...
Run 100,000: π = 3.140360000000
...
Run 150,000: π = 3.143546666667
...
Run 200,000: π = 3.146540000000
```
Relatively quickly, the algorithm finds that the first few digits are `3.14`.
However, it struggles with finding the rest of the digits. In fact,
even at the millionth iteration of this algorithm, it still believed
$pi = 3.144592$.

This is one of the flaws of Monte Carlo algorithms: the Law of Diminishing
Returns. A Monte Carlo algorithm can quickly find a _close_ answer, but
at a certain point starts to lose efficiency very quickly.
For problems in low dimensions, this can be solved with stratification:
the large square can be divided into many smaller squares and one sample is
taken in each. This ensures that the sampled points are well spread out, and
the result converges towards the true value more quickly.

For example, after 1 million iterations, these results are found:
#align(center, table(
  columns: 2,
  stroke: none,
  [Regular Estimate],[$pi approx$ 3.1423480],
  [Stratified Estimate],[$pi approx$ 3.1415200],
  [True Value],[$pi =$ 3.1415926],
))

=== Stratifying a Ray Tracer

Ray tracing can be stratified too. Stratification decreases in efficiency
with the number of dimensions of the problem. Each bounce of a ray adds two
dimensions, so it's not ideal for ray tracing, but it still makes the image a
bit sharper around the edges.

Technically, introducing stratification into a ray tracer adds noise -- meaning that the image is somewhat different from a perfect physical simulation.
However, this noise tends to be less noticeable than other sources (like
aliasing artifacts). Stratification works best with _low-dimensional_ problems. This means that it will be more impactful on the first bounce of a ray, and significantly less impactful on subsequent bounces (as each bounce adds two new dimensions).

== Weighted Light Sampling

In the previous Action Log, I rendered a Cornell Box (see @cornell-box).
The main optimization I'd like to focus on is reducing the noise visible in the image. The noise is present because the amount of light varies across adjacent, similar, pixels. In turn, this effect is due to randomly reflected rays tending to miss the small light source most of the time, which causes a noisy effect. If, instead, they were to hit the light source more often, but were downscaled appropriately, the image would appear less noisy.

#figure(
  image("../assets/cornell-3.png", width: 70%),
  caption: [A rendering of a Cornell Box, from the last Action Log]
)<cornell-box>

The way that this is done is with a statistics concept called *probability distribution functions*, or #strong[PDF]s for short.

The area under the graph of a probability density function is always equal to 1, or 100%, because the sum of all possible positions along the curve must be 100%. Randomly sampling based on a nonuniform PDF tends to produce samples that lie where the PDF is large, and away from places where the PDF was small.

This is perfect, because using a PDF oriented towards the light source increases the number of samples taken of rays that actually hit the light source, and provides an easy way to downscale their impacts accordingly.

More specifically, the color of a pixel can be described analytically by the formula

$ "Color"(bold(x), omega_o, lambda) approx
sum (
  A(bold(x), omega_i, omega_o, lambda) dot
  "pScatter"(bold(x), omega_i, omega_o, lambda) dot
  "Color"(bold(x), omega_o, lambda)
) / p(bold(x), omega_i, omega_o, lambda) $

where $omega$ represents ray directions ($omega_i$ being rays into the collision point, $omega_o$ being the reflected or refracted ray), $bold(x)$ the position where light scatters, and $lambda$ the wavelength (in the case of this ray tracer, the color) of the incoming light.

Essentially, what this formula descrives is that the color of a pixel, based on its position, ray direction, and wavelength, is dependant on four things:

+ $A(...)$, the inherent albedo (color) of an object,
+ $"pScatter"(...)$, the PDF describing how light scatters off an object,
+ $"Color"(...)$, the color of the bounced beam, and
+ $p(...)$, the color (wavelength) of the bounced beam.

#figure(
  image("../assets/new-cornell.png", width: 70%),
  caption: [A rendering of a Cornell Box, with reduced noise because only rays connecting with the light are sampled]
)<new-cornell-box>

== More Content?

I did want to get to more in this Action Log, but implementing the light sampling introduced _so many bugs_ to fix that it wasn't feasible to get to anything else. Still, the light sampling is a massive innovation that really helps to bring the ray tracer to the next level.

Also, after spending four months on this project... I'm definitely feeling a bit burned out. I'm excited to revisit it at some point in the future, maybe when I have a month or two to get new ideas and forget how annoying this project can be sometimes.

Finally, I wanted to mention that -- although I said I wouldn't finish it in the last Action Log -- I did mostly finish the project of text-based configuration. This was super useful when fiddling with Cornell Box-based projects, because I could keep the `cornell_box.toml` (@text-config) file the same and alter the `main.rs` file to add contents.

#figure(
  ```toml
textures = {}

[materials.red]
type = "SolidColor"
color = 0xA60D0D

[materials.green]
type = "SolidColor"
color = 0x1F7326

# Walls
[[objects]]
type = "Parallelogram"
corner = [555, 0, 0]
vectors = [[0, 0, 555], [0, 555, 0]]
material = "green"

[[objects]]
type = "Parallelogram"
corner = [0, 0, 555]
vectors = [[0, 0, -555], [0, 555, 0]]
material = "red"
```,
  caption: [A TOML representation of two walls of a Cornell Box]
)<text-config>
