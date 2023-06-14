# Clock

Clock Signals are periodic signals. There are several ways to manage a close
signal.

- *Positive edge* clock with `p` and `P`
- *Negative edge* clock with `n` and `N`
- Manual *positive edge* or *High* with `h` or `H`
- Manual *negitive edge* or *Low* with `l` or `L`

The state's case determines whether to place a edge marker.

- Lowercase does *NOT* contain a edge marker 
- Uppercase does contain a edge marker

The period of a clock signal can be controlled with the `period` option. Any
fractional period will get rounded up to an integer.

```wavedrom[with_source]
{
    signal: [
        { name: "posedge clk",          wave: "p......." },
        { name: "posedge clk marked",   wave: "P......." },
        { name: "posedge clk period=2", wave: "p.",       period: 4 },
        {},
        { name: "negedge clk",          wave: "n......." },
        { name: "negedge clk marked",   wave: "N......." },
        { name: "negedge clk period=2", wave: "n...",     period: 2 },
        {},
        { name: "manual",               wave: "hlh.l..." },
        { name: "manual marked",        wave: "HLH.L..." },
    ]
}
```
