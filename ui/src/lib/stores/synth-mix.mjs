/**
 * @param {number[]} levels
 * @param {boolean[]} muted
 * @param {number | null} solo
 * @param {number} role
 */
export function appliedMixGain(levels, muted, solo, role) {
	if (muted[role] || (solo !== null && solo !== role)) return 0;
	return levels[role] ?? 1;
}
