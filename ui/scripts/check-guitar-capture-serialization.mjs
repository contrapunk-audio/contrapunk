import assert from 'node:assert/strict';
import { serializeGuitarCaptureOperation } from '../src/lib/audio/guitarCapture.ts';

let releaseStart;
const startBlocked = new Promise((resolve) => { releaseStart = resolve; });
const order = [];
let tail = Promise.resolve();

function enqueue(name, wait = Promise.resolve()) {
	const queued = serializeGuitarCaptureOperation(tail, async () => {
		order.push(`${name}:begin`);
		await wait;
		order.push(`${name}:end`);
	});
	tail = queued.tail;
	return queued.result;
}

const start = enqueue('start', startBlocked);
const restart = enqueue('restart');
const stop = enqueue('stop');
await Promise.resolve();
assert.deepEqual(order, ['start:begin']);
releaseStart();
await Promise.all([start, restart, stop]);
assert.deepEqual(order, [
	'start:begin', 'start:end',
	'restart:begin', 'restart:end',
	'stop:begin', 'stop:end'
]);

const failed = serializeGuitarCaptureOperation(tail, async () => {
	throw new Error('expected');
});
tail = failed.tail;
await assert.rejects(failed.result);
const recovered = serializeGuitarCaptureOperation(tail, async () => order.push('recovered'));
await recovered.result;
assert.equal(order.at(-1), 'recovered');

console.log('guitar capture lifecycle serialization: ok');
