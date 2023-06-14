# Top & Bottom

The Top and Bottom signals correspond to the logical *on* and *off* states. They
are represented by a `1` and `0`. Transitions to and from these states takes a
certain amount of time.

```wavedrom[with_source]
{
    signal: [
        { wave: "101010101" },
        { wave: "000111110" },

        // You can create a vertical gap with an empty signal
        {},

        // You can continue a signal with '.'
        { wave: "1.0.1.0.1" },
    ]
}
```
