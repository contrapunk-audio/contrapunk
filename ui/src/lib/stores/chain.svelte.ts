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

class ChainStore {
	blocks = $state<ChainBlock[]>([]);
	clapPlugins = $state<ClapPluginDescriptor[]>([]);
	loadingPlugins = $state(false);

	async refresh() {
		try {
			this.blocks = await adapter.listChainBlocks();
		} catch {
			this.blocks = [];
		}
	}

	async removeAt(index: number) {
		try {
			await adapter.removeChainBlock(index);
		} finally {
			await this.refresh();
		}
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
