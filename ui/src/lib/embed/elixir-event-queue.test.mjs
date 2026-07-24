import assert from 'node:assert/strict';
import { ElixirEventQueue } from './elixir-event-queue.js';
import { addVoiceOwner, takeVoiceOwner } from './elixir-ownership.js';

const queue = new ElixirEventQueue(3);
assert.equal(queue.insert({ kind: 0, atFrame: 20, voiceId: 1, seq: 1 }), true);
assert.equal(queue.insert({ kind: 1, atFrame: 10, voiceId: 2, seq: 2 }), true);
assert.equal(queue.insert({ kind: 2, atFrame: 20, voiceId: 3, seq: 3 }), true);
assert.deepEqual(Array.from(queue.voiceId), [2, 1, 3], 'time order is stable for equal frames');
assert.equal(queue.insert({ kind: 3, atFrame: 30, seq: 4 }), false, 'capacity is hard bounded');
queue.removeFirst();
assert.deepEqual(Array.from(queue.voiceId.slice(0, queue.length)), [1, 3]);
queue.clear();
assert.equal(queue.length, 0);

const owners = new Map();
addVoiceOwner(owners, 0, 60, 10);
addVoiceOwner(owners, 0, 60, 11);
addVoiceOwner(owners, 1, 60, 12);
assert.equal(takeVoiceOwner(owners, 1, 60), 12, 'roles own equal pitches independently');
assert.equal(takeVoiceOwner(owners, 0, 60), 10, 'repeated pitches release FIFO');
assert.equal(takeVoiceOwner(owners, 0, 60), 11);
assert.equal(takeVoiceOwner(owners, 0, 60), undefined);
