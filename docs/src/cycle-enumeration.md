# Cycle Enumeration Marker

The `head.tick` and the `foot.tock` properties can be set to add cycle
enumeration markers. The `every` property defines every how many cycles a
marker should be put.

```wavedrom[with_source]
{
    signal: [
        { name: "A", wave: "2....." },
        { name: "B", wave: "3....." },
        { name: "C", wave: "4....." },
        { name: "D", wave: "5....." },
        { name: "E", wave: "6....." },
    ],
    head: {
        tick: 42,
    },
    foot: {
        tock: 1,
        every: 2,
    }
}
```
