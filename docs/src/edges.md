# Edges

Edges or arrows can be used to better highlight certain properties of the
diagram. An edge is defined as between two nodes. Nodes are defined on signals
and then edges are defined as `<start><type><end> [label]` in the `edge`
property.

```wavedrom[with_source]
{ signal: [
  { name: 'A', wave: '01........0.',  node: '.a........j' },
  { name: 'B', wave: '0.1.......0.',  node: '..b.......i' },
  { name: 'C', wave: '0..1....0...',  node: '...c....h..' },
  { name: 'D', wave: '0...1..0....',  node: '....d..g...' },
  { name: 'E', wave: '0....10.....',  node: '.....ef....' }
  ],
  edge: [
    'a~b t1', 'c-~a t2', 'c-~>d time 3', 'd~-e',
    'e~>f', 'f->g', 'g-~>h', 'h~>i some text', 'h~->j'
  ]
}
```

There are several types of arrows.

| Identifier | Type | Property |
|-|-|-|
| `~` | Spline | Start and ending horizontal |
| `-~` | Spline | Start horizontal |
| `~-` | Spline | Ending horizontal |
| `-` | Sharp | Shortest path |
| `+` | Sharp | Shortest path with bars |
| `-|-` | Sharp | Start and ending horizontal |
| `-|` | Sharp | Start horizontal |
| `|-` | Sharp | Ending horizontal |

For all except the `+` the `<` and `>` can be appended to add arrows at the
beginning and the end.
