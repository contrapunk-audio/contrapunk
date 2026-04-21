/**
 * Chain Store — reactive view of the live audio chain topology.
 *
 * Mirrors what the Rust `ChainCommander` exposes on the main thread:
 * a list of `ChainBlock` descriptors (one per block, in order). The
 * Tauri backend updates synchronously when the UI issues add/remove
 * calls, so we re-fetch after every mutation.
 */

import { adapter } from '$lib/adapter';
import type { ChainBlock, ClapPluginDescriptor } from '$lib/adapter/types';

/** Extract the plugin id from a chain block's typeId ("clap:42" -> 42). */
export function parsePluginId(typeId: string): number | null {
	if (!typeId.startsWith('clap:')) return null;
	const n = parseInt(typeId.slice(5), 10);
	return Number.isFinite(n) ? n : null;
}

class ChainStore {
	blocks = $state<ChainBlock[]>([]);
	clapPlugins = $state<ClapPluginDescriptor[]>([]);
	loadingPlugins = $state(false);

	async refresh() {
		try {
			const next = await adapter.listChainBlocks();
			console.log('[chainStore] refresh got', next.length, 'blocks', next);
			this.blocks = next;
			// Backend setup races with the first webview render; if we get
			// an empty list on startup, retry once after a short delay.
			if (next.length === 0) {
				setTimeout(async () => {
					try {
						const retry = await adapter.listChainBlocks();
						if (retry.length > 0) {
							console.log('[chainStore] retry got', retry.length, 'blocks');
							this.blocks = retry;
						}
					} catch {
						/* ignore */
					}
				}, 500);
			}
		} catch (e) {
			console.warn('[chainStore] refresh failed:', e);
			this.blocks = [];
		}
	}

	async removeAt(index: number) {
		// If it's a CLAP block, also drop its controller from the
		// main-thread registry so its GUI+instance get destroyed.
		const target = this.blocks[index];
		const pluginId = target ? parsePluginId(target.typeId) : null;
		try {
			await adapter.removeChainBlock(index);
			if (pluginId !== null) {
				try {
					await adapter.removePlugin(pluginId);
				} catch {
					/* non-fatal */
				}
			}
		} finally {
			await this.refresh();
		}
	}

	async openPluginGui(pluginId: number) {
		await adapter.openPluginGui(pluginId);
	}

	async openPluginGuiEmbedded(
		pluginId: number,
		x: number,
		y: number,
		width: number,
		height: number
	) {
		await adapter.openPluginGuiEmbedded(pluginId, x, y, width, height);
	}

	async setPluginGuiFrame(
		pluginId: number,
		x: number,
		y: number,
		width: number,
		height: number
	) {
		await adapter.setPluginGuiFrame(pluginId, x, y, width, height);
	}

	async closePluginGui(pluginId: number) {
		await adapter.closePluginGui(pluginId);
	}

	async scanPlugins() {
		this.loadingPlugins = true;
		try {
			this.clapPlugins = await adapter.listClapPlugins();
		} catch {
			this.clapPlugins = [];
		} finally {
			this.loadingPlugins = false;
		}
	}

	async addClapPlugin(path: string) {
		await adapter.addClapPluginToChain(path);
		await this.refresh();
	}
}

export const chainStore = new ChainStore();
