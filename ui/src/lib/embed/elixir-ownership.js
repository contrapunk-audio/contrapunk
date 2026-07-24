/** @param {Map<string, number[]>} voices @param {number} role @param {number} midi @param {number} voiceId */
export function addVoiceOwner(voices, role, midi, voiceId) {
	const key = `${role}:${midi}`;
	const owners = voices.get(key) ?? [];
	owners.push(voiceId);
	voices.set(key, owners);
}

/** @param {Map<string, number[]>} voices @param {number} role @param {number} midi */
export function takeVoiceOwner(voices, role, midi) {
	const key = `${role}:${midi}`;
	const owners = voices.get(key);
	if (!owners?.length) return undefined;
	const voiceId = owners.shift();
	if (owners.length === 0) voices.delete(key);
	return voiceId;
}
