# Data Types

To display general data in signal, we have the `data` property and several
states that can contain data. The data states corresponding to the numbers `2`
to `9` with different background colors. Similarly, the `=` state can also be
used as a data state. The `data` property defines the data that goes into the
data states. If the `data` property is given a string, the string is split
over whitespace and filled into states. The `data` property can also be defined
with an array.

Data states need to be extended using the `.` state, otherwise it is transited
to a new data state.

```wavedrom[with_source]
{
    signal: [
        { name: "data states", wave: "023456789=0" },
        { name: "with text",   wave: "023456789=0", data: "a b c d e f g h i" },
        {},
        { name: "continued",   wave: "2..2..2....", data: [
            "First State",
            "Second State",
            "Third State",
        ]}
    ]
}
```
