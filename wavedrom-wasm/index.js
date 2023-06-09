const SUCCESS_RETURN = 0;
const ERROR_MSGS = [
    "Succesful Build",
    "Invalid JSON",
    "Invalid JSON value",
    "Failed to Render SVG",
    "Invalid UTF-8",
    "Unknown Error",
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
    let { malloc, free, render, memory } = mod.exports;

	const input = document.getElementById("input");
	const output = document.getElementById("output");
	const error = document.getElementById("status-message");

    const handler = () => {
        render_svg(input, output, error, memory, malloc, free, render);
    };

    input.onchange = handler;
    input.onkeyup = handler;

    // Call the render method initialially
    handler();
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