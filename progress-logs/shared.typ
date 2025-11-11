#let shared-styles(it) = {
  show link: it => text(rgb("#5343ff"), underline(it))

  it
}


#let purple = rgb("#9333ea")

#let note(text) = align(center)[
  #rect(
    text,
    fill: rgb("e2e8f0"),
    width: 75%,
    radius: 5%,
    inset: 1.5em
  )
]

#let display_color(text) = [
  #box(width: 0.75em, height: 0.75em, fill: rgb(text), radius: 100%)
  #raw(text)
]

#let display_color_img(text) = [
  #box(
    width: 0.75em,
    height: 0.75em,
    fill: rgb(text),
    radius: 100%
  )
]
