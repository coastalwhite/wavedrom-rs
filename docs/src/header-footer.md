# Header & Footer

Both the `head` and the `foot` properties can be used to add header and footer
text.

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
        text: "Hello World!",
    },
    foot: {
        text: "Bye World!",
    }
}
```
