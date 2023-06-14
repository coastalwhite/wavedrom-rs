# Up & Down

The *Up* and *Down* states are used to gradually transition to a logical `1` and
`0`. *Up* and *Down* are represented with a `u` and `d`.

```wavedrom[with_source]
{
    signal: [
        { name: "up",   wave: "0..u..0..u" },
        { name: "down", wave: "1.d.1.d..1" },
    ]
}
```
