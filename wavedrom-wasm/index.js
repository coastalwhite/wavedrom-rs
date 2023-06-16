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
    const r = parseInt(hex.substring(0,2), 16);
    const g = parseInt(hex.substring(2,4), 16);
    const b = parseInt(hex.substring(4,6), 16);

    return (1 << 24) | (r << 16) | (g << 8) | (b);
}

function deserialize_color(value) {
    let rgb = (value>>>0).toString(16).padStart(8, '0');

    return "#" + rgb.substring(2);
}

function deserialize_opt_color(value) {
    if ((value & 0xFF000000) == 0) {
        return "#FFFFFF";
    } else {
        return deserialize_color(value);
    }
}

const PARAMETERS = [
    ['font-size', 0],
    ['background-enabled', 1, (_, elem) => {
        if (elem.checked) {
            const color_selector = document.getElementById("param:background");
            return serialize_color(color_selector.value);
        } else {
            return 0;
        }
    }, (value, elem) => {
        elem.checked = (value & 0xFF000000) != 0;
    }],
    ['background', 1, serialize_color, deserialize_opt_color],
    ['signal-height', 2],
    ['cycle-width', 3],
    ['transition-offset', 4],
    ['marker-font-size', 5],
    ['bg-box2', 6,  serialize_color, deserialize_color],
    ['bg-box3', 7,  serialize_color, deserialize_color],
    ['bg-box4', 8,  serialize_color, deserialize_color],
    ['bg-box5', 9,  serialize_color, deserialize_color],
    ['bg-box6', 10, serialize_color, deserialize_color],
    ['bg-box7', 11, serialize_color, deserialize_color],
    ['bg-box8', 12, serialize_color, deserialize_color],
    ['bg-box9', 13, serialize_color, deserialize_color],

	['padding-figure-top', 14],
	['padding-figure-bottom', 15],
	['padding-figure-left', 16],
	['padding-figure-right', 17],
	['padding-schema-top', 18],
	['padding-schema-bottom', 19],

	['spacing-textbox-to-schema', 20],
	['spacing-groupbox-to-textbox', 21],
	['spacing-line-to-line', 22],
	
	['group-indicator-width', 23],
	['group-indicator-spacing', 24],
	['group-indicator-label-spacing', 25],
	['group-indicator-label-fontsize', 26],
	
	['header-fontsize', 27],
	['header-height', 28],
	['top-cycle-marker-height', 29],
	['top-cycle-marker-fontsize', 30],
	
	['footer-fontsize', 31],
	['footer-height', 32],
	['bottom-cycle-marker-height', 33],
	['bottom-cycle-marker-fontsize', 34],

	['edge-node-fontsize', 35],
    ['edge-node-text-color', 36,  serialize_color, deserialize_color],
    ['edge-node-background-color', 37,  serialize_color, deserialize_color],

	['edge-text-fontsize', 38],
    ['edge-text-color', 39,  serialize_color, deserialize_color],
    ['edge-text-background-color', 40,  serialize_color, deserialize_color],

    ['edge-color', 41,  serialize_color, deserialize_color],
    ['edge-arrow-color', 42,  serialize_color, deserialize_color],
	['edge-arrow-size', 43],
];

document.getElementById('input').value = `
{
	signal: [
		{ name: "clk", wave: "p.........." },
		{ name: "req", wave: "0.10......." },
		{ name: "data", wave: "x......2.x.", data: "0xBEEF" },
		{ name: "done", wave: "0......1.0." },
	]
}
`.trim();

document.getElementById('input').addEventListener('keydown', function(e) {
  if (e.key == 'Tab') {
    e.preventDefault();
    var start = this.selectionStart;
    var end = this.selectionEnd;

    // set textarea value to: text before caret + tab + text after caret
    this.value = this.value.substring(0, start) +
      "\t" + this.value.substring(end);

    // put caret at right position again
    this.selectionStart =
      this.selectionEnd = start + 1;
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
	const array = new Uint8Array(
		memory.buffer, ptr, size,
	);

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
        makeVisible('success-icon');
        makeInvisible('failure-icon');

        const size = (array[1] << 24) |
                     (array[2] << 16) |
                     (array[3] << 8)  |
                      array[4];
        const out = decode_string(memory, rptr + 5, size);
        output.innerHTML = out;

        free(rptr, size + 5);
	} else {
        makeInvisible('success-icon');
        makeVisible('failure-icon');

        free(rptr, 1);
    }
}

fetch("./wavedrom.wasm")
  .then((response) => response.arrayBuffer())
  .then((bytes) => WebAssembly.instantiate(bytes, { env: {} }))
  .then((results) => {
    let module = {};
    let mod = results.instance;
    let { malloc, free, render, memory, modify_parameter, get_parameter_default } = mod.exports;

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
    
    for (let i = 0; i < PARAMETERS.length; i += 1) {
        const [id, idx] = PARAMETERS[i];

        let serializer, deserializer;
        if (PARAMETERS[i].length > 2) {
            serializer = PARAMETERS[i][2];
        } else {
            serializer = (value) => parseInt(value);
        }
        if (PARAMETERS[i].length > 3) {
            deserializer = PARAMETERS[i][3];
        } else {
            deserializer = (value) => { value.toString() };
        }

        const elem = document.getElementById('param:' + id);
        if (elem == undefined || elem == null) {
            console.error("Did not find: " + id);
        }
        
        const value = get_parameter_default(idx);
        if (PARAMETERS[i].length > 3) {
            elem.value = PARAMETERS[i][3](value, elem);
        } else {
            elem.value = value.toString();
        }

        const handler = () => option_handler(elem, idx, serializer);
        elem.onchange = handler;
        elem.onkeyup = handler;
    }
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
	const bb = new Blob([fileContent ], { type: 'text/svg' });
	const a = document.createElement('a');
	a.download = 'figure.svg';
	a.href = window.URL.createObjectURL(bb);
	a.click();
}

function matchDisabledBackground() {
	const is_enabled = document.getElementById("param:background-enabled");
	const color_selector = document.getElementById("param:background");

    color_selector.disabled = !is_enabled.checked;
}

document.getElementById('param:background-enabled').addEventListener('change', function(e) {
    matchDisabledBackground();
});

matchDisabledBackground();