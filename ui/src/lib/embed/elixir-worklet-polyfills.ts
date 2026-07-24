// AudioWorkletGlobalScope omits these Web APIs, but wasm-bindgen initializes both.
if (typeof globalThis.TextDecoder === 'undefined') {
	(globalThis as typeof globalThis & { TextDecoder: typeof TextDecoder }).TextDecoder = class {
		decode(input?: ArrayBuffer | ArrayBufferView) {
			const bytes = input
				? input instanceof ArrayBuffer
					? new Uint8Array(input)
					: new Uint8Array(input.buffer, input.byteOffset, input.byteLength)
				: new Uint8Array();
			let text = '';
			for (let index = 0; index < bytes.length; ) {
				const first = bytes[index++];
				if (first < 0x80) text += String.fromCodePoint(first);
				else if (first < 0xe0)
					text += String.fromCodePoint(((first & 0x1f) << 6) | (bytes[index++] & 0x3f));
				else if (first < 0xf0)
					text += String.fromCodePoint(
						((first & 0x0f) << 12) |
							((bytes[index++] & 0x3f) << 6) |
							(bytes[index++] & 0x3f)
					);
				else
					text += String.fromCodePoint(
						((first & 0x07) << 18) |
							((bytes[index++] & 0x3f) << 12) |
							((bytes[index++] & 0x3f) << 6) |
							(bytes[index++] & 0x3f)
					);
			}
			return text;
		}
	} as unknown as typeof TextDecoder;
}
if (typeof globalThis.TextEncoder === 'undefined') {
	(globalThis as typeof globalThis & { TextEncoder: typeof TextEncoder }).TextEncoder = class {
		encode(input = '') {
			const bytes: number[] = [];
			for (const character of input) {
				const point = character.codePointAt(0) ?? 0;
				if (point < 0x80) bytes.push(point);
				else if (point < 0x800) bytes.push(0xc0 | (point >> 6), 0x80 | (point & 0x3f));
				else if (point < 0x10000)
					bytes.push(
						0xe0 | (point >> 12),
						0x80 | ((point >> 6) & 0x3f),
						0x80 | (point & 0x3f)
					);
				else
					bytes.push(
						0xf0 | (point >> 18),
						0x80 | ((point >> 12) & 0x3f),
						0x80 | ((point >> 6) & 0x3f),
						0x80 | (point & 0x3f)
					);
			}
			return Uint8Array.from(bytes);
		}
	} as unknown as typeof TextEncoder;
}
