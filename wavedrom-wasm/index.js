const SUCCESS_RETURN = 0;
const ERROR_MSGS = [
    "Succesful Build",
    "Invalid JSON",
    "Failed to Render SVG",
    "Invalid UTF-8",
    "Unknown Error",
];

function serialize_color(value) {
    const hex = value.substring(1);
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);

    return (1 << 24) | (r << 16) | (g << 8) | b;
}

function deserialize_color(value) {
    let rgb = (value >>> 0).toString(16).padStart(8, "0");

    return "#" + rgb.substring(2);
}

function deserialize_opt_color(value) {
    if ((value & 0xff000000) == 0) {
        return "#FFFFFF";
    } else {
        return deserialize_color(value);
    }
}

const PARAMETERS = [
    "signal-height",
    "cycle-width",
    "transition-offset",

    [
        [
            "background-enabled",
            (_, elem) => {
                if (elem.checked) {
                    const color_selector = document.getElementById("param:background");
                    return serialize_color(color_selector.value);
                } else {
                    return 0;
                }
            },
            (value, elem) => {
                elem.checked = (value & 0xff000000) != 0;
            },
        ],
        ["background", serialize_color, deserialize_opt_color],
    ],
    "signal-marker-fontsize",
    ["signal-marker-color", serialize_color, deserialize_color],

    "signal-name-fontsize",
    ["signal-name-color", serialize_color, deserialize_color],

    ["signal-gap-color", serialize_color, deserialize_color],
    ["signal-gap-background-color", serialize_color, deserialize_color],

    ["signal-path-color", serialize_color, deserialize_color],

    ["signal-hint-line-color", serialize_color, deserialize_color],

    ["signal-undefined-color", serialize_color, deserialize_color],
    [
        [
            "signal-undefined-background-color-enabled",
            (_, elem) => {
                if (elem.checked) {
                    const color_selector = document.getElementById(
                        "param:signal-undefined-background-color"
                    );
                    return serialize_color(color_selector.value);
                } else {
                    return 0;
                }
            },
            (value, elem) => {
                elem.checked = (value & 0xff000000) != 0;
            },
        ],
        [
            "signal-undefined-background-color",
            serialize_color,
            deserialize_opt_color,
        ],
    ],

    ["bg-box2", serialize_color, deserialize_color],
    ["bg-box3", serialize_color, deserialize_color],
    ["bg-box4", serialize_color, deserialize_color],
    ["bg-box5", serialize_color, deserialize_color],
    ["bg-box6", serialize_color, deserialize_color],
    ["bg-box7", serialize_color, deserialize_color],
    ["bg-box8", serialize_color, deserialize_color],
    ["bg-box9", serialize_color, deserialize_color],

    "padding-figure-top",
    "padding-figure-bottom",
    "padding-figure-left",
    "padding-figure-right",
    "padding-schema-top",
    "padding-schema-bottom",

    "spacing-textbox-to-schema",
    "spacing-groupbox-to-textbox",
    "spacing-line-to-line",

    "group-indicator-width",
    "group-indicator-spacing",
    ["group-indicator-color", serialize_color, deserialize_color],
    "group-indicator-label-spacing",
    "group-indicator-label-fontsize",
    ["group-indicator-label-color", serialize_color, deserialize_color],

    "header-fontsize",
    "header-height",
    ["header-color", serialize_color, deserialize_color],
    "top-cycle-marker-height",
    "top-cycle-marker-fontsize",
    ["top-cycle-marker-color", serialize_color, deserialize_color],

    "footer-fontsize",
    "footer-height",
    ["footer-color", serialize_color, deserialize_color],
    "bottom-cycle-marker-height",
    "bottom-cycle-marker-fontsize",
    ["bottom-cycle-marker-color", serialize_color, deserialize_color],

    "edge-node-fontsize",
    ["edge-node-text-color", serialize_color, deserialize_color],
    ["edge-node-background-color", serialize_color, deserialize_color],

    "edge-text-fontsize",
    ["edge-text-color", serialize_color, deserialize_color],
    ["edge-text-background-color", serialize_color, deserialize_color],

    ["edge-color", serialize_color, deserialize_color],
    ["edge-arrow-color", serialize_color, deserialize_color],
    "edge-arrow-size",

    "register-bar-width",
    "register-bar-height",

    "register-hint-indent",

    "register-name-fontsize",
    "register-bitmarker-fontsize",
    "register-attribute-fontsize",

    "register-padding-top",
    "register-padding-bottom",
    "register-padding-left",
    "register-padding-right",
    
    "register-spacing-lane",
    "register-spacing-attribute",

    "register-offset-bitmarker-x",
    "register-offset-bitmarker-y",
    "register-offset-attribute-y",
];

