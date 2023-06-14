# Gaps

Gaps in the signal can be added with the `|` state. This extends the previous
signal by one and placed a gap indicator over extension cycle.

```wavedrom[with_source]
{
    signal: [
        { name: "clk",  wave: "p..|.." },
        { name: "req",  wave: "010|.." },
        { name: "done", wave: "0..|10" },
    ]
}
```
