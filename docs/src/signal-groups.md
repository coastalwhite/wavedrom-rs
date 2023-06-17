# Signal Groups

A sequence of signals can be grouped by putting them into an array. A name /
label for the group can also be added by starting the array with a string.

```wavedrom[with_source]
{
    signal: [
        [
            "group",
            { name: "A", wave: "p...." },
            [
                "embed",
                { name: "B", wave: "2...." },
                { name: "C", wave: "3...." },
            ],
        ],
        [
            { name: "D", wave: "4...." },
            { name: "E", wave: "5...." },
        ]
    ]
}
```