document.getElementById("input").value = `
{
	signal: [
		{ name: "clk", wave: "p.........." },
		{ name: "req", wave: "0.10......." },
		{ name: "data", wave: "x......2.x.", data: "0xBEEF" },
		{ name: "done", wave: "0......1.0." },
	]
}
`.trim();

document.getElementById("input").addEventListener("keydown", function(e) {
    if (e.key == "Tab") {
        e.preventDefault();
        var start = this.selectionStart;
        var end = this.selectionEnd;

        // set textarea value to: text before caret + tab + text after caret
        this.value =
            this.value.substring(0, start) + "\t" + this.value.substring(end);

        // put caret at right position again
        this.selectionStart = this.selectionEnd = start + 1;
    }
});

function encode_string(str, memory, malloc) {
    if (!("TextEncoder" in window)) {
        throw new Error("No support for TextEncoder");
    }

    const text_encoder = new TextEncoder();

    const memorySize = memory.buffer.byteLength;

    const size = new Blob([str]).size * 2;
    const ptr = malloc(size);

    // Claim memory that will hold the UTF-8 string
    const array = new Uint8Array(memory.buffer, ptr, size);

    // Write UTF-8 form into memory
    const { read, written } = text_encoder.encodeInto(str, array);

    // If our memory did not have enough space, the string is truncated.
    if (read != str.length) {
        alert("String is too big! Truncated string...");
    }

    // Return the address + length of the string in memory
    return [ptr, written];
}

function decode_string(memory, addr, length) {
    if (!("TextDecoder" in window)) {
        throw new Error("No support for TextDecoder");
    }

    const text_decoder = new TextDecoder();
    const array = memory.buffer.slice(addr, addr + length);

    return text_decoder.decode(array);
}

function render_svg(input, output, error, memory, malloc, free, render) {
    const [ptr, length] = encode_string(input.value, memory, malloc);
    const rptr = render(ptr, length);
    const array = new Uint8Array(memory.buffer, rptr, 5);

    const return_code = Math.min(array[0], ERROR_MSGS.length - 1);

    error.innerText = ERROR_MSGS[return_code];

    if (return_code == SUCCESS_RETURN) {
        makeVisible("success-icon");
        makeInvisible("failure-icon");

        const size =
            (array[1] << 24) | (array[2] << 16) | (array[3] << 8) | array[4];
        const out = decode_string(memory, rptr + 5, size);
        output.innerHTML = out;

        free(rptr, size + 5);
    } else {
        makeInvisible("success-icon");
        makeVisible("failure-icon");

        free(rptr, 1);
    }
}

