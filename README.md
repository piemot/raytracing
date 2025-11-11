<div align="center">

# Raytracing

</div>

## Progress Logs

This project was completed over the period of my second term of Grade 12, from March to June 2025.
Over this period, I produced three Progress Logs (written in Typst), which can be accessed below:

<div align="center">

[![View Progress Log 1 Online](https://img.shields.io/badge/Progress%20Log%201-0074d9?style=for-the-badge)](https://piemot.github.io/raytracing/log-1.pdf)
[![View Progress Log 2 Online](https://img.shields.io/badge/Progress%20Log%202-0074d9?style=for-the-badge)](https://piemot.github.io/raytracing/log-2.pdf)
[![View Progress Log 3 Online](https://img.shields.io/badge/Progress%20Log%203-0074d9?style=for-the-badge)](https://piemot.github.io/raytracing/log-3.pdf)

</div>

## Plans

This repository hosts a raytracing engine I'm making for the Capstone segment of my [BC Career Life Connections 12](https://www.vsb.bc.ca/vln/page/11175/career-life-connections-and-capstone-12) course.
It will be based off of the excellent [Raytracing in One Weekend](https://raytracing.github.io/) (*[github](https://github.com/RayTracing/raytracing.github.io)*) series by Peter Shirley, Trevor D Black, and Steve Hollisch, as well as the [open-source contributors to the project](https://github.com/RayTracing/raytracing.github.io/graphs/contributors).

Of course, I will be extending the guide in a number of ways.
* Firstly, I'll be writing the project in Rust instead of the depicted C++. Rust is a language I enjoy coding in more and am more familiar with.
* Secondly, I'll be specializing utility classes like `Vec3` into more typesafe versions; a `Color` will be different from a `Vec3`, which in turn will be different from a `Point3`. This will allow the compiler to do basic dimensional analysis and ensure correctness.
* Finally, I'll likely be implementing other features as I think of them. I'll be tracking my progress in the [JOURNAL.md](JOURNAL.md) file alongside this one, and keeping track of what I do and when I do it.

Since this is an academic project, I will be holding myself to strict academic integrity standards. When programming, a formal list of resources used is often not necessary, but whenever possible I will be attributing resources used in the below [Attributions](#attributions) section.

All work, unless otherwise stated (in [Attributions](#attributions), a [JOURNAL](JOURNAL.md) entry, or a code comment) is my own. This includes the use of AI tooling; tools such as ChatGPT or GitHub Copilot have not been used in this project.

## Attributions

### Basic tooling
* the Rust Programming Language ([rust-lang.org](https://rust-lang.org))
* Visual Studio Code ([code.visualstudio.com](https://code.visualstudio.com/)): My primary programming editor
* vim ([vim.org](https://www.vim.org/)): My backup programming editor
* Excalidraw ([excalidraw.com](https://excalidraw.com/)): Diagramming software

### Libraries
See [Cargo.toml](Cargo.toml) for a full list of dependencies used. [crates.io](https://crates.io) can be searched to find sub-dependencies, source code, licensing information, and attributions.

### Textures
* [Solar System Scope](assets/textures/earth.png),
    [CC BY 4.0](https://creativecommons.org/licenses/by/4.0),
    via Wikimedia Commons
