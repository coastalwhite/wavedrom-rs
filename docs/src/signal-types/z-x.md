# High Impedance & Undefined

The *High Impedance* is given by a `z` and is represented by a straight
centered line. The *Undefined* is given by a `x` and is represented by hatch
pattern.

A *Undefined* line needs to be extended with a `.` to avoid a transition.

```wavedrom[with_source]
{
    signal: [
        { name: "data",     wave: "0z..1z.x.." },
        { name: "response", wave: "0x..1x.z.." },
    ]
}
```
