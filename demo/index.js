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

let change_code = () => {};

function render_svg(input, output, error, memory, malloc, free, render) {
    let [ptr, length] = encode_string(input.value, memory, malloc);
    rptr = render(ptr, length);
    const array = new Uint8Array(
        memory.buffer, rptr, 5,
    );

    switch (array[0]) {
        case 0: 
            error.innerText = "";
            break;
        case 1: 
            free(rptr, 1);
            error.innerText = "Invalid JSON";
            return;
        case 2: 
            free(rptr, 1);
            error.innerText = "Invalid JSON value";
            return;
        case 3: 
            free(rptr, 1);
            error.innerText = "Failed to Assemble SVG";
            return;
        case 4: 
            free(rptr, 1);
            error.innerText = "Failed to Render SVG";
            return;
        default:
            free(rptr, 1);
            error.innerText = "Unknown Error";
            return;
    }

    let size = (array[1] << 24) | (array[2] << 16) | (array[3] << 8) | array[4];
    
    const out = decode_string(memory, rptr + 5, size);
    output.innerHTML = out;
    free(rptr, size + 5);
}

fetch("./rust/target/wasm32-unknown-unknown/release/rust.wasm")
  .then((response) => response.arrayBuffer())
  .then((bytes) => WebAssembly.instantiate(bytes, { env: {} }))
  .then((results) => {
    let module = {};
    let mod = results.instance;
    let { malloc, free, render, memory } = mod.exports;

	const input = document.getElementById("input");
	const error = document.getElementById("error");
	const output = document.getElementById("output");

    const handler = () => {
        render_svg(input, output, error, memory, malloc, free, render);
    };

    input.onchange = handler;
    input.onkeyup = handler;

    handler();
  });