fetch("./wavedrom.wasm")
    .then((response) => response.arrayBuffer())
    .then((bytes) => WebAssembly.instantiate(bytes, { env: {} }))
    .then((results) => {
        let mod = results.instance;
        let {
            malloc,
            free,
            render,
            memory,
            modify_parameter,
            get_parameter,
            merge_in_skin,
			reset_parameters,
            export_parameters,
        } = mod.exports;

        const input = document.getElementById("input");
        const output = document.getElementById("output");
        const error = document.getElementById("status-message");

        const rerender = () => {
            render_svg(input, output, error, memory, malloc, free, render);
        };

        input.onchange = rerender;
        input.onkeyup = rerender;

        // Call the render method initialially
        rerender();

        const option_handler = (elem, idx, serializer) => {
            const parameter = idx;
            const value = serializer(elem.value, elem);

            modify_parameter(parameter, value);
            rerender();
        };

        function refresh_parameter_values() {
            for (let idx = 0; idx < PARAMETERS.length; idx += 1) {
                const item = PARAMETERS[idx];

                if (Array.isArray(item) && Array.isArray(item[0])) {
                    for (let i = 0; i < item.length; i += 1) {
                        refresh_parameter_value(idx, item[i])  
                    }
                } else {
                    refresh_parameter_value(idx, item);
                }
            }
        }

        function refresh_parameter_value(idx, item) {
            const value = get_parameter(idx);
            let id, deserializer;
            if (Array.isArray(item)) {
                console.assert(item.length > 2);
                id = item[0];
                deserializer = item[2];
            } else {
                id = item;
                deserializer = (value) => value.toString();
            }

            const elem = document.getElementById("param:" + id);
            if (elem == undefined || elem == null) {
                console.error("Did not find: " + id);
            }

            elem.value = deserializer(value, elem);
        }

        function define_setting_field(idx, item) {
            let id, serializer;
            if (Array.isArray(item)) {
                console.assert(item.length > 2);
                id = item[0];
                serializer = item[1];
            } else {
                id = item;
                serializer = (value) => parseInt(value);
            }

            const elem = document.getElementById("param:" + id);
            if (elem == undefined || elem == null) {
                console.error("Did not find: " + id);
            }

            const handler = () => option_handler(elem, idx, serializer);
            elem.onchange = handler;
            elem.onkeyup = handler;
        }

        refresh_parameter_values();

        for (let idx = 0; idx < PARAMETERS.length; idx += 1) {
            const item = PARAMETERS[idx];

            if (Array.isArray(item) && Array.isArray(item[0])) {
                for (let i = 0; i < item.length; i += 1) {
                    define_setting_field(idx, item[i]);
                }
            } else {
                define_setting_field(idx, item);
            }
        }

        document
            .getElementById("reset-button")
            .addEventListener("click", () => {
				reset_parameters();
				refresh_parameter_values();
				rerender();
		});

        document
            .getElementById("export-button")
            .addEventListener("click", () => {
                const rptr = export_parameters();
                const array = new Uint8Array(memory.buffer, rptr, 5);
                const return_code = array[0];

                if (return_code != 0) {
                    makeInvisible("success-icon");
                    makeVisible("failure-icon");

                    error.innerHTML = "Failed to get skin";

                    free(rptr, 1);
                }

                const size = (array[1] << 24) | (array[2] << 16) | (array[3] << 8) | array[4];
                const json = decode_string(memory, rptr + 5, size);
                const bb = new Blob([json], { type: "application/json" });
                const a = document.createElement("a");
                a.download = "skin.json";
                a.href = window.URL.createObjectURL(bb);
                a.click();

                free(rptr, size + 5);
            });

        document
            .getElementById("skin-file-button")
            .addEventListener("click", (_) => {
                const skin_file = document.getElementById("skin-file");

                if (skin_file.files.length == 0) {
                    return;
                }

                let file = skin_file.files[0];

                let reader = new FileReader();

                reader.readAsText(file);

                reader.onload = function() {
                    const [ptr, length] = encode_string(reader.result, memory, malloc);
                    const result = merge_in_skin(ptr, length);

                    if (result != 0) {
                        error.innerHTML = "Invalid skin file";
                    }

                    refresh_parameter_values();
                    rerender();
                };

                reader.onerror = function() {
                    makeInvisible("success-icon");
                    makeVisible("failure-icon");
                    error.innerHTML = "Failed to upload file";
                };
            });
    });

function makeVisible(id) {
    const menu = document.getElementById(id);
    if (menu.classList.contains("hidden")) {
        menu.classList.remove("hidden");
    }
}
function makeInvisible(id) {
    const menu = document.getElementById(id);
    if (!menu.classList.contains("hidden")) {
        menu.classList.add("hidden");
    }
}

function toggleVisibility(id) {
    const menu = document.getElementById(id);
    if (menu.classList.contains("hidden")) {
        menu.classList.remove("hidden");
    } else {
        menu.classList.add("hidden");
    }
}

function exportToSvg() {
    const output = document.getElementById("output");
    const fileContent = output.innerHTML;
    const bb = new Blob([fileContent], { type: "text/svg" });
    const a = document.createElement("a");
    a.download = "figure.svg";
    a.href = window.URL.createObjectURL(bb);
    a.click();
}

function matchDisabledBackground() {
    const is_enabled = document.getElementById("param:background-enabled");
    const color_selector = document.getElementById("param:background");

    color_selector.disabled = !is_enabled.checked;
}

document
    .getElementById("param:background-enabled")
    .addEventListener("change", function(e) {
        matchDisabledBackground();
    });

matchDisabledBackground();