# Note on WaveDrom-js

Wavedrom-rs is a port of a [JavaScript implementation with the same
name][wavedrom-js]. This project is not officially associated with that
project, but does try to be an complete replacement for the [Digital Timing
Diagram][dtd-wiki] part of this project.

This project currently supports all features of the JavaScript project with
exception of:

- JavaScript skins
- Custom tspans through JsonML in headers and footers

There is currently no plan to implement these features.

In turn, wavedrom-rs improves on its JavaScript counterpart in several areas.

- Proper handling of UTF-8 character widths
- Minimization of SVG in several areas
- More customization editor
- More correct rendering of edges

[wavedrom-js]: https://wavedrom.com/
[dtd-wiki]: https://en.wikipedia.org/wiki/Digital_timing_diagram
