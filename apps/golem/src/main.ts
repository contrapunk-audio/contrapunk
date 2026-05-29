import { mount } from 'svelte';
import App from './App.svelte';

const target = document.getElementById('app');

if (!target) {
  throw new Error('Golem UI mount target #app was not found');
}

const app = mount(App, { target });

export default app;
